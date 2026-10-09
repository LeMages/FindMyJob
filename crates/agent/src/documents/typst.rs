//! Compilation d'un document Typst (.typ) en PDF : référence autonome, ou
//! template lisant ses données depuis un fichier virtuel `data.json`.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_layout::PagedDocument;

// Importation des modules internes
use crate::documents::typst_diagnostics::describe_compile_error;
use crate::documents::typst_text::extract_text;

/// Nom sous lequel les données d'un template sont exposées au compilateur.
/// Un template les lit avec `json("data.json")`.
pub const TEMPLATE_DATA_FILE: &str = "data.json";

/// Résultat d'une compilation : le PDF, et ce qu'il faut pour le comparer à
/// une référence (nombre de pages, texte rendu).
///
/// # Attributes
///
/// * `pdf` - Octets du PDF généré.
/// * `page_count` - Nombre de pages du document.
/// * `text` - Texte rendu, dans l'ordre de mise en page.
pub struct CompiledDocument {
    pub pdf: Vec<u8>,
    pub page_count: usize,
    pub text: String,
}

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
    compile_document(content, root, fonts_dir).map(|doc| doc.pdf)
}

/// Compile une source Typst autonome et retourne le PDF, le nombre de pages
/// et le texte rendu.
///
/// # Arguments
///
/// * `content` - Source Typst complète du document.
/// * `root` - Dossier racine utilisé pour résoudre les fichiers importés par le document.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
///
/// # Returns
///
/// `CompiledDocument`, ou message d'erreur de compilation.
pub fn compile_document(content: String, root: PathBuf, fonts_dir: PathBuf) -> Result<CompiledDocument, String> {
    compile(content, None, root, fonts_dir)
}

/// Compile un template avec ses données : `data_json` est servi au
/// compilateur comme fichier virtuel `data.json`, sans jamais être évalué
/// comme du markup (les valeurs restent des chaînes).
///
/// # Arguments
///
/// * `template` - Source Typst du template.
/// * `data_json` - Données sérialisées en JSON (schéma canonique).
/// * `root` - Dossier racine utilisé pour résoudre les autres fichiers du template.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
///
/// # Returns
///
/// `CompiledDocument`, ou message d'erreur de compilation.
pub fn compile_template(
    template: String,
    data_json: Vec<u8>,
    root: PathBuf,
    fonts_dir: PathBuf,
) -> Result<CompiledDocument, String> {
    compile(template, Some(data_json), root, fonts_dir)
}

/// Construit le moteur Typst, compile, puis exporte en PDF.
fn compile(
    content: String,
    data_json: Option<Vec<u8>>,
    root: PathBuf,
    fonts_dir: PathBuf,
) -> Result<CompiledDocument, String> {
    let mut font_options = TypstKitFontOptions::default()
        .include_system_fonts(true)
        .include_embedded_fonts(true);
    if fonts_dir.is_dir() {
        font_options = font_options.include_dirs([fonts_dir]);
    }

    // Le fichier virtuel est ajouté avant le resolver disque : un éventuel
    // `data.json` présent dans `root` ne peut pas masquer les vraies données.
    let mut builder = TypstEngine::builder().main_file(content.as_str());
    if let Some(data) = data_json {
        builder = builder.with_static_file_resolver([(TEMPLATE_DATA_FILE, data)]);
    }
    let engine = builder
        .with_file_system_resolver(root)
        .search_fonts_with(font_options)
        .build();

    let doc: PagedDocument = engine
        .compile()
        .output
        .map_err(|e| format!("Erreur de compilation Typst :\n{}", describe_compile_error(&e, &content)))?;

    let pdf = typst_pdf::pdf(&doc, &Default::default()).map_err(|e| format!("Erreur de génération PDF : {:?}", e))?;
    Ok(CompiledDocument { pdf, page_count: doc.pages().len(), text: extract_text(&doc) })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dossier racine sans fichier annexe, et dossier de polices inexistant.
    fn dirs() -> (PathBuf, PathBuf) {
        (std::env::temp_dir(), std::env::temp_dir().join("findmyjob-no-fonts"))
    }

    #[test]
    fn compile_template_lit_les_donnees_comme_du_texte() {
        let (root, fonts) = dirs();
        let template = "#let data = json(\"data.json\")\n#text(data.name)".to_string();
        let data = br#"{"name": "A *b* @c #d"}"#.to_vec();
        let doc = compile_template(template, data, root, fonts).expect("compilation");
        assert_eq!(doc.page_count, 1);
        assert!(doc.text.contains("*b*"), "texte rendu : {}", doc.text);
        assert!(doc.text.contains("@c"), "texte rendu : {}", doc.text);
    }

    #[test]
    fn erreur_de_compilation_situee_par_ligne() {
        let (root, fonts) = dirs();
        let template = "#let data = json(\"data.json\")\n#text(data.name\n".to_string();
        let error = compile_template(template, br#"{"name": "x"}"#.to_vec(), root, fonts)
            .err()
            .expect("erreur attendue");
        assert!(error.contains("ligne 2"), "message : {error}");
    }
}
