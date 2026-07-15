//! Commandes Tauri pour l'historique de statut des candidatures.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::status_history;
use crate::error::AppError;
use crate::models::StatusHistoryView;

/// Liste l'historique de statut d'une candidature.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entrées triées par date de changement croissante.
#[tauri::command]
pub async fn get_status_history(
    state: tauri::State<'_, DbState>,
    candidature_id: i64,
) -> Result<Vec<StatusHistoryView>, AppError> {
    status_history::list_for_candidature(&state.pool, candidature_id).await
}
