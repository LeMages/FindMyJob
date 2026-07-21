//! Accès et logique de persistance pour l'historique de statut des candidatures.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::error::AppError;
use crate::models::StatusHistoryView;

/// Enregistre un changement de statut dans l'historique d'une candidature.
///
/// # Arguments
///
/// * `tx` - Transaction SQLite active (l'écriture doit rester atomique avec
///   la modification du statut elle-même).
/// * `candidature_id` - Identifiant de la candidature concernée.
/// * `ancien_status` - Statut précédent, `None` s'il s'agit de la création.
/// * `nouveau_status` - Nouveau statut de la candidature.
/// * `content` - Message (mail ou message de candidature) ayant accompagné
///   le changement, `None` s'il n'y en a pas (ex. « Postuler sans message »).
///
/// # Returns
///
/// `Ok(())` une fois l'entrée insérée.
pub async fn record(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    candidature_id: i64,
    ancien_status: Option<&str>,
    nouveau_status: &str,
    content: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO status_history (candidature_id, ancien_status, nouveau_status, content) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(candidature_id)
    .bind(ancien_status)
    .bind(nouveau_status)
    .bind(content)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Liste l'historique de statut d'une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entrées triées par date de changement croissante.
pub async fn list_for_candidature(
    pool: &SqlitePool,
    candidature_id: i64,
) -> Result<Vec<StatusHistoryView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, ancien_status, nouveau_status, content, \
                CAST(changed_at AS TEXT) AS changed_at \
         FROM status_history \
         WHERE candidature_id = ? \
         ORDER BY changed_at",
    )
    .bind(candidature_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<StatusHistoryView, sqlx::Error> {
            Ok(StatusHistoryView {
                id: r.try_get("id")?,
                candidature_id: r.try_get("candidature_id")?,
                ancien_status: r.try_get("ancien_status")?,
                nouveau_status: r.try_get("nouveau_status")?,
                content: r.try_get("content")?,
                changed_at: r.try_get("changed_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}
