//! Repérage des fichiers de police dans un dossier, sous-dossiers compris
//! (une archive Google Fonts range ses fichiers statiques sous `static/`).

// Importation des modules standard
use std::path::{Path, PathBuf};

/// Extensions lues par Typst, comparées en minuscule : fichiers simples et
/// collections.
const FONT_EXTENSIONS: [&str; 4] = ["ttf", "otf", "ttc", "otc"];

/// Formats web, que Typst ne sait pas lire.
const WEB_FONT_EXTENSIONS: [&str; 2] = ["woff", "woff2"];

/// Profondeur maximale explorée sous le dossier choisi : une archive de
/// police dépasse rarement deux niveaux.
const MAX_DEPTH: usize = 8;

/// Fichiers de police trouvés dans un dossier.
///
/// # Attributes
///
/// * `fonts` - Chemins des fichiers lisibles par Typst, relatifs au dossier,
///   triés.
/// * `web_font_count` - Nombre de fichiers WOFF/WOFF2 rencontrés.
pub struct FontFileScan {
    pub fonts: Vec<PathBuf>,
    pub web_font_count: usize,
}

/// Extension d'un fichier, en minuscule.
fn lowercase_extension(path: &Path) -> Option<String> {
    path.extension().and_then(|ext| ext.to_str()).map(str::to_lowercase)
}

/// Indique si un fichier est une police lisible par Typst (`.ttf`, `.otf`,
/// `.ttc`, `.otc`, insensible à la casse).
fn is_font_file(path: &Path) -> bool {
    lowercase_extension(path).is_some_and(|ext| FONT_EXTENSIONS.contains(&ext.as_str()))
}

/// Indique si un fichier est une police web (`.woff`, `.woff2`).
fn is_web_font_file(path: &Path) -> bool {
    lowercase_extension(path).is_some_and(|ext| WEB_FONT_EXTENSIONS.contains(&ext.as_str()))
}

/// Parcourt un dossier et ses sous-dossiers (jusqu'à `MAX_DEPTH` niveaux)
/// à la recherche de fichiers de police.
///
/// # Arguments
///
/// * `root` - Dossier à parcourir.
///
/// # Returns
///
/// `FontFileScan`, ou message d'erreur si un dossier est illisible.
pub fn scan_font_files(root: &Path) -> Result<FontFileScan, String> {
    let mut scan = FontFileScan { fonts: Vec::new(), web_font_count: 0 };
    let mut pending: Vec<(PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

    // Parcours itératif borné par la profondeur : pas de récursion, et un
    // lien symbolique circulaire ne peut pas boucler indéfiniment.
    while let Some((dir, depth)) = pending.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("Impossible de lire {} : {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if depth < MAX_DEPTH {
                    pending.push((path, depth + 1));
                }
            } else if is_font_file(&path) {
                // `path` est construit depuis `root` : le préfixe est toujours présent.
                if let Ok(relative) = path.strip_prefix(root) {
                    scan.fonts.push(relative.to_path_buf());
                }
            } else if is_web_font_file(&path) {
                scan.web_font_count += 1;
            }
        }
    }

    scan.fonts.sort();
    Ok(scan)
}
