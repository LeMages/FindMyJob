//! DTO représentant une famille de polices importée par l'utilisateur pour la
//! compilation Typst.

/// Famille de polices installée dans `<app_data_dir>/polices/<name>/`.
///
/// # Attributes
///
/// * `name` - Nom du sous-dossier (nom de la famille, ex. `Roboto`).
/// * `file_count` - Nombre de fichiers `.ttf`/`.otf` trouvés dans le dossier.
#[derive(serde::Serialize)]
pub struct FontFamilyView {
    pub name: String,
    pub file_count: usize,
}
