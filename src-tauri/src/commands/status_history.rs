//! Commandes Tauri pour l'historique de statut des candidatures.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::error::AppError;
use crate::models::StatusHistoryView;

/// Liste la timeline de suivi d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
///
/// # Returns
///
/// Entrées dans l'ordre chronologique.
#[tauri::command]
pub async fn get_status_history(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
) -> Result<Vec<StatusHistoryView>, AppError> {
    db::status_history::list_for_candidature(&ctx.state()?.pool, candidature_id).await
}
