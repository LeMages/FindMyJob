//! Templatisation d'un document de référence : appel LLM
//! (`llm::templatize`), validation du template obtenu par compilation, et
//! retry avec les problèmes relevés en feedback.
//!
//! La référence est la source de vérité de la structure et du texte : rien
//! n'en est retiré, un texte sans champ dans le schéma reste écrit en dur
//! dans le template. Un template est accepté s'il :
//! - lit ses données depuis `data.json` et n'évalue ni ne calcule rien
//!   (`eval`, `datetime.today`, `read` interdits) ;
//! - compilé avec ses données d'exemple, n'a pas plus de pages que la
//!   référence, et rend exactement son texte (seule une date calculée à la
//!   compilation peut différer) ;
//! - compile avec des données vides ;
//! - compilé avec les champs secondaires vides (coordonnées, organisations,
//!   lieux, niveaux), ne laisse aucun séparateur orphelin ;
//! - compilé avec un marqueur unique dans chaque champ (rendu sentinelle),
//!   ne garde en dur aucune valeur qui a un champ (nom, ville, entreprise,
//!   résumé…) : elle ne serait jamais remplacée par celle du profil.
//!
//! Les consignes extraites de la référence (`guidance`) doivent redonner
//! exactement la prose des données d'exemple.

// Importation des modules standard
use std::path::Path;

// Importation des modules internes
use crate::documents::sparse_check::{cleared_data, sparse_problems, SPARSE_FIELDS};
use crate::documents::typst::{compile_template, CompiledDocument};
use crate::storage_paths;
use crate::docgen::date_format::format_long_date;
use crate::docgen::example_facts::{
    cv_information_values, cv_prose_values, lm_information_values, lm_prose_values,
};
use crate::docgen::guidance_consistency::guidance_problems;
use crate::docgen::sentinel_data::{hardcoded_values, is_month_word, sentinel_cv_data, sentinel_lm_data};
use crate::docgen::text_diff::diff_words;
use crate::error::AppError;
use crate::llm::{templatize_reference, TemplatizeFeedback, TemplatizeOutput};
use crate::llm::ClaudeTarget;
use crate::models::{CvData, CvExampleData, LmData, LmExampleData, ProseField, TemplateArtifacts};
use crate::trace_log;

/// Nombre total d'appels LLM autorisés : un essai + 2 retries avec feedback.
const MAX_ATTEMPTS: u8 = 3;
/// Phrase ajoutée au feedback quand une tentative reproduit exactement les
/// problèmes d'une tentative précédente.
const OSCILLATION_HINT: &str = "Cette tentative reproduit exactement les problèmes d'une tentative précédente : \
     tu oscilles entre deux solutions qui échouent. Change d'approche au lieu de revenir à une sortie déjà refusée.";
/// Nombre de mots en écart cités dans un message de feedback.
const MAX_WORDS_REPORTED: usize = 30;
/// Constructions interdites dans un template : elles évalueraient une valeur
/// comme du code ou rendraient le document dépendant de la date de compilation.
const FORBIDDEN_CONSTRUCTS: [&str; 3] = ["eval(", "datetime.today", "read("];

/// Langues de document prises en charge (`lang`, `detected_lang`).
const SUPPORTED_LANGS: [&str; 2] = ["fr", "en"];

/// Nom du template écrit à côté de `reference.typ`.
pub const TEMPLATE_FILE: &str = "template.typ";
/// Nom des données d'exemple écrites à côté de `reference.typ`.
pub const EXAMPLE_DATA_FILE: &str = "example_data.json";

/// Template validé, prêt à être écrit sur disque.
///
/// # Attributes
///
/// * `template_typ` - Source Typst du template.
/// * `example_data_json` - Données d'exemple, JSON indenté.
/// * `page_count` - Nombre de pages du template rendu avec ses données
///   d'exemple : plafond de pages des documents générés.
/// * `attempts` - Nombre d'appels LLM qu'il a fallu.
pub struct ValidatedTemplate {
    pub template_typ: String,
    pub example_data_json: String,
    pub page_count: usize,
    pub attempts: u8,
}

/// Issue de la validation d'une tentative.
enum Validation {
    /// Template accepté ; nombre de pages du rendu avec les données d'exemple.
    Accepted { page_count: usize },
    /// Template refusé ; problèmes à renvoyer en feedback. `tolerable` vaut
    /// le nombre de pages quand les seuls problèmes sont des séparateurs
    /// orphelins : le template reste utilisable en dernier recours.
    Rejected { problems: Vec<String>, tolerable: Option<usize> },
}

/// Convertit une référence en template validé, avec jusqu'à 2 retries.
///
/// # Arguments
///
/// * `reference_typ` - Source Typst de la référence.
/// * `reference` - Rendu de la référence (nombre de pages, texte), déjà compilé par l'appelant.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `root` - Dossier racine de compilation (fichiers annexes de la référence).
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// `ValidatedTemplate`, ou `AppError::Validation` listant les problèmes de
/// la dernière tentative si aucune ne passe la validation.
pub async fn templatize(
    reference_typ: &str,
    reference: &CompiledDocument,
    doc_type: &str,
    root: &Path,
    fonts_dir: &Path,
    target: &ClaudeTarget,
) -> Result<ValidatedTemplate, AppError> {
    let mut feedback: Option<TemplatizeFeedback> = None;
    let mut previous_problems: Vec<Vec<String>> = Vec::new();
    let computes_date = reference_typ.contains("datetime.today");

    for attempt in 1..=MAX_ATTEMPTS {
        let output = templatize_reference(reference_typ, doc_type, feedback.as_ref(), target).await?;
        let example_data_json = serde_json::to_string_pretty(&output.example_data)
            .map_err(|e| AppError::Llm(format!("Données d'exemple non sérialisables : {e}")))?;

        let context = ValidationContext { reference, computes_date, doc_type, root, fonts_dir };
        let problems = match validate(&output, &example_data_json, &context).await? {
            Validation::Accepted { page_count } => {
                return Ok(ValidatedTemplate {
                    template_typ: output.template_typ,
                    example_data_json,
                    page_count,
                    attempts: attempt,
                });
            }
            // Dernière tentative et seulement des séparateurs orphelins : un
            // défaut d'affichage limité aux champs vides du profil ne doit
            // pas faire échouer l'import.
            Validation::Rejected { problems, tolerable: Some(page_count) } if attempt == MAX_ATTEMPTS => {
                trace_log::log(&format!(
                    "[templatize] {doc_type} : template accepté malgré des séparateurs orphelins : {problems:?}"
                ));
                return Ok(ValidatedTemplate {
                    template_typ: output.template_typ,
                    example_data_json,
                    page_count,
                    attempts: attempt,
                });
            }
            Validation::Rejected { problems, .. } => problems,
        };
        let problems = flag_oscillation(problems, &mut previous_problems);

        trace_log::log(&format!("[templatize] {doc_type} : tentative {attempt}/{MAX_ATTEMPTS} rejetée : {problems:?}"));
        feedback = Some(TemplatizeFeedback {
            previous_template: output.template_typ,
            previous_example_data: example_data_json,
            problems,
        });
    }

    let problems = feedback.map(|f| f.problems.join("\n- ")).unwrap_or_default();
    Err(AppError::Validation(format!(
        "La conversion du document de référence en template a échoué après {MAX_ATTEMPTS} tentatives. \
         Derniers problèmes relevés :\n- {problems}"
    )))
}

/// Ce que la validation doit savoir de la référence et de la compilation.
///
/// # Attributes
///
/// * `reference` - Rendu de la référence.
/// * `computes_date` - Vrai si la référence calcule sa date (`datetime.today`).
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `root` - Dossier racine de compilation.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
struct ValidationContext<'a> {
    reference: &'a CompiledDocument,
    computes_date: bool,
    doc_type: &'a str,
    root: &'a Path,
    fonts_dir: &'a Path,
}

/// Valide une sortie de templatisation.
async fn validate(
    output: &TemplatizeOutput,
    example_data_json: &str,
    context: &ValidationContext<'_>,
) -> Result<Validation, AppError> {
    // ------------------------------------------------------------------
    // 1. Contrôles statiques du template et typage des données d'exemple
    // ------------------------------------------------------------------
    let mut problems = static_problems(&output.template_typ);
    let checked = match check_example(&output.example_data, context.doc_type) {
        Ok(checked) => checked,
        Err(problem) => {
            problems.push(problem);
            return Ok(Validation::Rejected { problems, tolerable: None });
        }
    };
    problems.extend(checked.guidance_problems);

    // ------------------------------------------------------------------
    // 2. Rendus avec les données d'exemple, vides, clairsemées et
    //    sentinelles (compilation bloquante, hors du runtime async)
    // ------------------------------------------------------------------
    let template = output.template_typ.clone();
    let example = example_for_rendering(&output.example_data, example_data_json, context)?;
    let sparse = cleared_data(&example, &SPARSE_FIELDS)?;
    let sentinel = sentinel_data(&output.example_data, context.doc_type)?;
    let data_sets = [example.clone(), checked.blank_json.into_bytes(), sparse, sentinel];
    let (root, fonts_dir) = (context.root.to_path_buf(), context.fonts_dir.to_path_buf());
    let [with_example, with_blank, with_sparse, with_sentinel] = tokio::task::spawn_blocking(move || {
        data_sets.map(|data| compile_template(template.clone(), data, root.clone(), fonts_dir.clone()))
    })
    .await
    .map_err(|e| AppError::Validation(format!("Erreur interne de compilation Typst : {e}")))?;

    // ------------------------------------------------------------------
    // 3. Comparaison avec la référence, puis recherche de contenu en dur
    //    et de séparateurs orphelins
    // ------------------------------------------------------------------
    let page_count = match with_example {
        Ok(rendered) => {
            problems.extend(compare_with_reference(context, &rendered));
            Some(rendered.page_count)
        }
        Err(errors) => {
            problems.push(format!("Le template ne compile pas avec example_data : {errors}"));
            None
        }
    };
    problems.extend(blank_problem(with_blank));
    problems.extend(sentinel_problem(with_sentinel, &checked.values));
    let blocking = problems.len();
    let (root, fonts_dir) = (context.root.to_path_buf(), context.fonts_dir.to_path_buf());
    problems.extend(sparse_problems(with_sparse, &output.template_typ, &example, root, fonts_dir).await?);

    Ok(match page_count {
        Some(page_count) if problems.is_empty() => Validation::Accepted { page_count },
        // Seuls des séparateurs orphelins : template utilisable en dernier recours.
        Some(page_count) if blocking == 0 => Validation::Rejected { problems, tolerable: Some(page_count) },
        _ => Validation::Rejected { problems, tolerable: None },
    })
}

/// Ajoute `OSCILLATION_HINT` aux problèmes d'une tentative s'ils sont
/// exactement ceux d'une tentative précédente, puis les mémorise.
///
/// # Arguments
///
/// * `problems` - Problèmes de la tentative courante.
/// * `previous` - Ensembles (triés) des problèmes des tentatives précédentes.
///
/// # Returns
///
/// Problèmes à renvoyer en feedback.
fn flag_oscillation(mut problems: Vec<String>, previous: &mut Vec<Vec<String>>) -> Vec<String> {
    let mut sorted = problems.clone();
    sorted.sort_unstable();
    if previous.contains(&sorted) {
        problems.push(OSCILLATION_HINT.to_string());
    }
    previous.push(sorted);
    problems
}

/// Problème du rendu avec des données vides : le template doit compiler.
/// Le texte qu'il garde en dur est légitime (texte de la référence sans
/// champ).
fn blank_problem(rendered: Result<CompiledDocument, String>) -> Option<String> {
    rendered.err().map(|errors| {
        format!("Le template ne compile pas quand tous les champs sont vides (chaînes vides, listes vides) : {errors}")
    })
}

/// Problème du rendu sentinelle : une valeur qui a un champ dans le schéma
/// (`values`) est restée écrite en dur, même dans une branche conditionnée
/// à un champ rempli.
fn sentinel_problem(rendered: Result<CompiledDocument, String>, values: &[String]) -> Option<String> {
    let rendered = match rendered {
        Ok(rendered) => rendered,
        Err(errors) => {
            return Some(format!(
                "Le template ne compile pas quand chaque champ contient une valeur et chaque liste un seul élément : \
                 {errors}"
            ))
        }
    };
    let hardcoded = hardcoded_values(&rendered.text, values);
    (!hardcoded.is_empty()).then(|| {
        format!(
            "Rendu avec un marqueur unique dans chaque champ de data, le template affiche encore ces valeurs de \
             example_data, donc écrites en dur : {}. Une valeur qui a un champ dans le schéma vient toujours de \
             data, même au milieu d'un texte recopié, sinon elle ne serait jamais remplacée par celle du profil.",
            quote_words(&hardcoded)
        )
    })
}

/// Données sentinelles du type de document, dans la langue des données
/// d'exemple (déjà vérifiée par `blank_data_json`).
fn sentinel_data(example_data: &serde_json::Value, doc_type: &str) -> Result<Vec<u8>, AppError> {
    let lang = example_data.get("lang").and_then(|v| v.as_str()).unwrap_or("fr");
    let data = match doc_type {
        "cv" => sentinel_cv_data(lang),
        _ => sentinel_lm_data(lang),
    };
    data.and_then(|value| serde_json::to_vec(&value))
        .map_err(|e| AppError::Validation(format!("Données sentinelles non sérialisables : {e}")))
}

/// Contrôles statiques : chargement de `data.json` et constructions interdites.
fn static_problems(template: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !template.contains("json(\"data.json\")") {
        problems.push("Le template doit charger ses données avec `json(\"data.json\")`.".to_string());
    }
    for construct in FORBIDDEN_CONSTRUCTS {
        if template.contains(construct) {
            problems.push(format!("Le template utilise `{construct}`, interdit."));
        }
    }
    problems
}

/// Données d'exemple vérifiées, et ce que la validation en tire.
///
/// # Attributes
///
/// * `blank_json` - Données vides (même langue, listes et chaînes vides).
/// * `values` - Valeurs qui ont un champ (informations et prose), qu'aucun
///   texte en dur du template ne doit reproduire.
/// * `guidance_problems` - Incohérences des consignes avec la prose.
struct CheckedExample {
    blank_json: String,
    values: Vec<String>,
    guidance_problems: Vec<String>,
}

/// Vérifie que les données d'exemple respectent le schéma canonique typé
/// et que leurs consignes redonnent leur prose, puis prépare les données
/// vides et les valeurs à ne pas trouver en dur.
///
/// # Returns
///
/// `CheckedExample`, ou un problème à renvoyer en feedback.
fn check_example(example_data: &serde_json::Value, doc_type: &str) -> Result<CheckedExample, String> {
    if let Some(raw) = example_data.get("guidance").and_then(|g| g.as_str()) {
        return Err(format!("guidance n'est pas un objet JSON valide : {raw}"));
    }
    let to_json = |value: Result<String, serde_json::Error>| value.map_err(|e| format!("Données vides non sérialisables : {e}"));
    match doc_type {
        "cv" => {
            let example: CvExampleData = serde_json::from_value(example_data.clone())
                .map_err(|e| format!("example_data ne respecte pas le schéma du CV : {e}"))?;
            check_langs(&example.data.lang, &example.detected_lang)?;
            if example.data.labels.months_short.len() != 12 {
                return Err(format!(
                    "labels.months_short doit contenir 12 abréviations, il en contient {}.",
                    example.data.labels.months_short.len()
                ));
            }
            let mut blank = CvData { lang: example.data.lang.clone(), ..Default::default() };
            blank.labels.months_short = vec![String::new(); 12];
            let prose = [("identity.title", &example.data.identity.title), ("summary", &example.data.summary)];
            Ok(CheckedExample {
                blank_json: to_json(serde_json::to_string(&blank))?,
                values: [cv_information_values(&example.data), cv_prose_values(&example.data)].concat(),
                guidance_problems: guidance_problems(&example.guidance, &prose_fields(&prose)),
            })
        }
        _ => {
            let example: LmExampleData = serde_json::from_value(example_data.clone())
                .map_err(|e| format!("example_data ne respecte pas le schéma de la lettre : {e}"))?;
            check_langs(&example.data.lang, &example.detected_lang)?;
            let blank = LmData { lang: example.data.lang.clone(), ..Default::default() };
            let paragraphs: Vec<(String, &String)> =
                example.data.paragraphs.iter().enumerate().map(|(i, p)| (format!("paragraphs[{i}]"), p)).collect();
            let mut fields = prose_fields(&[("subject", &example.data.subject)]);
            fields.extend(prose_fields(&paragraphs.iter().map(|(path, p)| (path.as_str(), *p)).collect::<Vec<_>>()));
            Ok(CheckedExample {
                blank_json: to_json(serde_json::to_string(&blank))?,
                values: [lm_information_values(&example.data), lm_prose_values(&example.data)].concat(),
                guidance_problems: guidance_problems(&example.guidance, &fields),
            })
        }
    }
}

/// Champs de prose des données d'exemple, aux chemins de `guidance`.
fn prose_fields(fields: &[(&str, &String)]) -> Vec<ProseField> {
    fields.iter().map(|(path, text)| ProseField { path: path.to_string(), text: text.to_string() }).collect()
}

/// Vérifie que `lang` et `detected_lang` sont des langues prises en charge
/// (le schéma les laisse en chaîne libre, voir `llm::doc_data_schema`).
fn check_langs(lang: &str, detected_lang: &str) -> Result<(), String> {
    for (field, value) in [("lang", lang), ("detected_lang", detected_lang)] {
        if !SUPPORTED_LANGS.contains(&value) {
            return Err(format!("{field} vaut \"{value}\", attendu l'une de : {}.", SUPPORTED_LANGS.join(", ")));
        }
    }
    Ok(())
}

/// Compare le rendu du template avec celui de la référence : pas plus de
/// pages, et exactement les mêmes mots (seule une date calculée par la
/// référence peut différer). Rien n'est retiré de la référence.
fn compare_with_reference(context: &ValidationContext<'_>, rendered: &CompiledDocument) -> Vec<String> {
    let mut problems = Vec::new();
    if rendered.page_count > context.reference.page_count {
        problems.push(format!(
            "Rendu avec example_data, le template fait {} page(s), plus que la référence ({}).",
            rendered.page_count, context.reference.page_count
        ));
    }

    let diff = match diff_words(&context.reference.text, &rendered.text) {
        Ok(diff) => diff,
        Err(e) => {
            problems.push(e);
            return problems;
        }
    };
    let tolerated = |word: &&String| !(context.computes_date && is_date_word(word));
    let missing: Vec<String> = diff.missing.iter().filter(tolerated).cloned().collect();
    let extra: Vec<String> = diff.extra.iter().filter(tolerated).cloned().collect();
    if !missing.is_empty() || !extra.is_empty() {
        problems.push(describe_text_diff(&missing, &extra));
    }
    problems
}

/// Données servant au rendu de validation : les données d'exemple telles
/// quelles, sauf pour une lettre dont la référence calcule sa date. Les
/// données d'exemple n'ont alors pas de date, et le template masquerait
/// toute la ligne de date : Rust y place la date du jour, comme il le
/// fera à la génération.
fn example_for_rendering(
    example_data: &serde_json::Value,
    example_data_json: &str,
    context: &ValidationContext<'_>,
) -> Result<Vec<u8>, AppError> {
    let date_is_empty = example_data.get("date").and_then(|v| v.as_str()).is_some_and(str::is_empty);
    if !(context.computes_date && date_is_empty) {
        return Ok(example_data_json.as_bytes().to_vec());
    }
    let lang = example_data.get("lang").and_then(|v| v.as_str()).unwrap_or("fr");
    let mut with_date = example_data.clone();
    with_date["date"] = serde_json::Value::String(format_long_date(chrono::Local::now().date_naive(), lang));
    serde_json::to_vec(&with_date).map_err(|e| AppError::Validation(format!("Données d'exemple non sérialisables : {e}")))
}

/// Indique si un mot peut appartenir à une date calculée à la compilation :
/// nombre (jour, année) ou nom de mois, entier ou abrégé.
fn is_date_word(word: &str) -> bool {
    let cleaned = word.trim_matches(|c: char| c == '.' || c == ',');
    if !cleaned.is_empty() && cleaned.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    is_month_word(word)
}

/// Formule un écart de texte pour le feedback.
fn describe_text_diff(missing: &[String], extra: &[String]) -> String {
    format!(
        "Rendu avec example_data, le texte diffère de la référence. Rien de la référence n'est retiré : un texte \
         sans champ dans le schéma reste écrit en dur dans le template. \
         Mots de la référence absents du rendu : {}. \
         Mots du rendu absents de la référence : {}.",
        quote_words(missing),
        quote_words(extra)
    )
}

/// Cite une liste de mots, tronquée à `MAX_WORDS_REPORTED`.
fn quote_words(words: &[String]) -> String {
    if words.is_empty() {
        return "aucun".to_string();
    }
    let shown = words.iter().take(MAX_WORDS_REPORTED).map(|w| format!("\"{w}\"")).collect::<Vec<_>>().join(", ");
    if words.len() > MAX_WORDS_REPORTED {
        format!("{shown} (et {} autres)", words.len() - MAX_WORDS_REPORTED)
    } else {
        shown
    }
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

/// Écrit `template.typ` et `example_data.json` dans le dossier de la
/// référence.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `artifacts` - Emplacements calculés par `template_artifacts`.
/// * `validated` - Template validé.
///
/// # Returns
///
/// `Ok(())` une fois les deux fichiers écrits.
pub fn write_template_files(
    data_dir: &Path,
    artifacts: &TemplateArtifacts,
    validated: &ValidatedTemplate,
) -> Result<(), AppError> {
    let files = [
        (&artifacts.template_path, &validated.template_typ),
        (&artifacts.example_data_path, &validated.example_data_json),
    ];
    for (relative, content) in files {
        let path = storage_paths::to_absolute(data_dir, relative);
        std::fs::write(&path, content.as_bytes())
            .map_err(|e| AppError::Validation(format!("Écriture de {} impossible : {e}", path.display())))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "templatize_tests.rs"]
mod tests;
