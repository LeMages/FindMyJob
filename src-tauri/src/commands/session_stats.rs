//! Commande Tauri des statistiques d'une session.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::error::AppError;
use crate::models::SessionStats;
use crate::services;

/// Calcule les statistiques d'une session (onglet Statistiques du suivi).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Statistiques calculées sur les candidatures envoyées de la session.
#[tauri::command]
pub async fn get_session_stats(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
) -> Result<SessionStats, AppError> {
    services::session_stats::load::session_stats(&ctx.state()?.pool, session_id).await
}
