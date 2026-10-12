//! Commandes Tauri utilitaires ne relevant d'aucune entité métier précise.

// Importation des modules internes
use crate::error::AppError;

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
