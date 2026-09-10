//! Commandes Tauri pour les formations du profil.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{formations, users};
use crate::error::AppError;
use crate::models::{CreateFormationInput, FormationView};

/// Ajoute une formation au profil de l'utilisateur.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données de la formation à créer.
///
/// # Returns
///
/// `FormationView` fraîchement créée.
#[tauri::command]
pub async fn add_formation(
    state: tauri::State<'_, DbState>,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    formations::insert(&state.pool, user.id, input).await
}

/// Met à jour une formation existante.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la formation à modifier.
/// * `input` - Nouvelles données de la formation.
///
/// # Returns
///
/// `FormationView` mise à jour.
#[tauri::command]
pub async fn update_formation(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    formations::update(&state.pool, id, input).await
}

/// Supprime une formation du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la formation à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_formation(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    formations::delete(&state.pool, id).await
}
