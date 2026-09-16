//! Commandes Tauri pour l'utilisateur unique de l'application et l'agrégation
//! de son profil complet (expériences, formations, projets, compétences, langues).

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateUserInput, ProfileDto, UpdateUserInfoInput, UserView};

/// Indique si le profil a déjà été créé (déclenche ou non l'onboarding au démarrage).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// `true` si un utilisateur existe déjà.
#[tauri::command]
pub async fn user_exists(api: tauri::State<'_, ApiClient>) -> Result<bool, AppError> {
    api.get("/api/users/exists").await
}

/// Crée l'utilisateur unique de l'application (fin de l'onboarding).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données saisies dans l'assistant d'onboarding.
///
/// # Returns
///
/// `UserView` fraîchement créé.
#[tauri::command]
pub async fn create_user(
    api: tauri::State<'_, ApiClient>,
    input: CreateUserInput,
) -> Result<UserView, AppError> {
    api.post("/api/users", &input).await
}

/// Récupère l'utilisateur unique de l'application.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// `UserView` du profil.
#[tauri::command]
pub async fn get_user(api: tauri::State<'_, ApiClient>) -> Result<UserView, AppError> {
    api.get("/api/users/me").await
}

/// Récupère le profil complet de l'utilisateur : informations, expériences,
/// formations, projets, compétences et langues.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// `ProfileDto` agrégeant l'ensemble des données du profil.
#[tauri::command]
pub async fn get_profile(api: tauri::State<'_, ApiClient>) -> Result<ProfileDto, AppError> {
    api.get("/api/profile").await
}

/// Met à jour la description libre du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `description` - Nouvelle description.
///
/// # Returns
///
/// `UserView` mis à jour.
#[tauri::command]
pub async fn update_description(
    api: tauri::State<'_, ApiClient>,
    description: Option<String>,
) -> Result<UserView, AppError> {
    api.patch("/api/users/description", &description).await
}

/// Met à jour les informations de contact du profil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Champs à mettre à jour.
///
/// # Returns
///
/// `UserView` mis à jour.
#[tauri::command]
pub async fn update_user_info(
    api: tauri::State<'_, ApiClient>,
    input: UpdateUserInfoInput,
) -> Result<UserView, AppError> {
    api.patch("/api/users/info", &input).await
}
