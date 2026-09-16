//! Commandes Tauri pour les expériences professionnelles du profil.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateExperienceInput, ExperienceView};

/// Ajoute une expérience au profil de l'utilisateur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données de l'expérience à créer.
///
/// # Returns
///
/// `ExperienceView` fraîchement créée.
#[tauri::command]
pub async fn add_experience(
    api: tauri::State<'_, ApiClient>,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    api.post("/api/experiences", &input).await
}

/// Met à jour une expérience existante.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de l'expérience à modifier.
/// * `input` - Nouvelles données de l'expérience.
///
/// # Returns
///
/// `ExperienceView` mise à jour.
#[tauri::command]
pub async fn update_experience(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    api.put(&format!("/api/experiences/{id}"), &input).await
}

/// Supprime une expérience du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de l'expérience à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_experience(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/experiences/{id}")).await
}
