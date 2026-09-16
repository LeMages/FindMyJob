//! Commandes Tauri pour la gestion des polices utilisateur importées pour la
//! compilation Typst : liste, import, suppression.

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::FontFamilyView;

/// Liste les familles de polices importées par l'utilisateur.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// Une entrée par famille de polices, triée par nom.
#[tauri::command]
pub async fn list_fonts(api: tauri::State<'_, ApiClient>) -> Result<Vec<FontFamilyView>, AppError> {
    api.get("/api/fonts").await
}

/// Importe le contenu d'un dossier de police (ex. archive Google Fonts
/// décompressée) dans le dossier de polices de l'agent.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `source_path` - Chemin du dossier source sélectionné par l'utilisateur.
///
/// # Returns
///
/// La famille de polices importée.
#[tauri::command]
pub async fn import_font_folder(
    api: tauri::State<'_, ApiClient>,
    source_path: String,
) -> Result<FontFamilyView, AppError> {
    api.post("/api/fonts", &json!({ "source_path": source_path })).await
}

/// Supprime une famille de polices importée.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `name` - Nom de la famille à supprimer.
///
/// # Returns
///
/// `Ok(())` si le dossier a été supprimé.
#[tauri::command]
pub async fn delete_font(api: tauri::State<'_, ApiClient>, name: String) -> Result<(), AppError> {
    api.delete(&format!("/api/fonts/{name}")).await
}
