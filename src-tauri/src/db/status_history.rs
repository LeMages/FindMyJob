//! Accès et logique de persistance pour l'historique de statut des candidatures.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::error::AppError;
use crate::models::StatusHistoryView;

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
        "SELECT id, candidature_id, ancien_status, nouveau_status, \
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
                changed_at: r.try_get("changed_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}
