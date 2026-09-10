//! Compilation d'un document Typst (.typ) en PDF.

// Importation des modules standard
use std::path::PathBuf;

/// Compile une source Typst en PDF.
///
/// Recherche les fichiers annexes du document (images, polices locales) dans
/// `root`, et s'appuie sur les polices système, les polices embarquées de
/// `typst-kit`, ainsi que les polices importées par l'utilisateur (`fonts_dir`).
///
/// # Arguments
///
/// * `content` - Source Typst complète du document.
/// * `root` - Dossier racine utilisé pour résoudre les fichiers importés par le document.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur ; ignoré s'il n'existe pas encore.
///
/// # Returns
///
/// Octets du PDF généré, ou message d'erreur de compilation.
pub fn compile_to_pdf(content: String, root: PathBuf, fonts_dir: PathBuf) -> Result<Vec<u8>, String> {
    use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};

    let mut font_options = TypstKitFontOptions::default()
        .include_system_fonts(true)
        .include_embedded_fonts(true);
    if fonts_dir.is_dir() {
        font_options = font_options.include_dirs([fonts_dir]);
    }

    let engine = TypstEngine::builder()
        .main_file(content.as_str())
        .with_file_system_resolver(root)
        .search_fonts_with(font_options)
        .build();

    let doc = engine
        .compile()
        .output
        .map_err(|e| format!("Erreur de compilation Typst : {:?}", e))?;

    typst_pdf::pdf(&doc, &Default::default())
        .map_err(|e| format!("Erreur de génération PDF : {:?}", e))
}
