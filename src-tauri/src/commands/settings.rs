//! Commandes Tauri pour les paramètres applicatifs : clé API, modèle LLM,
//! chemins de stockage, test de connexion.

// Importation des modules standard
use std::path::Path;
use std::time::Instant;

// Importation des modules internes
use crate::db::init::{resolve_data_dir, DbState};
use crate::db::settings;
use crate::error::AppError;
use crate::llm::client::{self, DEFAULT_MODEL};
use crate::security;

/// Statut de la clé API, sans jamais exposer sa valeur réelle.
#[derive(serde::Serialize)]
pub struct ApiKeyStatus {
    exists: bool,
    source: &'static str,
}

/// Résultat du test de connexion à l'API Claude.
#[derive(serde::Serialize)]
pub struct TestConnectionResult {
    success: bool,
    model: String,
    message: String,
    latency_ms: u64,
}

/// Retourne le statut de la clé API Anthropic configurée, sans jamais
/// révéler sa valeur.
///
/// # Returns
///
/// `ApiKeyStatus` indiquant si une clé existe et sa source (`.env` en
/// développement, trousseau OS en production).
#[tauri::command]
pub fn get_api_key_status() -> Result<ApiKeyStatus, AppError> {
    Ok(ApiKeyStatus {
        exists: security::has_api_key(),
        source: security::key_source(),
    })
}

/// Enregistre (ou remplace) la clé API Anthropic.
///
/// # Arguments
///
/// * `key` - Nouvelle valeur de la clé API.
///
/// # Returns
///
/// `Ok(())` si l'écriture a réussi.
#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), AppError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(AppError::Validation("La clé API est requise.".to_string()));
    }
    security::set_api_key(key)
}

/// Supprime la clé API Anthropic enregistrée.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi.
#[tauri::command]
pub fn delete_api_key() -> Result<(), AppError> {
    security::delete_api_key()
}

/// Retourne l'identifiant du modèle Claude actuellement configuré.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// Le modèle configuré en base, ou [`DEFAULT_MODEL`] si aucun n'est enregistré.
#[tauri::command]
pub async fn get_llm_model(state: tauri::State<'_, DbState>) -> Result<String, AppError> {
    settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await
}

/// Change le modèle Claude utilisé pour les appels LLM.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `model` - Identifiant du modèle à sélectionner, doit figurer dans
///   [`client::AVAILABLE_MODELS`].
///
/// # Returns
///
/// `Ok(())` si le modèle a été enregistré.
#[tauri::command]
pub async fn set_llm_model(state: tauri::State<'_, DbState>, model: String) -> Result<(), AppError> {
    if !client::AVAILABLE_MODELS.contains(&model.as_str()) {
        return Err(AppError::Validation(format!("Modèle inconnu : {model}")));
    }
    settings::set(&state.pool, settings::KEY_LLM_MODEL, &model).await
}

/// Retourne le thème d'affichage actuellement configuré.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// Le thème configuré en base, ou [`settings::DEFAULT_THEME`] si aucun n'est enregistré.
#[tauri::command]
pub async fn get_theme(state: tauri::State<'_, DbState>) -> Result<String, AppError> {
    settings::resolve_theme(&state.pool).await
}

/// Change le thème d'affichage de l'application.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `theme` - Thème à appliquer, doit figurer dans [`settings::VALID_THEMES`].
///
/// # Returns
///
/// `Ok(())` si le thème a été enregistré.
#[tauri::command]
pub async fn set_theme(state: tauri::State<'_, DbState>, theme: String) -> Result<(), AppError> {
    if !settings::VALID_THEMES.contains(&theme.as_str()) {
        return Err(AppError::Validation(format!("Thème inconnu : {theme}")));
    }
    settings::set(&state.pool, settings::KEY_THEME, &theme).await
}

/// Retourne le chemin du dossier de documents de l'application.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Chemin absolu du dossier de documents.
#[tauri::command]
pub fn get_documents_path(app: tauri::AppHandle) -> Result<String, AppError> {
    Ok(resolve_data_dir(&app).to_string_lossy().to_string())
}

/// Ouvre un chemin dans l'explorateur de fichiers natif. Si le chemin
/// désigne un fichier, ouvre son dossier parent.
///
/// # Arguments
///
/// * `path` - Chemin du fichier ou du dossier à révéler.
///
/// # Returns
///
/// `Ok(())` si l'explorateur de fichiers a pu être lancé.
#[tauri::command]
pub fn open_folder(path: String) -> Result<(), AppError> {
    let target = Path::new(&path);
    let folder = if target.is_file() {
        target.parent().unwrap_or(target)
    } else {
        target
    };
    open::that(folder).map_err(|e| AppError::Validation(e.to_string()))
}

/// Teste la connexion à l'API Claude avec la clé et le modèle actuellement
/// configurés (message court, `max_tokens: 10`).
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// `TestConnectionResult` : les échecs attendus (clé invalide, réseau, refus
/// de l'API) sont retournés avec `success: false`, pas comme une erreur.
#[tauri::command]
pub async fn test_api_connection(state: tauri::State<'_, DbState>) -> Result<TestConnectionResult, AppError> {
    let model = settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;

    let started = Instant::now();
    let result = client::call_claude_with_tokens(
        "Réponds uniquement par le mot OK.",
        "Test de connexion.",
        None,
        10,
        &model,
    )
    .await;
    let latency_ms = started.elapsed().as_millis() as u64;

    let (success, message) = match result {
        Ok(_) => (true, "Connexion réussie.".to_string()),
        Err(e) => (false, e.to_string()),
    };

    Ok(TestConnectionResult { success, model, message, latency_ms })
}
