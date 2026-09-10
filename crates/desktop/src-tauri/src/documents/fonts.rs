//! Résolution de l'AppHandle vers le dossier de données, pour déléguer la
//! gestion des polices utilisateur à `findmyjob_core::documents`.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use crate::db::init::resolve_data_dir;
use crate::error::AppError;
use crate::models::FontFamilyView;

/// Résout le dossier racine des polices importées par l'utilisateur.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Chemin absolu vers `<app_data_dir>/polices/`.
pub fn resolve_fonts_dir(app: &tauri::AppHandle) -> PathBuf {
    findmyjob_core::documents::resolve_fonts_dir(&resolve_data_dir(app))
}

/// Liste les familles de polices importées par l'utilisateur.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Une entrée par sous-dossier de `polices/`, triée par nom.
pub fn list_fonts(app: &tauri::AppHandle) -> Result<Vec<FontFamilyView>, AppError> {
    findmyjob_core::documents::list_fonts(&resolve_fonts_dir(app))
}

/// Importe le contenu d'un dossier de police vers `polices/<nom_du_dossier>/`.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `source_path` - Chemin du dossier source sélectionné par l'utilisateur.
///
/// # Returns
///
/// La famille de polices importée (nom, nombre de fichiers copiés).
pub fn import_font_folder(app: &tauri::AppHandle, source_path: &str) -> Result<FontFamilyView, AppError> {
    findmyjob_core::documents::import_font_folder(&resolve_fonts_dir(app), source_path)
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
pub fn delete_font(app: &tauri::AppHandle, name: &str) -> Result<(), AppError> {
    findmyjob_core::documents::delete_font(&resolve_fonts_dir(app), name)
}
