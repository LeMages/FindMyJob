//! Gestion des polices utilisateur importées pour la compilation Typst :
//! liste, import depuis un dossier source (ex. archive Google Fonts
//! décompressée), suppression. Le filesystem fait foi — pas de table SQL
//! associée.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::models::FontFamilyView;

/// Extensions de fichiers de police acceptées, comparées en minuscule.
const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];

/// Résout le dossier racine des polices importées par l'utilisateur, à
/// partir du dossier de données déjà résolu par l'appelant.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
///
/// # Returns
///
/// Chemin absolu vers `<data_dir>/polices/`.
pub fn resolve_fonts_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("polices")
}

/// Indique si un fichier a une extension de police acceptée (`.ttf`/`.otf`,
/// insensible à la casse).
fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| FONT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

/// Compte les fichiers `.ttf`/`.otf` directement présents dans un dossier.
fn count_font_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| entries.flatten().filter(|e| is_font_file(&e.path())).count())
        .unwrap_or(0)
}

/// Liste les familles de polices importées par l'utilisateur.
///
/// # Arguments
///
/// * `fonts_dir` - Dossier racine des polices importées, déjà résolu par l'appelant.
///
/// # Returns
///
/// Une entrée par sous-dossier de `polices/`, triée par nom. Liste vide si
/// aucune police n'a encore été importée (dossier absent).
pub fn list_fonts(fonts_dir: &Path) -> Result<Vec<FontFamilyView>, AppError> {
    if !fonts_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(fonts_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le dossier des polices : {}", e)))?;

    let mut fonts: Vec<FontFamilyView> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let file_count = count_font_files(&e.path());
            FontFamilyView { name, file_count }
        })
        .collect();

    fonts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(fonts)
}

/// Importe le contenu d'un dossier de police (ex. archive Google Fonts
/// décompressée) : copie les fichiers `.ttf`/`.otf` trouvés vers
/// `polices/<nom_du_dossier>/`, en ignorant silencieusement les autres
/// fichiers (LICENSE, README, etc.). Un import répété vers un dossier déjà
/// existant met à jour ses fichiers (écrasement).
///
/// # Arguments
///
/// * `fonts_dir` - Dossier racine des polices importées, déjà résolu par l'appelant.
/// * `source_path` - Chemin du dossier source sélectionné par l'utilisateur.
///
/// # Returns
///
/// La famille de polices importée (nom, nombre de fichiers copiés).
pub fn import_font_folder(fonts_dir: &Path, source_path: &str) -> Result<FontFamilyView, AppError> {
    let source = Path::new(source_path);
    if !source.is_dir() {
        return Err(AppError::Validation("Le dossier sélectionné est introuvable.".to_string()));
    }

    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| AppError::Validation("Nom de dossier invalide.".to_string()))?;

    let dest_dir = fonts_dir.join(&name);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let entries = std::fs::read_dir(source)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le dossier source : {}", e)))?;

    let mut copied = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || !is_font_file(&path) {
            continue;
        }
        let filename = entry.file_name();
        std::fs::copy(&path, dest_dir.join(&filename)).map_err(|e| {
            AppError::Validation(format!("Copie du fichier {} échouée : {}", filename.to_string_lossy(), e))
        })?;
        copied += 1;
    }

    if copied == 0 {
        return Err(AppError::Validation(
            "Aucun fichier .ttf/.otf trouvé dans ce dossier.".to_string(),
        ));
    }

    Ok(FontFamilyView { name, file_count: copied })
}

/// Supprime une famille de polices importée.
///
/// # Arguments
///
/// * `fonts_dir` - Dossier racine des polices importées, déjà résolu par l'appelant.
/// * `name` - Nom de la famille à supprimer (nom du sous-dossier dans `polices/`).
///
/// # Returns
///
/// `Ok(())` si le dossier a été supprimé.
pub fn delete_font(fonts_dir: &Path, name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.contains(['/', '\\']) || name.contains("..") {
        return Err(AppError::Validation("Nom de police invalide.".to_string()));
    }

    let target = fonts_dir.join(name);
    if !target.is_dir() {
        return Err(AppError::NotFound("Cette police est introuvable.".to_string()));
    }

    std::fs::remove_dir_all(&target).map_err(|e| AppError::Validation(format!("Suppression impossible : {}", e)))
}
