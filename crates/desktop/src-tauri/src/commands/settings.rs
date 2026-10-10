//! Commandes Tauri pour les paramètres applicatifs : modèle LLM, thème, test
//! de connexion.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::error::AppError;
use crate::models::TestConnectionResult;
use crate::services;
use findmyjob_core::llm::client::DEFAULT_MODEL;

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
    services::settings::test_connection(&ctx.state()?.pool).await
}
