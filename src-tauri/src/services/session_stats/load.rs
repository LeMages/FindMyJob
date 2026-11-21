//! Point d'entrée des statistiques d'une session : lit les données brutes et
//! l'heure courante, puis délègue le calcul à `compute`.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::SessionStats;
use crate::services::session_stats::compute::compute;

/// Calcule les statistiques d'une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Statistiques de la session, ou erreur de lecture.
pub async fn session_stats(pool: &SqlitePool, session_id: i64) -> Result<SessionStats, AppError> {
    let input = db::session_stats::load(pool, session_id).await?;
    Ok(compute(&input, clock::now(), clock::today_local()))
}
