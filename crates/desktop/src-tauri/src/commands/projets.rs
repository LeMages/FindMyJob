//! Commandes Tauri pour les projets personnels du profil.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateProjetInput, ProjetView};

/// Ajoute un projet au profil de l'utilisateur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données du projet à créer.
///
/// # Returns
///
/// `ProjetView` fraîchement créé.
#[tauri::command]
pub async fn add_projet(
    api: tauri::State<'_, ApiClient>,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    api.post("/api/projets", &input).await
}

/// Met à jour un projet existant.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant du projet à modifier.
/// * `input` - Nouvelles données du projet.
///
/// # Returns
///
/// `ProjetView` mis à jour.
#[tauri::command]
pub async fn update_projet(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    api.put(&format!("/api/projets/{id}"), &input).await
}

/// Supprime un projet du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant du projet à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimé.
#[tauri::command]
pub async fn delete_projet(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/projets/{id}")).await
}
