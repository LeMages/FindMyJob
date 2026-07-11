//! Commandes Tauri pour les projets personnels du profil.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{projets, users};
use crate::error::AppError;
use crate::models::{CreateProjetInput, ProjetView};

/// Ajoute un projet au profil de l'utilisateur.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données du projet à créer.
///
/// # Returns
///
/// `ProjetView` fraîchement créé.
#[tauri::command]
pub async fn add_projet(
    state: tauri::State<'_, DbState>,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    projets::insert(&state.pool, user.id, input).await
}

/// Met à jour un projet existant.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant du projet à modifier.
/// * `input` - Nouvelles données du projet.
///
/// # Returns
///
/// `ProjetView` mis à jour.
#[tauri::command]
pub async fn update_projet(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    projets::update(&state.pool, id, input).await
}

/// Supprime un projet du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant du projet à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimé.
#[tauri::command]
pub async fn delete_projet(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    projets::delete(&state.pool, id).await
}
