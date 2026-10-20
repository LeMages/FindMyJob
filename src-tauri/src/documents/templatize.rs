//! Templatisation d'un document de référence, en deux étapes :
//!
//! - lecture déterministe du balisage (`documents::markup`), avant tout
//!   appel au LLM ;
//! - étape A : le LLM range le contenu dans le schéma canonique
//!   (`llm::templatize_example`), validé par `example_check` ; Rust en tire
//!   les consignes rattachées et les textes en dur de la référence ;
//! - étape B : le LLM écrit le template en texte libre
//!   (`llm::templatize_template`), validé par compilation
//!   (`template_check`) ; les corrections portent sur des extraits quand le
//!   modèle le choisit, et le meilleur essai est toujours conservé.
//!
//! Le modèle de cette étape est imposé (`TEMPLATIZE_MODEL`), indépendamment
//! du modèle choisi dans les paramètres : la templatisation exige un modèle
//! fort et n'a lieu qu'une fois par référence.

// Importation des modules standard
use std::path::Path;
use std::sync::Arc;

// Importation des modules internes
use crate::documents::example_check::{check_example, CheckedExample, ExampleContext};
use crate::documents::markup::{describe_markup_errors, parse_markup, ReferenceMarkup};
use crate::documents::template_check::{check_template, TemplateContext, TemplateVerdict};
use crate::documents::template_edits::apply_edits;
use crate::documents::templatize_debug::TemplatizeDebug;
use crate::documents::typst::CompiledDocument;
use crate::error::AppError;
use crate::llm::client::TEMPLATIZE_MODEL;
use crate::llm::templatize_example::{extract_example_data, ExampleFeedback};
use crate::llm::templatize_template::{write_template, TemplateFeedback, TemplateReply, TemplateRequest};
use crate::llm::{CallRecorder, ClaudeTarget};
use crate::models::{ReferenceTexts, TemplateArtifacts, REFERENCE_FORMAT_VERSION, REFERENCE_TEXTS_FILE};
use crate::storage_paths;
use crate::trace_log;

/// Appels autorisés pour les données d'exemple : un essai + un retry.
const MAX_EXAMPLE_ATTEMPTS: u8 = 2;
/// Appels autorisés pour le template : un essai + deux corrections.
const MAX_TEMPLATE_ATTEMPTS: u8 = 3;
/// Problèmes techniques cités au plus dans un message d'échec.
const MAX_PROBLEMS_IN_ERROR: usize = 3;

/// Nom du template écrit à côté de `reference.typ`.
pub const TEMPLATE_FILE: &str = "template.typ";
/// Nom des données d'exemple écrites à côté de `reference.typ`.
pub const EXAMPLE_DATA_FILE: &str = "example_data.json";

/// Template validé, prêt à être écrit sur disque.
///
/// # Attributes
///
/// * `template_typ` - Source Typst du template.
/// * `example_data_json` - Données d'exemple (textes en dur compris), JSON indenté.
/// * `reference_texts_json` - Textes et consignes de la référence (`ReferenceTexts`), JSON indenté.
/// * `page_count` - Nombre de pages du template rendu avec ses données
///   d'exemple : plafond de pages des documents générés.
/// * `example_attempts`, `template_attempts` - Appels LLM de chaque étape.
pub struct ValidatedTemplate {
    pub template_typ: String,
    pub example_data_json: String,
    pub reference_texts_json: String,
    pub page_count: usize,
    pub example_attempts: u8,
    pub template_attempts: u8,
}

/// Référence à templatiser, déjà compilée.
///
/// # Attributes
///
/// * `source` - Source Typst de la référence.
/// * `rendered` - Rendu de la référence (pages, texte, lignes).
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `root` - Dossier racine de compilation (fichiers annexes de la référence).
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
pub struct TemplatizeInput<'a> {
    pub source: &'a str,
    pub rendered: &'a CompiledDocument,
    pub doc_type: &'a str,
    pub root: &'a Path,
    pub fonts_dir: &'a Path,
}

/// Convertit une référence en template validé.
///
/// # Arguments
///
/// * `input` - Référence compilée et dossiers.
/// * `target` - Clé API Claude (le modèle est remplacé par `TEMPLATIZE_MODEL`).
/// * `debug_dir` - Dossier où conserver la référence, chaque essai et ses
///   appels à Claude (mode DEV uniquement, `None` en PROD).
///
/// # Returns
///
/// `ValidatedTemplate`, ou `AppError::Validation` avec un message lisible.
pub async fn templatize(
    input: &TemplatizeInput<'_>,
    target: &ClaudeTarget,
    debug_dir: Option<&Path>,
) -> Result<ValidatedTemplate, AppError> {
    // Le balisage est lu avant tout appel au LLM : une erreur de balisage
    // est à corriger par l'utilisateur, inutile de payer une templatisation.
    let markup = parse_markup(input.source).map_err(|errors| AppError::Validation(describe_markup_errors(&errors)))?;
    let session = Session {
        input,
        markup,
        computes_date: input.source.contains("datetime.today"),
        debug: debug_dir.map(|dir| TemplatizeDebug::start(dir, input.source)),
        recorder: Arc::new(CallRecorder::default()),
    };
    let target = target.with_model(TEMPLATIZE_MODEL).with_recorder(Arc::clone(&session.recorder));

    let (mut checked, example_attempts) = session.example_data(&target).await?;
    let (template, verdict, template_attempts) = session.template(&mut checked, &target).await?;
    let page_count = verdict.page_count.ok_or_else(|| {
        AppError::Validation("Le template accepté ne compile pas avec ses données d'exemple.".to_string())
    })?;
    let texts = ReferenceTexts {
        format_version: REFERENCE_FORMAT_VERSION,
        lang: checked.lang.clone(),
        texts: checked.texts.clone(),
        guidance: checked.guidance.clone(),
    };
    Ok(ValidatedTemplate {
        template_typ: template,
        example_data_json: pretty(&checked.example)?,
        reference_texts_json: pretty(&texts)?,
        page_count,
        example_attempts,
        template_attempts,
    })
}

/// Une templatisation en cours.
struct Session<'a> {
    input: &'a TemplatizeInput<'a>,
    markup: ReferenceMarkup,
    computes_date: bool,
    debug: Option<TemplatizeDebug>,
    recorder: Arc<CallRecorder>,
}

/// Meilleur template obtenu jusqu'ici, avec son verdict.
struct Best {
    template: String,
    verdict: TemplateVerdict,
}

impl Session<'_> {
    /// Étape A : données d'exemple validées.
    async fn example_data(&self, target: &ClaudeTarget) -> Result<(CheckedExample, u8), AppError> {
        let context = ExampleContext {
            doc_type: self.input.doc_type,
            reference_text: &self.input.rendered.text,
            computes_date: self.computes_date,
            markup: &self.markup,
        };
        let mut feedback: Option<ExampleFeedback> = None;
        for attempt in 1..=MAX_EXAMPLE_ATTEMPTS {
            let raw = extract_example_data(self.input.source, self.input.doc_type, feedback.as_ref(), target).await?;
            let json = pretty(&raw)?;
            match check_example(&raw, &context) {
                Ok(checked) => {
                    self.save(&format!("example_{attempt}"), &[("example_data.json", &json)], &[]);
                    return Ok((checked, attempt));
                }
                Err(problems) => {
                    self.log_rejected("données d'exemple", attempt, MAX_EXAMPLE_ATTEMPTS, &problems);
                    self.save(&format!("example_{attempt}"), &[("example_data.json", &json)], &problems);
                    feedback = Some(ExampleFeedback { previous: json, problems });
                }
            }
        }
        let problems = feedback.map(|f| f.problems).unwrap_or_default();
        Err(AppError::Validation(example_failure_message(&problems)))
    }

    /// Étape B : template validé, en gardant toujours le meilleur essai.
    async fn template(
        &self,
        checked: &mut CheckedExample,
        target: &ClaudeTarget,
    ) -> Result<(String, TemplateVerdict, u8), AppError> {
        let context = TemplateContext {
            reference: self.input.rendered,
            computes_date: self.computes_date,
            doc_type: self.input.doc_type,
            root: self.input.root,
            fonts_dir: self.input.fonts_dir,
        };
        let mut best: Option<Best> = None;
        let mut pending_problems: Vec<String> = Vec::new();
        for attempt in 1..=MAX_TEMPLATE_ATTEMPTS {
            let example_json = pretty(&checked.example)?;
            let request = TemplateRequest {
                reference_typ: self.input.source,
                doc_type: self.input.doc_type,
                example_data_json: &example_json,
                texts: &checked.texts,
            };
            let feedback = best.as_ref().map(|b| TemplateFeedback {
                previous_template: b.template.clone(),
                problems: [pending_problems.clone(), b.verdict.problems.clone()].concat(),
            });
            let reply = write_template(&request, feedback.as_ref(), target).await?;
            let template = match resolve_reply(reply, best.as_ref()) {
                Ok(template) => template,
                Err(problems) => {
                    // Remplacements inapplicables : l'essai ne produit rien,
                    // le meilleur template reste celui à corriger.
                    self.log_rejected("template", attempt, MAX_TEMPLATE_ATTEMPTS, &problems);
                    self.save(&format!("template_{attempt}"), &[], &problems);
                    pending_problems = problems;
                    continue;
                }
            };
            pending_problems.clear();
            let verdict = check_template(&template, checked, &context).await?;
            self.save(&format!("template_{attempt}"), &[("template.typ", &template)], &verdict.problems);
            if verdict.problems.is_empty() {
                return Ok((template, verdict, attempt));
            }
            self.log_rejected("template", attempt, MAX_TEMPLATE_ATTEMPTS, &verdict.problems);
            adopt_new_texts(checked, &verdict);
            if best.as_ref().is_none_or(|b| verdict.score() < b.verdict.score()) {
                best = Some(Best { template, verdict });
            }
        }
        match best {
            // Seuls des séparateurs orphelins : un défaut d'affichage limité
            // aux champs vides du profil ne doit pas faire échouer l'import.
            Some(best) if best.verdict.blocking == 0 && best.verdict.page_count.is_some() => {
                trace_log::log(&format!(
                    "[templatize] {} : template accepté malgré des séparateurs orphelins : {:?}",
                    self.input.doc_type, best.verdict.problems
                ));
                Ok((best.template, best.verdict, MAX_TEMPLATE_ATTEMPTS))
            }
            best => {
                let problems = best.map(|b| b.verdict.problems).unwrap_or(pending_problems);
                Err(AppError::Validation(template_failure_message(&problems)))
            }
        }
    }

    /// Trace un essai refusé.
    fn log_rejected(&self, step: &str, attempt: u8, max: u8, problems: &[String]) {
        trace_log::log(&format!(
            "[templatize] {} : {step}, essai {attempt}/{max} refusé : {problems:?}",
            self.input.doc_type
        ));
    }

    /// Conserve un essai et ses appels dans le dossier de debug.
    fn save(&self, prefix: &str, files: &[(&str, &str)], problems: &[String]) {
        let calls = self.recorder.drain();
        if let Some(debug) = &self.debug {
            debug.save_attempt(prefix, files, problems, &calls);
        }
    }
}

/// Template complet, ou template précédent corrigé par les remplacements.
fn resolve_reply(reply: TemplateReply, best: Option<&Best>) -> Result<String, Vec<String>> {
    match (reply, best) {
        (TemplateReply::Full(template), _) => Ok(template),
        (TemplateReply::Edits(edits), Some(best)) => apply_edits(&best.template, &edits),
        (TemplateReply::Edits(_), None) => {
            Err(vec!["Aucun template à corriger : renvoie le template complet dans un bloc ```typst```.".to_string()])
        }
    }
}

/// Reprend les textes en dur découverts dans le rendu sentinelle, pour que
/// l'essai suivant puisse les lire dans `data.texts`.
fn adopt_new_texts(checked: &mut CheckedExample, verdict: &TemplateVerdict) {
    if verdict.texts == checked.texts {
        return;
    }
    checked.texts = verdict.texts.clone();
    if let (Some(map), Ok(texts)) = (checked.example.as_object_mut(), serde_json::to_value(&checked.texts)) {
        map.insert("texts".to_string(), texts);
    }
}

/// Message d'échec de l'étape A : ce qui s'est passé et quoi faire. Les
/// erreurs de balisage sont citées (l'utilisateur peut les corriger) ; le
/// détail technique reste dans les traces.
fn example_failure_message(problems: &[String]) -> String {
    let markup: Vec<&String> = problems.iter().filter(|p| p.starts_with("Balisage de la référence")).collect();
    if !markup.is_empty() {
        let lines: Vec<String> = markup.iter().map(|p| format!("- {p}")).collect();
        return format!(
            "Le balisage de la référence ne correspond pas à son texte. Corrigez-le dans le fichier .typ puis \
             réimportez-le :\n{}",
            lines.join("\n")
        );
    }
    format!(
        "Le contenu de la référence n'a pas pu être lu de façon fiable. Réessayez l'import ; si l'échec se répète, \
         simplifiez la mise en page du document. Détail : {}",
        summary(problems)
    )
}

/// Message d'échec de l'étape B.
fn template_failure_message(problems: &[String]) -> String {
    format!(
        "La mise en page de la référence n'a pas pu être reproduite exactement après {MAX_TEMPLATE_ATTEMPTS} essais. \
         Réessayez l'import ; si l'échec se répète, simplifiez la mise en page (moins de fonctions et de logique \
         Typst). Détail : {}",
        summary(problems)
    )
}

/// Premiers problèmes, sur une ligne chacun.
fn summary(problems: &[String]) -> String {
    let shown: Vec<String> = problems
        .iter()
        .take(MAX_PROBLEMS_IN_ERROR)
        .map(|p| p.lines().next().unwrap_or_default().to_string())
        .collect();
    let more = problems.len().saturating_sub(MAX_PROBLEMS_IN_ERROR);
    if more > 0 {
        format!("{} (et {more} autre(s))", shown.join(" ; "))
    } else {
        shown.join(" ; ")
    }
}

/// Sérialise en JSON indenté.
fn pretty<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string_pretty(value).map_err(|e| AppError::Validation(format!("JSON non sérialisable : {e}")))
}

/// Fichiers de templatisation d'une version : chemins relatifs dans son
/// dossier et nombre de pages, calculés avant toute écriture.
///
/// # Arguments
///
/// * `dir` - Dossier relatif de la version.
/// * `validated` - Template validé.
///
/// # Returns
///
/// `TemplateArtifacts` des fichiers à écrire par `write_template_files`.
pub fn template_artifacts(dir: &str, validated: &ValidatedTemplate) -> Result<TemplateArtifacts, AppError> {
    let page_count = i32::try_from(validated.page_count)
        .map_err(|_| AppError::Validation(format!("Nombre de pages hors limites : {}", validated.page_count)))?;
    Ok(TemplateArtifacts {
        template_path: storage_paths::file_in(dir, TEMPLATE_FILE),
        example_data_path: storage_paths::file_in(dir, EXAMPLE_DATA_FILE),
        page_count,
    })
}

/// Écrit `template.typ`, `example_data.json` et `reference_texts.json` dans
/// le dossier de la version.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `dir` - Dossier relatif de la version.
/// * `artifacts` - Emplacements calculés par `template_artifacts`.
/// * `validated` - Template validé.
///
/// # Returns
///
/// `Ok(())` une fois les trois fichiers écrits.
pub fn write_template_files(
    data_dir: &Path,
    dir: &str,
    artifacts: &TemplateArtifacts,
    validated: &ValidatedTemplate,
) -> Result<(), AppError> {
    let texts_path = storage_paths::file_in(dir, REFERENCE_TEXTS_FILE);
    let files = [
        (artifacts.template_path.as_str(), &validated.template_typ),
        (artifacts.example_data_path.as_str(), &validated.example_data_json),
        (texts_path.as_str(), &validated.reference_texts_json),
    ];
    for (relative, content) in files {
        let path = storage_paths::to_absolute(data_dir, relative);
        std::fs::write(&path, content.as_bytes())
            .map_err(|e| AppError::Validation(format!("Écriture de {} impossible : {e}", path.display())))?;
    }
    Ok(())
}
