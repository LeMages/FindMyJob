//! Commandes Tauri pour les expériences professionnelles du profil.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{experiences, users};
use crate::error::AppError;
use crate::models::{CreateExperienceInput, ExperienceView};

/// Ajoute une expérience au profil de l'utilisateur.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données de l'expérience à créer.
///
/// # Returns
///
/// `ExperienceView` fraîchement créée.
#[tauri::command]
pub async fn add_experience(
    state: tauri::State<'_, DbState>,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    experiences::insert(&state.pool, user.id, input).await
}

/// Met à jour une expérience existante.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de l'expérience à modifier.
/// * `input` - Nouvelles données de l'expérience.
///
/// # Returns
///
/// `ExperienceView` mise à jour.
#[tauri::command]
pub async fn update_experience(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    experiences::update(&state.pool, id, input).await
}

/// Supprime une expérience du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de l'expérience à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_experience(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    experiences::delete(&state.pool, id).await
}
