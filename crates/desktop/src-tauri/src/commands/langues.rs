//! Commandes Tauri pour les langues du profil.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{langues, users};
use crate::error::AppError;
use crate::models::{CreateLangInput, LangView};

/// Ajoute une langue au profil de l'utilisateur.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données de la langue à créer.
///
/// # Returns
///
/// `LangView` fraîchement créée.
#[tauri::command]
pub async fn add_lang(
    state: tauri::State<'_, DbState>,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    langues::insert(&state.pool, user.id, input).await
}

/// Met à jour une langue existante.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la langue à modifier.
/// * `input` - Nouvelles données de la langue.
///
/// # Returns
///
/// `LangView` mise à jour.
#[tauri::command]
pub async fn update_lang(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    langues::update(&state.pool, id, input).await
}

/// Supprime une langue du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la langue à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_lang(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    langues::delete(&state.pool, id).await
}
