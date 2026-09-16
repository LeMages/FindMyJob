//! Commandes Tauri pour les compétences du profil.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateSkillInput, SkillView};

/// Ajoute une compétence au profil de l'utilisateur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données de la compétence à créer.
///
/// # Returns
///
/// `SkillView` fraîchement créée.
#[tauri::command]
pub async fn add_skill(
    api: tauri::State<'_, ApiClient>,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    api.post("/api/skills", &input).await
}

/// Met à jour une compétence existante.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la compétence à modifier.
/// * `input` - Nouvelles données de la compétence.
///
/// # Returns
///
/// `SkillView` mise à jour.
#[tauri::command]
pub async fn update_skill(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    api.put(&format!("/api/skills/{id}"), &input).await
}

/// Supprime une compétence du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la compétence à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_skill(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/skills/{id}")).await
}
