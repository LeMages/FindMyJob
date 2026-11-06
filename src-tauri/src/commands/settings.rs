//! Commandes Tauri pour les paramètres applicatifs : modèle LLM, thème,
//! langue de l'interface, test de connexion, clé API Anthropic.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::error::AppError;
use crate::models::{ApiKeyStatus, TestConnectionResult};
use crate::services;
use crate::llm::client::DEFAULT_MODEL;

/// Renvoie le modèle Claude configuré.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// Identifiant du modèle (modèle par défaut si aucun n'est enregistré).
#[tauri::command]
pub async fn get_llm_model(ctx: tauri::State<'_, AppContext>) -> Result<String, AppError> {
    db::settings::resolve_llm_model(&ctx.state()?.pool, DEFAULT_MODEL).await
}

/// Enregistre le modèle Claude choisi.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `model` - Identifiant du modèle.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Validation` pour un modèle inconnu.
#[tauri::command]
pub async fn set_llm_model(ctx: tauri::State<'_, AppContext>, model: String) -> Result<(), AppError> {
    services::settings::set_llm_model(&ctx.state()?.pool, &model).await
}

/// Renvoie le thème d'affichage configuré.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `"clair"`, `"sombre"` ou `"systeme"`.
#[tauri::command]
pub async fn get_theme(ctx: tauri::State<'_, AppContext>) -> Result<String, AppError> {
    db::settings::resolve_theme(&ctx.state()?.pool).await
}

/// Enregistre le thème d'affichage choisi.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `theme` - Thème choisi.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Validation` pour un thème inconnu.
#[tauri::command]
pub async fn set_theme(ctx: tauri::State<'_, AppContext>, theme: String) -> Result<(), AppError> {
    services::settings::set_theme(&ctx.state()?.pool, &theme).await
}

/// Renvoie la langue de l'interface configurée.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `"fr"` ou `"en"`.
#[tauri::command]
pub async fn get_ui_language(ctx: tauri::State<'_, AppContext>) -> Result<String, AppError> {
    db::settings::resolve_ui_lang(&ctx.state()?.pool).await
}

/// Enregistre la langue de l'interface choisie.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `lang` - Langue choisie.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Validation` pour une langue inconnue.
#[tauri::command]
pub async fn set_ui_language(ctx: tauri::State<'_, AppContext>, lang: String) -> Result<(), AppError> {
    services::settings::set_ui_lang(&ctx.state()?.pool, &lang).await
}

/// Teste la connexion à l'API Claude.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `TestConnectionResult`.
#[tauri::command]
pub async fn test_api_connection(ctx: tauri::State<'_, AppContext>) -> Result<TestConnectionResult, AppError> {
    services::settings::test_connection(ctx.state()?).await
}

/// Renvoie l'état de la clé API Anthropic, sans la clé complète.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// `ApiKeyStatus` (présence, source, 4 derniers caractères).
#[tauri::command]
pub async fn get_api_key_status(ctx: tauri::State<'_, AppContext>) -> Result<ApiKeyStatus, AppError> {
    services::settings::api_key_status(&ctx.state()?.api_key).await
}

/// Enregistre la clé API Anthropic dans le trousseau du système (mode PROD).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `key` - Clé saisie.
///
/// # Returns
///
/// L'état de la clé enregistrée, ou `AppError::Validation` (clé vide, mode DEV).
#[tauri::command]
pub async fn save_api_key(ctx: tauri::State<'_, AppContext>, key: String) -> Result<ApiKeyStatus, AppError> {
    services::settings::save_api_key(&ctx.state()?.api_key, key).await
}

/// Supprime la clé API Anthropic du trousseau du système (mode PROD).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// L'état de la clé (absente), ou `AppError::Validation` en mode DEV.
#[tauri::command]
pub async fn delete_api_key(ctx: tauri::State<'_, AppContext>) -> Result<ApiKeyStatus, AppError> {
    services::settings::delete_api_key(&ctx.state()?.api_key).await
}
