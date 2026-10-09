//! Commandes Tauri pour les sessions de candidature.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateSessionInput, SessionView, UpdateSessionInput};

/// Crée une nouvelle session de candidature.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Nom de la session à créer.
///
/// # Returns
///
/// `SessionView` fraîchement créée.
#[tauri::command]
pub async fn create_session(
    api: tauri::State<'_, ApiClient>,
    input: CreateSessionInput,
) -> Result<SessionView, AppError> {
    api.post("/api/sessions", &input).await
}

/// Liste les sessions de l'utilisateur avec leurs statistiques.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// Sessions triées par date de création décroissante.
#[tauri::command]
pub async fn get_sessions(api: tauri::State<'_, ApiClient>) -> Result<Vec<SessionView>, AppError> {
    api.get("/api/sessions").await
}

/// Met à jour le nom et le statut d'une session existante.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la session à modifier.
/// * `input` - Nouvelles valeurs des champs éditables de la session.
///
/// # Returns
///
/// `SessionView` mise à jour.
#[tauri::command]
pub async fn update_session(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: UpdateSessionInput,
) -> Result<SessionView, AppError> {
    api.put(&format!("/api/sessions/{id}"), &input).await
}

/// Supprime une session et toutes les candidatures qui en dépendent.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la session à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_session(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/sessions/{id}")).await
}
