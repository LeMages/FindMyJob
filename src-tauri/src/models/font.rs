//! DTOs représentant les polices importées par l'utilisateur pour la
//! compilation Typst.

/// Famille de polices lue dans les métadonnées des fichiers.
///
/// # Attributes
///
/// * `name` - Nom de famille exact, à écrire dans `font:` (ex. `Libertinus Serif`).
/// * `variants` - Variantes disponibles, triées (ex. `400`, `700 italique`,
///   `400–700` pour une police variable).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FontFamilyInfo {
    pub name: String,
    pub variants: Vec<String>,
}

/// Dossier de polices importé dans `<app_data_dir>/polices/<folder>/`.
///
/// # Attributes
///
/// * `folder` - Nom du sous-dossier : identifiant de suppression, jamais
///   affiché comme nom de police.
/// * `families` - Familles contenues, triées par nom.
/// * `file_count` - Nombre de fichiers de police du dossier.
/// * `shadowed_families` - Familles déjà fournies par Typst ou par le
///   système : la version importée ne sert que pour leurs variantes absentes.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FontFolderView {
    pub folder: String,
    pub families: Vec<FontFamilyInfo>,
    pub file_count: usize,
    pub shadowed_families: Vec<String>,
}
