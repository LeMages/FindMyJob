//! Commandes Tauri pour l'historique de statut des candidatures.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::StatusHistoryView;

/// Liste l'historique de statut d'une candidature.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entrées triées par date de changement croissante.
#[tauri::command]
pub async fn get_status_history(
    api: tauri::State<'_, ApiClient>,
    candidature_id: i64,
) -> Result<Vec<StatusHistoryView>, AppError> {
    api.get(&format!("/api/candidatures/{candidature_id}/history")).await
}
