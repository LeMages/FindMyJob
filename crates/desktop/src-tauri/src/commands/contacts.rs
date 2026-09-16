//! Commandes Tauri pour les contacts recruteur.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::ContactView;

/// Liste les contacts recruteur d'une candidature.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Contacts triés par date de création croissante.
#[tauri::command]
pub async fn get_contacts(
    api: tauri::State<'_, ApiClient>,
    candidature_id: i64,
) -> Result<Vec<ContactView>, AppError> {
    api.get(&format!("/api/candidatures/{candidature_id}/contacts")).await
}
