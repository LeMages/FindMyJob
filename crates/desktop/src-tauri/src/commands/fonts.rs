//! Commandes Tauri pour la gestion des polices utilisateur importées pour la
//! compilation Typst : liste, import, suppression.

// Importation des modules internes
use crate::documents::fonts;
use crate::error::AppError;
use crate::models::FontFamilyView;

/// Liste les familles de polices importées par l'utilisateur.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Une entrée par famille de polices, triée par nom.
#[tauri::command]
pub fn list_fonts(app: tauri::AppHandle) -> Result<Vec<FontFamilyView>, AppError> {
    fonts::list_fonts(&app)
}

/// Importe le contenu d'un dossier de police (ex. archive Google Fonts
/// décompressée) dans le dossier `polices/` de l'application.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `source_path` - Chemin du dossier source sélectionné par l'utilisateur.
///
/// # Returns
///
/// La famille de polices importée.
#[tauri::command]
pub fn import_font_folder(app: tauri::AppHandle, source_path: String) -> Result<FontFamilyView, AppError> {
    fonts::import_font_folder(&app, &source_path)
}

/// Supprime une famille de polices importée.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `name` - Nom de la famille à supprimer.
///
/// # Returns
///
/// `Ok(())` si le dossier a été supprimé.
#[tauri::command]
pub fn delete_font(app: tauri::AppHandle, name: String) -> Result<(), AppError> {
    fonts::delete_font(&app, &name)
}
