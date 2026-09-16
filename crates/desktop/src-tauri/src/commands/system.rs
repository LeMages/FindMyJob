//! Commandes Tauri diagnostiques et utilitaires ne relevant d'aucune entité métier précise.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;

/// Résultat de la commande de diagnostic `db_status`.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DbStatusResult {
    db_path: String,
    tables: Vec<String>,
}

/// Retourne le chemin de la base et la liste de ses tables côté agent, pour
/// diagnostic.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// Chemin de la base et noms de tables.
#[tauri::command]
pub async fn db_status(api: tauri::State<'_, ApiClient>) -> Result<DbStatusResult, String> {
    api.get("/api/system/db-status").await.map_err(|e| e.to_string())
}

/// Ouvre une URL dans le navigateur par défaut de l'utilisateur.
///
/// # Arguments
///
/// * `url` - URL à ouvrir.
///
/// # Returns
///
/// `Ok(())` si le navigateur a pu être lancé.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), AppError> {
    open::that(url).map_err(|e| AppError::Validation(e.to_string()))
}
