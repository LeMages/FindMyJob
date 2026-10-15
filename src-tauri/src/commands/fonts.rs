//! Commandes Tauri pour la gestion des polices utilisateur importées pour la
//! compilation Typst : liste, import, suppression.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::documents;
use crate::error::AppError;
use crate::models::FontFamilyView;

/// Liste les familles de polices importées.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// Familles triées par nom.
#[tauri::command]
pub async fn list_fonts(ctx: tauri::State<'_, AppContext>) -> Result<Vec<FontFamilyView>, AppError> {
    documents::list_fonts(&ctx.state()?.fonts_dir)
}

/// Importe les polices d'un dossier choisi par l'utilisateur.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `source_path` - Dossier source sélectionné.
///
/// # Returns
///
/// La famille importée.
#[tauri::command]
pub async fn import_font_folder(
    ctx: tauri::State<'_, AppContext>,
    source_path: String,
) -> Result<FontFamilyView, AppError> {
    documents::import_font_folder(&ctx.state()?.fonts_dir, &source_path)
}

/// Supprime une famille de polices importée.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `name` - Nom de la famille.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_font(ctx: tauri::State<'_, AppContext>, name: String) -> Result<(), AppError> {
    documents::delete_font(&ctx.state()?.fonts_dir, &name)
}
