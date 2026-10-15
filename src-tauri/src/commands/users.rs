//! Commandes Tauri pour l'utilisateur unique de l'application et son profil.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{CreateUserInput, ProfileDto, UpdateUserInfoInput, UserView};

/// Indique si le profil a déjà été créé (déclenche l'onboarding sinon).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `true` si l'utilisateur existe.
#[tauri::command]
pub async fn user_exists(ctx: tauri::State<'_, AppContext>) -> Result<bool, AppError> {
    db::users::user_exists(&ctx.state()?.pool).await
}

/// Crée l'utilisateur unique à la fin de l'onboarding.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données saisies lors de l'onboarding.
///
/// # Returns
///
/// `UserView` fraîchement créé.
#[tauri::command]
pub async fn create_user(ctx: tauri::State<'_, AppContext>, input: CreateUserInput) -> Result<UserView, AppError> {
    db::users::insert_user(&ctx.state()?.pool, input, &clock::now_utc()).await
}

/// Récupère l'identité affichée de l'utilisateur.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `UserView`, ou `AppError::NotFound` si le profil n'existe pas.
#[tauri::command]
pub async fn get_user(ctx: tauri::State<'_, AppContext>) -> Result<UserView, AppError> {
    db::users::fetch_user(&ctx.state()?.pool).await
}

/// Récupère le profil complet (informations et entrées).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `ProfileDto`, ou `AppError::NotFound` si le profil n'existe pas.
#[tauri::command]
pub async fn get_profile(ctx: tauri::State<'_, AppContext>) -> Result<ProfileDto, AppError> {
    db::profile::build_profile(&ctx.state()?.pool).await
}

/// Met à jour la description libre du profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `description` - Nouvelle description.
///
/// # Returns
///
/// `UserView` mis à jour.
#[tauri::command]
pub async fn update_description(
    ctx: tauri::State<'_, AppContext>,
    description: Option<String>,
) -> Result<UserView, AppError> {
    db::users::update_description(&ctx.state()?.pool, description).await
}

/// Met à jour les informations de contact du profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Champs à mettre à jour.
///
/// # Returns
///
/// `UserView` mis à jour.
#[tauri::command]
pub async fn update_user_info(
    ctx: tauri::State<'_, AppContext>,
    input: UpdateUserInfoInput,
) -> Result<UserView, AppError> {
    db::users::update_user_info(&ctx.state()?.pool, input).await
}
