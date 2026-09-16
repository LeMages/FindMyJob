//! Commandes Tauri pour les paramètres applicatifs : clé API, modèle LLM,
//! chemins de stockage, test de connexion.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;

/// Statut de la clé API, sans jamais exposer sa valeur réelle.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiKeyStatus {
    exists: bool,
    source: String,
}

/// Résultat du test de connexion à l'API Claude.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TestConnectionResult {
    success: bool,
    model: String,
    message: String,
    latency_ms: u64,
}

/// Résout le dossier de données de l'application (documents, base de
/// données de développement). En debug, un dossier `dev-data/` à la racine
/// du dépôt ; en release, le dossier de données applicatif standard de l'OS.
///
/// Seule appelante : [`get_documents_path`], purement informatif côté
/// desktop (les documents eux-mêmes sont désormais stockés par l'agent).
#[cfg_attr(debug_assertions, allow(unused_variables))]
fn resolve_data_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        // Le CARGO_MANIFEST_DIR de ce crate est crates/desktop/src-tauri : trois
        // parents remontent jusqu'à la racine du repo (crates/desktop/src-tauri
        // -> crates/desktop -> crates -> racine). Ce unwrap ne peut échouer que
        // si le crate est placé à la racine du disque.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dev-data")
    }
    #[cfg(not(debug_assertions))]
    {
        use tauri::Manager;
        app.path()
            .app_data_dir()
            .expect("failed to resolve app data dir")
    }
}

/// Retourne le statut de la clé API Anthropic configurée côté agent, sans
/// jamais révéler sa valeur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// `ApiKeyStatus` indiquant si une clé existe et sa source.
#[tauri::command]
pub async fn get_api_key_status(api: tauri::State<'_, ApiClient>) -> Result<ApiKeyStatus, AppError> {
    api.get("/api/settings/api-key").await
}

/// Enregistre (ou remplace) la clé API Anthropic côté agent.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `key` - Nouvelle valeur de la clé API.
///
/// # Returns
///
/// `Ok(())` si l'écriture a réussi.
#[tauri::command]
pub async fn set_api_key(api: tauri::State<'_, ApiClient>, key: String) -> Result<(), AppError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(AppError::Validation("La clé API est requise.".to_string()));
    }
    api.put_no_content("/api/settings/api-key", &json!({ "key": key })).await
}

/// Supprime la clé API Anthropic enregistrée côté agent.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi.
#[tauri::command]
pub async fn delete_api_key(api: tauri::State<'_, ApiClient>) -> Result<(), AppError> {
    api.delete("/api/settings/api-key").await
}

/// Retourne l'identifiant du modèle Claude actuellement configuré.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// Le modèle configuré côté agent.
#[tauri::command]
pub async fn get_llm_model(api: tauri::State<'_, ApiClient>) -> Result<String, AppError> {
    api.get("/api/settings/model").await
}

/// Change le modèle Claude utilisé pour les appels LLM.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `model` - Identifiant du modèle à sélectionner.
///
/// # Returns
///
/// `Ok(())` si le modèle a été enregistré.
#[tauri::command]
pub async fn set_llm_model(api: tauri::State<'_, ApiClient>, model: String) -> Result<(), AppError> {
    api.put_no_content("/api/settings/model", &json!({ "model": model })).await
}

/// Retourne le thème d'affichage actuellement configuré.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// Le thème configuré côté agent.
#[tauri::command]
pub async fn get_theme(api: tauri::State<'_, ApiClient>) -> Result<String, AppError> {
    api.get("/api/settings/theme").await
}

/// Change le thème d'affichage de l'application.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `theme` - Thème à appliquer.
///
/// # Returns
///
/// `Ok(())` si le thème a été enregistré.
#[tauri::command]
pub async fn set_theme(api: tauri::State<'_, ApiClient>, theme: String) -> Result<(), AppError> {
    api.put_no_content("/api/settings/theme", &json!({ "theme": theme })).await
}

/// Retourne le chemin du dossier de données de l'application.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Chemin absolu du dossier de données.
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
/// configurés côté agent.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// `TestConnectionResult` : les échecs attendus (clé invalide, réseau, refus
/// de l'API) sont retournés avec `success: false`, pas comme une erreur.
#[tauri::command]
pub async fn test_api_connection(api: tauri::State<'_, ApiClient>) -> Result<TestConnectionResult, AppError> {
    api.post("/api/settings/test-connection", &()).await
}
