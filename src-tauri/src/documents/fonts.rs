//! Gestion des polices utilisateur importées pour la compilation Typst :
//! liste, import depuis un dossier source (ex. archive Google Fonts
//! décompressée), suppression. Le filesystem fait foi — pas de table SQL
//! associée. Les noms affichés sont les noms de famille lus dans les
//! fichiers, jamais le nom du dossier.

// Importation des modules standard
use std::path::Path;

// Importation des modules internes
use crate::documents::font_files::scan_font_files;
use crate::documents::font_metadata::read_families;
use crate::error::AppError;
use crate::models::FontFolderView;

/// Décrit un dossier de polices déjà importé.
fn describe_folder(folder: String, dir: &Path) -> Result<FontFolderView, AppError> {
    let scan = scan_font_files(dir).map_err(AppError::Validation)?;
    let families = read_families(dir, &scan.fonts);
    Ok(FontFolderView { folder, families, file_count: scan.fonts.len(), shadowed_families: Vec::new() })
}

/// Liste les dossiers de polices importés par l'utilisateur.
///
/// # Arguments
///
/// * `fonts_dir` - Dossier racine des polices importées, déjà résolu par l'appelant.
///
/// # Returns
///
/// Une entrée par sous-dossier de `polices/`, avec ses familles, triée par
/// dossier. Liste vide si aucune police n'a encore été importée (dossier
/// absent).
pub fn list_fonts(fonts_dir: &Path) -> Result<Vec<FontFolderView>, AppError> {
    if !fonts_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(fonts_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le dossier des polices : {}", e)))?;

    let mut folders = Vec::new();
    for entry in entries.flatten().filter(|e| e.path().is_dir()) {
        let folder = entry.file_name().to_string_lossy().to_string();
        folders.push(describe_folder(folder, &entry.path())?);
    }

    folders.sort_by(|a, b| a.folder.cmp(&b.folder));
    Ok(folders)
}

/// Importe le contenu d'un dossier de police (ex. archive Google Fonts
/// décompressée, sélectionnée à n'importe quel niveau) : copie les fichiers
/// `.ttf`/`.otf`/`.ttc`/`.otc` trouvés, sous-dossiers compris, vers
/// `polices/<nom_du_dossier>/` en gardant leur arborescence, et ignore les
/// autres fichiers (LICENSE, README, etc.). Un import répété vers un dossier
/// déjà existant met à jour ses fichiers (écrasement).
///
/// # Arguments
///
/// * `fonts_dir` - Dossier racine des polices importées, déjà résolu par l'appelant.
/// * `source_path` - Chemin du dossier source sélectionné par l'utilisateur.
///
/// # Returns
///
/// Le dossier importé, avec ses familles et leurs variantes.
pub fn import_font_folder(fonts_dir: &Path, source_path: &str) -> Result<FontFolderView, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation du dossier source et repérage des polices
    // ------------------------------------------------------------------
    let source = Path::new(source_path);
    if !source.is_dir() {
        return Err(AppError::Validation("Le dossier sélectionné est introuvable.".to_string()));
    }
    let folder = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| AppError::Validation("Nom de dossier invalide.".to_string()))?;

    let scan = scan_font_files(source).map_err(AppError::Validation)?;
    if scan.fonts.is_empty() && scan.web_font_count > 0 {
        return Err(AppError::Validation(
            "Les polices WOFF et WOFF2 ne sont pas prises en charge : utilisez un fichier .ttf ou .otf.".to_string(),
        ));
    }
    if scan.fonts.is_empty() {
        return Err(AppError::Validation(
            "Aucun fichier .ttf, .otf, .ttc ou .otc trouvé dans ce dossier.".to_string(),
        ));
    }

    // ------------------------------------------------------------------
    // 2. Lecture des familles avant copie : un dossier sans aucune police
    //    lisible n'est pas importé
    // ------------------------------------------------------------------
    let families = read_families(source, &scan.fonts);
    if families.is_empty() {
        return Err(AppError::Validation(
            "Aucune police lisible dans ce dossier : les fichiers sont peut-être endommagés.".to_string(),
        ));
    }

    // ------------------------------------------------------------------
    // 3. Copie, arborescence conservée (deux fichiers homonymes de
    //    sous-dossiers différents ne s'écrasent pas)
    // ------------------------------------------------------------------
    let dest_dir = fonts_dir.join(&folder);
    for relative in &scan.fonts {
        copy_font_file(&source.join(relative), &dest_dir.join(relative))?;
    }

    Ok(FontFolderView { folder, families, file_count: scan.fonts.len(), shadowed_families: Vec::new() })
}

/// Copie un fichier de police, en créant son dossier de destination.
fn copy_font_file(from: &Path, to: &Path) -> Result<(), AppError> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier {} : {e}", parent.display())))?;
    }
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| AppError::Validation(format!("Copie du fichier {} échouée : {e}", from.display())))
}

/// Supprime un dossier de polices importé.
///
/// # Arguments
///
/// * `fonts_dir` - Dossier racine des polices importées, déjà résolu par l'appelant.
/// * `folder` - Nom du sous-dossier à supprimer dans `polices/`.
///
/// # Returns
///
/// `Ok(())` si le dossier a été supprimé.
pub fn delete_font(fonts_dir: &Path, folder: &str) -> Result<(), AppError> {
    if folder.is_empty() || folder.contains(['/', '\\']) || folder.contains("..") {
        return Err(AppError::Validation("Nom de police invalide.".to_string()));
    }

    let target = fonts_dir.join(folder);
    if !target.is_dir() {
        return Err(AppError::NotFound("Cette police est introuvable.".to_string()));
    }

    std::fs::remove_dir_all(&target).map_err(|e| AppError::Validation(format!("Suppression impossible : {}", e)))
}
