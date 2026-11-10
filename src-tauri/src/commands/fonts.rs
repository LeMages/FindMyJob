//! Commandes Tauri pour la gestion des polices utilisateur importées pour la
//! compilation Typst : liste, import, suppression.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::documents;
use crate::error::AppError;
use crate::models::FontFolderView;

/// Liste les dossiers de polices importés, avec leurs familles.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// Dossiers triés par nom.
#[tauri::command]
pub async fn list_fonts(ctx: tauri::State<'_, AppContext>) -> Result<Vec<FontFolderView>, AppError> {
    let fonts_dir = ctx.state()?.fonts_dir.clone();
    // Lecture des fichiers et des polices système : hors du runtime async.
    tauri::async_runtime::spawn_blocking(move || documents::list_fonts(&fonts_dir))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {e}")))?
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
/// Le dossier importé, avec ses familles.
#[tauri::command]
pub async fn import_font_folder(
    ctx: tauri::State<'_, AppContext>,
    source_path: String,
) -> Result<FontFolderView, AppError> {
    let fonts_dir = ctx.state()?.fonts_dir.clone();
    tauri::async_runtime::spawn_blocking(move || documents::import_font_folder(&fonts_dir, &source_path))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {e}")))?
}

/// Supprime un dossier de polices importé.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `folder` - Nom du dossier.
///
/// # Returns
///
/// `Ok(())` si supprimé.
#[tauri::command]
pub async fn delete_font(ctx: tauri::State<'_, AppContext>, folder: String) -> Result<(), AppError> {
    documents::delete_font(&ctx.state()?.fonts_dir, &folder)
}
