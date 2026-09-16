//! Commandes Tauri pour les langues du profil.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateLangInput, LangView};

/// Ajoute une langue au profil de l'utilisateur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données de la langue à créer.
///
/// # Returns
///
/// `LangView` fraîchement créée.
#[tauri::command]
pub async fn add_lang(
    api: tauri::State<'_, ApiClient>,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    api.post("/api/langues", &input).await
}

/// Met à jour une langue existante.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la langue à modifier.
/// * `input` - Nouvelles données de la langue.
///
/// # Returns
///
/// `LangView` mise à jour.
#[tauri::command]
pub async fn update_lang(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    api.put(&format!("/api/langues/{id}"), &input).await
}

/// Supprime une langue du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la langue à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_lang(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/langues/{id}")).await
}
