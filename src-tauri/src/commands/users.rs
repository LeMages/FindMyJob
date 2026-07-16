//! Commandes Tauri pour l'utilisateur unique de l'application et l'agrégation
//! de son profil complet (expériences, formations, projets, compétences, langues).

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{experiences, formations, langues, projets, skills, users};
use crate::error::AppError;
use crate::models::{CreateUserInput, ProfileDto, UpdateUserInfoInput, UserView};

/// Indique si le profil a déjà été créé (déclenche ou non l'onboarding au démarrage).
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// `true` si un utilisateur existe déjà.
#[tauri::command]
pub async fn user_exists(state: tauri::State<'_, DbState>) -> Result<bool, AppError> {
    users::user_exists(&state.pool).await
}

/// Crée l'utilisateur unique de l'application (fin de l'onboarding).
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données saisies dans l'assistant d'onboarding.
///
/// # Returns
///
/// `UserView` fraîchement créé.
#[tauri::command]
pub async fn create_user(
    state: tauri::State<'_, DbState>,
    input: CreateUserInput,
) -> Result<UserView, AppError> {
    users::insert_user(&state.pool, input).await
}

/// Récupère l'utilisateur unique de l'application.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// `UserView` du profil.
#[tauri::command]
pub async fn get_user(state: tauri::State<'_, DbState>) -> Result<UserView, AppError> {
    users::fetch_first_user(&state.pool).await
}

/// Récupère le profil complet de l'utilisateur : informations, expériences,
/// formations, projets, compétences et langues.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// `ProfileDto` agrégeant l'ensemble des données du profil.
#[tauri::command]
pub async fn get_profile(state: tauri::State<'_, DbState>) -> Result<ProfileDto, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let profile_experiences = experiences::list_for_user(&state.pool, uid).await?;
    let profile_formations = formations::list_for_user(&state.pool, uid).await?;
    let profile_projets = projets::list_for_user(&state.pool, uid).await?;
    let profile_skills = skills::list_for_user(&state.pool, uid).await?;
    let profile_langues = langues::list_for_user(&state.pool, uid).await?;

    Ok(ProfileDto {
        user,
        experiences: profile_experiences,
        formations: profile_formations,
        projets: profile_projets,
        skills: profile_skills,
        langues: profile_langues,
    })
}

/// Met à jour la description libre du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `description` - Nouvelle description.
///
/// # Returns
///
/// `UserView` mis à jour.
#[tauri::command]
pub async fn update_description(
    state: tauri::State<'_, DbState>,
    description: Option<String>,
) -> Result<UserView, AppError> {
    users::update_description(&state.pool, description).await
}

/// Met à jour les informations de contact du profil.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Champs à mettre à jour.
///
/// # Returns
///
/// `UserView` mis à jour.
#[tauri::command]
pub async fn update_user_info(
    state: tauri::State<'_, DbState>,
    input: UpdateUserInfoInput,
) -> Result<UserView, AppError> {
    users::update_user_info(&state.pool, input).await
}
