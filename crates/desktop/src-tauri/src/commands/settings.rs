//! Commandes Tauri pour les paramètres applicatifs : modèle LLM, thème, test
//! de connexion.

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;

/// Résultat du test de connexion à l'API Claude.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TestConnectionResult {
    success: bool,
    model: String,
    message: String,
    latency_ms: u64,
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
