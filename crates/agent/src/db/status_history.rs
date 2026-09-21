//! Accès et logique de persistance pour la timeline de suivi des candidatures.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{EventType, StatusHistoryView};

/// Enregistre un événement dans la timeline de suivi d'une candidature.
///
/// # Arguments
///
/// * `tx` - Transaction PostgreSQL active (l'écriture doit rester atomique avec
///   les autres effets de l'événement : changement de statut, entretien...).
/// * `candidature_id` - Identifiant de la candidature concernée.
/// * `event_type` - Type de l'événement (voir `EventType`).
/// * `ancien_status` - Statut précédent, `None` s'il s'agit de la création.
/// * `nouveau_status` - Nouveau statut de la candidature, `None` si
///   l'événement ne change pas le statut.
/// * `content` - Message ou note associé à l'événement, `None` s'il n'y en a pas.
/// * `interview_id` - Entretien concerné, pour `entretien_planifie` /
///   `entretien_passe` uniquement.
///
/// # Returns
///
/// `Ok(())` une fois l'entrée insérée.
pub async fn record(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    candidature_id: i64,
    event_type: EventType,
    ancien_status: Option<&str>,
    nouveau_status: Option<&str>,
    content: Option<&str>,
    interview_id: Option<i64>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO search.status_history \
           (candidature_id, event_type, ancien_status, nouveau_status, content, interview_id) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(candidature_id)
    .bind(event_type.as_str())
    .bind(ancien_status)
    .bind(nouveau_status)
    .bind(content)
    .bind(interview_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_err)?;
    Ok(())
}

/// Liste la timeline de suivi d'une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entrées triées par date de changement croissante.
pub async fn list_for_candidature(
    pool: &PgPool,
    candidature_id: i64,
) -> Result<Vec<StatusHistoryView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, event_type, ancien_status, nouveau_status, content, \
                interview_id, changed_at \
         FROM search.status_history \
         WHERE candidature_id = $1 \
         ORDER BY changed_at",
    )
    .bind(candidature_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<StatusHistoryView, sqlx::Error> {
            Ok(StatusHistoryView {
                id: r.try_get("id")?,
                candidature_id: r.try_get("candidature_id")?,
                event_type: r.try_get("event_type")?,
                ancien_status: r.try_get("ancien_status")?,
                nouveau_status: r.try_get("nouveau_status")?,
                content: r.try_get("content")?,
                interview_id: r.try_get("interview_id")?,
                changed_at: r.try_get("changed_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}
