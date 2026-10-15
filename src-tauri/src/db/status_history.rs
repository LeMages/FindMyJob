//! Accès et logique de persistance pour la timeline de suivi des candidatures.

// Importation des crates externes
use sqlx::{Row, SqliteConnection, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::error::AppError;
use crate::models::{EventType, StatusHistoryView};

/// Entrée de timeline à enregistrer.
///
/// # Attributes
///
/// * `candidature_id` - Identifiant de la candidature concernée.
/// * `event_type` - Type de l'événement (voir `EventType`).
/// * `ancien_status` - Statut précédent, `None` s'il s'agit de la création.
/// * `nouveau_status` - Nouveau statut, `None` si l'événement ne change pas le statut.
/// * `content` - Message ou note associé à l'événement, `None` s'il n'y en a pas.
/// * `interview_id` - Entretien concerné (`entretien_planifie` / `entretien_passe`).
pub struct NewHistoryEntry<'a> {
    pub candidature_id: i64,
    pub event_type: EventType,
    pub ancien_status: Option<&'a str>,
    pub nouveau_status: Option<&'a str>,
    pub content: Option<&'a str>,
    pub interview_id: Option<i64>,
}

/// Enregistre un événement dans la timeline de suivi d'une candidature.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours (l'écriture
///   doit rester atomique avec les autres effets de l'événement).
/// * `entry` - Entrée à enregistrer.
/// * `now` - Horodatage de la transaction, partagé avec ses autres écritures.
///
/// # Returns
///
/// `Ok(())` une fois l'entrée insérée.
pub async fn record(conn: &mut SqliteConnection, entry: &NewHistoryEntry<'_>, now: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO status_history \
           (candidature_id, event_type, ancien_status, nouveau_status, content, interview_id, changed_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(entry.candidature_id)
    .bind(entry.event_type.as_str())
    .bind(entry.ancien_status)
    .bind(entry.nouveau_status)
    .bind(entry.content)
    .bind(entry.interview_id)
    .bind(now)
    .execute(conn)
    .await
    .map_err(map_db_err)?;
    Ok(())
}

/// Liste la timeline de suivi d'une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entrées triées par date de changement croissante, puis par ordre
/// d'insertion à horodatage égal.
pub async fn list_for_candidature(
    pool: &SqlitePool,
    candidature_id: i64,
) -> Result<Vec<StatusHistoryView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, event_type, ancien_status, nouveau_status, content, \
                interview_id, changed_at \
         FROM status_history \
         WHERE candidature_id = $1 \
         ORDER BY changed_at, id",
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
