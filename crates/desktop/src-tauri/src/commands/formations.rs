//! Commandes Tauri pour les formations du profil.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateFormationInput, FormationView};

/// Ajoute une formation au profil de l'utilisateur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données de la formation à créer.
///
/// # Returns
///
/// `FormationView` fraîchement créée.
#[tauri::command]
pub async fn add_formation(
    api: tauri::State<'_, ApiClient>,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    api.post("/api/formations", &input).await
}

/// Met à jour une formation existante.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la formation à modifier.
/// * `input` - Nouvelles données de la formation.
///
/// # Returns
///
/// `FormationView` mise à jour.
#[tauri::command]
pub async fn update_formation(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    api.put(&format!("/api/formations/{id}"), &input).await
}

/// Supprime une formation du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la formation à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_formation(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/formations/{id}")).await
}
