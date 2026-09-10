//! Commandes Tauri pour les compétences du profil.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{skills, users};
use crate::error::AppError;
use crate::models::{CreateSkillInput, SkillView};

/// Ajoute une compétence au profil de l'utilisateur.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données de la compétence à créer.
///
/// # Returns
///
/// `SkillView` fraîchement créée.
#[tauri::command]
pub async fn add_skill(
    state: tauri::State<'_, DbState>,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    skills::insert(&state.pool, user.id, input).await
}

/// Met à jour une compétence existante.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la compétence à modifier.
/// * `input` - Nouvelles données de la compétence.
///
/// # Returns
///
/// `SkillView` mise à jour.
#[tauri::command]
pub async fn update_skill(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    skills::update(&state.pool, id, input).await
}

/// Supprime une compétence du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la compétence à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_skill(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    skills::delete(&state.pool, id).await
}
