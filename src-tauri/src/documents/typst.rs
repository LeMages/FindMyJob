//! Compilation d'un document Typst (.typ) en PDF : référence autonome, ou
//! template au contrat FindMyJob, qui lit ses données dans
//! `sys.inputs.data` (JSON) et retombe sur ses valeurs d'exemple sans elles.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine, TypstTemplateMainFile};
use typst_layout::PagedDocument;
use typst_library::diag::{SourceDiagnostic, Warned};
use typst_library::foundations::{Dict, IntoValue, Str};

// Importation des modules internes
use crate::documents::missing_fonts::{missing_font_families, missing_fonts_message};
use crate::documents::typst_diagnostics::{
    describe_compile_error, describe_warnings, located_diagnostics, LocatedDiagnostic,
};
use crate::documents::typst_headings::{extract_headings, HeadingSection};
use crate::documents::typst_lines::{extract_lines, TextLine};
use crate::documents::typst_metadata::findmyjob_metadata;
use crate::documents::typst_text::extract_text;
use crate::trace_log;

/// Clé de `sys.inputs` qui porte les données JSON d'un template.
pub const DATA_INPUT_KEY: &str = "data";

/// Nom sous lequel les données d'un template sont exposées au compilateur.
/// Un template les lit avec `json("data.json")`.
pub const TEMPLATE_DATA_FILE: &str = "data.json";

/// Résultat d'une compilation : le PDF, et ce qu'il faut pour le comparer à
/// une référence (nombre de pages, texte rendu, lignes positionnées).
///
/// # Attributes
///
/// * `pdf` - Octets du PDF généré.
/// * `page_count` - Nombre de pages du document.
/// * `text` - Texte rendu, dans l'ordre de mise en page.
/// * `lines` - Lignes de texte positionnées, de haut en bas.
/// * `warnings` - Avertissements non bloquants du compilateur, situés.
/// * `metadata` - Valeurs des métadonnées `<findmyjob>`, dans l'ordre du document.
/// * `headings` - Titres de section rendus, avec la présence d'un contenu après chacun.
pub struct CompiledDocument {
    pub pdf: Vec<u8>,
    pub page_count: usize,
    pub text: String,
    pub lines: Vec<TextLine>,
    pub warnings: Vec<String>,
    pub metadata: Vec<Result<serde_json::Value, String>>,
    pub headings: Vec<HeadingSection>,
}

/// Échec typé d'une compilation, pour la validation d'une référence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileFailure {
    /// Erreurs du compilateur, situées quand c'est possible.
    Source(Vec<LocatedDiagnostic>),
    /// Familles de polices introuvables (Typst se replierait en silence).
    MissingFonts(Vec<String>),
    /// Échec de l'export PDF ou interne.
    Other(String),
}

impl CompileFailure {
    /// Message lisible (français), pour les appelants qui n'ont besoin que
    /// d'un texte.
    pub fn message(&self) -> String {
        match self {
            CompileFailure::Source(diagnostics) => {
                let lines: Vec<String> = diagnostics
                    .iter()
                    .map(|d| match d.line {
                        Some(line) => format!("ligne {line} : {}", d.message),
                        None => d.message.clone(),
                    })
                    .collect();
                format!("Erreur de compilation Typst :\n{}", lines.join("\n"))
            }
            CompileFailure::MissingFonts(families) => missing_fonts_message(families),
            CompileFailure::Other(message) => message.clone(),
        }
    }
}

/// Moteur de compilation d'un template au contrat FindMyJob, construit une
/// fois (recherche des polices comprise) pour plusieurs rendus. Il n'a
/// aucun accès au disque : le template est un fichier unique.
pub struct ContractEngine {
    engine: TypstEngine<TypstTemplateMainFile>,
    template: String,
}

impl ContractEngine {
    /// Prépare le moteur d'un template.
    ///
    /// # Arguments
    ///
    /// * `template` - Source Typst du template.
    /// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
    pub fn new(template: &str, fonts_dir: &Path) -> Self {
        let engine =
            TypstEngine::builder().main_file(template.to_string()).search_fonts_with(font_options(fonts_dir)).build();
        Self { engine, template: template.to_string() }
    }

    /// Compile le template. Avec `data_json`, les données sont injectées
    /// dans `sys.inputs.data` (chaîne JSON, jamais évaluée comme du markup) ;
    /// sans, le template se rend avec ses valeurs d'exemple.
    ///
    /// # Arguments
    ///
    /// * `data_json` - Données sérialisées en JSON, ou `None` pour l'exemple.
    ///
    /// # Returns
    ///
    /// `CompiledDocument`, ou l'échec typé.
    pub fn compile(&self, data_json: Option<&str>) -> Result<CompiledDocument, CompileFailure> {
        let Warned { output, warnings: diagnostics } = match data_json {
            Some(data) => {
                let mut inputs = Dict::new();
                inputs.insert(Str::from(DATA_INPUT_KEY), data.into_value());
                self.engine.compile_with_input::<_, PagedDocument>(inputs)
            }
            None => self.engine.compile::<PagedDocument>(),
        };
        let doc = output.map_err(|e| CompileFailure::Source(located_diagnostics(&e, &self.template)))?;
        finish(doc, &diagnostics, &self.template)
    }
}

/// Compile une fois un template au contrat FindMyJob (voir `ContractEngine`).
///
/// # Arguments
///
/// * `template` - Source Typst du template.
/// * `data_json` - Données sérialisées en JSON, ou `None` pour l'exemple.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
///
/// # Returns
///
/// `CompiledDocument`, ou l'échec typé.
pub fn compile_contract(
    template: &str,
    data_json: Option<&str>,
    fonts_dir: &Path,
) -> Result<CompiledDocument, CompileFailure> {
    ContractEngine::new(template, fonts_dir).compile(data_json)
}

/// Options de polices : système, embarquées et importées.
fn font_options(fonts_dir: &Path) -> TypstKitFontOptions {
    let options = TypstKitFontOptions::default().include_system_fonts(true).include_embedded_fonts(true);
    if fonts_dir.is_dir() {
        options.include_dirs([fonts_dir.to_path_buf()])
    } else {
        options
    }
}

/// Après une compilation réussie : avertissements tracés, polices manquantes
/// refusées, export PDF et extraction du texte, des lignes, des titres et
/// des métadonnées.
fn finish(
    doc: PagedDocument,
    diagnostics: &[SourceDiagnostic],
    content: &str,
) -> Result<CompiledDocument, CompileFailure> {
    let warnings = describe_warnings(diagnostics, content);
    for warning in &warnings {
        trace_log::log(&format!("[typst] avertissement : {warning}"));
    }
    // Typst se replie en silence sur sa police par défaut : un document dans
    // une autre police que celle demandée ne doit jamais sortir.
    let missing = missing_font_families(diagnostics, content);
    if !missing.is_empty() {
        return Err(CompileFailure::MissingFonts(missing));
    }
    let pdf = typst_pdf::pdf(&doc, &Default::default())
        .map_err(|e| CompileFailure::Other(format!("Erreur de génération PDF : {:?}", e)))?;
    Ok(CompiledDocument {
        pdf,
        page_count: doc.pages().len(),
        text: extract_text(&doc),
        lines: extract_lines(&doc),
        warnings,
        metadata: findmyjob_metadata(&doc),
        headings: extract_headings(&doc),
    })
}

/// Compile une source Typst en PDF.
///
/// Une police introuvable est une erreur, jamais un repli silencieux.
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

/// Construit le moteur Typst, compile, puis exporte en PDF. Les
/// avertissements du compilateur sont tracés et renvoyés ; une police
/// introuvable fait échouer la compilation.
fn compile(
    content: String,
    data_json: Option<Vec<u8>>,
    root: PathBuf,
    fonts_dir: PathBuf,
) -> Result<CompiledDocument, String> {
    // Le fichier virtuel est ajouté avant le resolver disque : un éventuel
    // `data.json` présent dans `root` ne peut pas masquer les vraies données.
    let mut builder = TypstEngine::builder().main_file(content.as_str());
    if let Some(data) = data_json {
        builder = builder.with_static_file_resolver([(TEMPLATE_DATA_FILE, data)]);
    }
    let engine = builder.with_file_system_resolver(root).search_fonts_with(font_options(&fonts_dir)).build();

    let Warned { output, warnings: diagnostics } = engine.compile::<PagedDocument>();
    let doc = output
        .map_err(|e| format!("Erreur de compilation Typst :\n{}", describe_compile_error(&e, &content)))?;
    finish(doc, &diagnostics, &content).map_err(|failure| failure.message())
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

    /// Template autonome minimal : exemple par défaut, données injectées sinon.
    const CONTRACT: &str = "#let example = (name: \"Exemple\")
#let data = if \"data\" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }
#metadata((contract: \"findmyjob/cv\", version: 1, example: example)) <findmyjob>
#text(data.name)";

    #[test]
    fn compile_contract_seul_avec_l_exemple() {
        let doc = compile_contract(CONTRACT, None, &dirs().1).expect("compilation");
        assert!(doc.text.contains("Exemple"), "texte rendu : {}", doc.text);
        assert_eq!(doc.metadata.len(), 1);
        let metadata = doc.metadata[0].as_ref().expect("métadonnée convertible");
        assert_eq!(metadata["contract"], "findmyjob/cv");
        assert_eq!(metadata["version"], 1);
        assert_eq!(metadata["example"]["name"], "Exemple");
    }

    #[test]
    fn compile_contract_lit_sys_inputs_comme_du_texte() {
        let doc = compile_contract(CONTRACT, Some(r#"{"name": "A *b* @c #d"}"#), &dirs().1).expect("compilation");
        assert!(doc.text.contains("*b*"), "texte rendu : {}", doc.text);
        assert!(!doc.text.contains("Exemple"), "texte rendu : {}", doc.text);
    }

    #[test]
    fn compile_contract_erreur_situee() {
        let failure = compile_contract("#let x = 1\n#text(x.name)\n", None, &dirs().1).err().expect("erreur attendue");
        let CompileFailure::Source(diagnostics) = failure else { panic!("erreur de source attendue") };
        assert_eq!(diagnostics[0].line, Some(2), "{diagnostics:?}");
    }

    #[test]
    fn compile_contract_sans_acces_disque() {
        let failure = compile_contract("#read(\"Cargo.toml\")", None, &dirs().1).err().expect("erreur attendue");
        assert!(matches!(failure, CompileFailure::Source(_)), "{failure:?}");
    }

    #[test]
    fn titres_de_section_et_contenu() {
        let source = "= Vide\n= Plein\nTexte\n= Dernier";
        let doc = compile_contract(source, None, &dirs().1).expect("compilation");
        let sections: Vec<(&str, bool)> = doc.headings.iter().map(|h| (h.title.as_str(), h.has_content)).collect();
        assert_eq!(sections, vec![("Vide", false), ("Plein", true), ("Dernier", false)]);
    }
}
