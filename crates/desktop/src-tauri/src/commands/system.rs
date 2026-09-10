//! Commandes Tauri diagnostiques et utilitaires ne relevant d'aucune entité métier précise.

// Importation des modules internes
use crate::db::{list_tables, DbState};
use crate::error::AppError;

/// Résultat de la commande de diagnostic `db_status`.
#[derive(serde::Serialize)]
pub struct DbStatusResult {
    db_path: String,
    tables: Vec<String>,
}

/// Retourne le chemin de la base et la liste de ses tables, pour diagnostic.
///
/// # Arguments
///
/// * `state` - État partagé contenant le pool de connexions et le chemin de la base.
///
/// # Returns
///
/// Chemin de la base et noms de tables.
#[tauri::command]
pub async fn db_status(state: tauri::State<'_, DbState>) -> Result<DbStatusResult, String> {
    let tables = list_tables(&state.pool).await.map_err(|e| e.to_string())?;

    Ok(DbStatusResult {
        db_path: state.db_path.to_string_lossy().to_string(),
        tables,
    })
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
