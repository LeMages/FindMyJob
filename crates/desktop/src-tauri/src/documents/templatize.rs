//! Templatisation d'un document de référence : appel LLM
//! (`llm::templatize`), validation du template obtenu par compilation, et
//! retry avec les problèmes relevés en feedback.
//!
//! Un template est accepté s'il :
//! - lit ses données depuis `data.json` et n'évalue ni ne calcule rien
//!   (`eval`, `datetime.today`, `read` interdits) ;
//! - compilé avec ses données d'exemple, n'a pas plus de pages que la
//!   référence, et rend le même texte qu'elle, à l'exception des textes
//!   déclarés supprimés (`dropped_text` : éléments sans champ dans le
//!   schéma) et d'une date calculée à la compilation ;
//! - compilé avec des données vides, compile toujours et n'affiche aucun mot
//!   (preuve qu'aucun contenu n'est resté écrit en dur) ;
//! - compilé avec les champs secondaires vides (coordonnées, organisations,
//!   lieux, niveaux), ne laisse aucun séparateur orphelin.

// Importation des modules standard
use std::path::Path;

// Importation des modules internes
use crate::documents::typst::{compile_template, CompiledDocument};
use crate::storage_paths;
use findmyjob_core::docgen::date_format::format_long_date;
use findmyjob_core::docgen::text_diff::{alphanumeric_words, diff_words, orphan_separators};
use findmyjob_core::error::AppError;
use findmyjob_core::llm::{templatize_reference, TemplatizeFeedback, TemplatizeOutput};
use findmyjob_core::models::{CvExampleData, LmExampleData, TemplateArtifacts};
use findmyjob_core::trace_log;

/// Nombre total d'appels LLM autorisés : un essai + 2 retries avec feedback.
const MAX_ATTEMPTS: u8 = 3;
/// Noms de mois (anglais puis français, en minuscules) : une date calculée
/// par la référence (`datetime.today`) s'affiche avec l'un d'eux, entier ou
/// abrégé, et n'a pas d'équivalent dans les données d'exemple.
const MONTH_NAMES: [&str; 24] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september", "october",
    "november", "december", "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août",
    "septembre", "octobre", "novembre", "décembre",
];
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
/// * `dropped_text` - Textes de la référence volontairement absents du template.
/// * `attempts` - Nombre d'appels LLM qu'il a fallu.
pub struct ValidatedTemplate {
    pub template_typ: String,
    pub example_data_json: String,
    pub page_count: usize,
    pub dropped_text: Vec<String>,
    pub attempts: u8,
}

/// Issue de la validation d'une tentative.
enum Validation {
    /// Template accepté ; nombre de pages du rendu avec les données d'exemple.
    Accepted { page_count: usize },
    /// Template refusé ; problèmes à renvoyer en feedback.
    Rejected(Vec<String>),
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
/// * `model` - Identifiant du modèle Claude à utiliser.
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
    model: &str,
) -> Result<ValidatedTemplate, AppError> {
    let mut feedback: Option<TemplatizeFeedback> = None;
    let computes_date = reference_typ.contains("datetime.today");

    for attempt in 1..=MAX_ATTEMPTS {
        let output = templatize_reference(reference_typ, doc_type, feedback.as_ref(), model).await?;
        let example_data_json = serde_json::to_string_pretty(&output.example_data)
            .map_err(|e| AppError::Llm(format!("Données d'exemple non sérialisables : {e}")))?;

        let context = ValidationContext { reference, computes_date, doc_type, root, fonts_dir };
        let problems = match validate(&output, &example_data_json, &context).await? {
            Validation::Accepted { page_count } => {
                return Ok(ValidatedTemplate {
                    template_typ: output.template_typ,
                    example_data_json,
                    page_count,
                    dropped_text: output.dropped_text,
                    attempts: attempt,
                });
            }
            Validation::Rejected(problems) => problems,
        };

        trace_log::log(&format!("[templatize] {doc_type} : tentative {attempt}/{MAX_ATTEMPTS} rejetée : {problems:?}"));
        feedback = Some(TemplatizeFeedback {
            previous_template: output.template_typ,
            previous_example_data: example_data_json,
            previous_dropped_text: output.dropped_text,
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
    if context.doc_type == "cv" {
        problems.extend(dated_drops(&output.dropped_text));
    }
    let blank_json = match blank_data_json(&output.example_data, context.doc_type) {
        Ok(json) => json,
        Err(problem) => {
            problems.push(problem);
            return Ok(Validation::Rejected(problems));
        }
    };

    // ------------------------------------------------------------------
    // 2. Rendus avec les données d'exemple puis avec des données vides
    //    (compilation bloquante, hors du runtime async)
    // ------------------------------------------------------------------
    let template = output.template_typ.clone();
    let example = example_for_rendering(&output.example_data, example_data_json, context)?;
    let root = context.root.to_path_buf();
    let fonts_dir = context.fonts_dir.to_path_buf();
    let sparse = sparse_data(&example)?;
    let (with_example, with_blank, with_sparse) = tokio::task::spawn_blocking(move || {
        let with_example = compile_template(template.clone(), example, root.clone(), fonts_dir.clone());
        let with_blank = compile_template(template.clone(), blank_json.into_bytes(), root.clone(), fonts_dir.clone());
        let with_sparse = compile_template(template, sparse, root, fonts_dir);
        (with_example, with_blank, with_sparse)
    })
    .await
    .map_err(|e| AppError::Validation(format!("Erreur interne de compilation Typst : {e}")))?;

    // ------------------------------------------------------------------
    // 3. Comparaison avec la référence, puis recherche de contenu en dur
    // ------------------------------------------------------------------
    let page_count = match with_example {
        Ok(rendered) => {
            problems.extend(compare_with_reference(context, &rendered, &output.dropped_text));
            Some(rendered.page_count)
        }
        Err(errors) => {
            problems.push(format!("Le template ne compile pas avec example_data : {errors}"));
            None
        }
    };
    match with_blank {
        Ok(rendered) => {
            let hardcoded = alphanumeric_words(&rendered.text);
            if !hardcoded.is_empty() {
                problems.push(format!(
                    "Rendu avec des données vides, le template affiche encore ces mots, donc écrits en dur : {}. \
                     Ils doivent venir de data.",
                    quote_words(&hardcoded)
                ));
            }
        }
        Err(errors) => problems.push(format!(
            "Le template ne compile pas quand tous les champs sont vides (chaînes vides, listes vides) : {errors}"
        )),
    }
    match with_sparse {
        Ok(rendered) => {
            let orphans = orphan_separators(&rendered.text);
            if !orphans.is_empty() {
                problems.push(format!(
                    "Rendu avec les champs secondaires vides ({}), le template laisse des séparateurs orphelins : {}. \
                     Un champ vide doit faire disparaître sa valeur et le séparateur qui l'accompagne.",
                    SPARSE_FIELDS.join(", "),
                    quote_words(&orphans)
                ));
            }
        }
        Err(errors) => problems.push(format!(
            "Le template ne compile pas quand les champs secondaires sont vides ({}) : {errors}",
            SPARSE_FIELDS.join(", ")
        )),
    }

    Ok(match page_count {
        Some(page_count) if problems.is_empty() => Validation::Accepted { page_count },
        _ => Validation::Rejected(problems),
    })
}

/// Refuse les textes datés déclarés supprimés d'un CV : ce sont des entrées
/// de liste (formation, expérience, projet), qui ont un champ dans le schéma.
fn dated_drops(dropped_text: &[String]) -> Vec<String> {
    dropped_text
        .iter()
        .filter(|text| contains_year(text))
        .map(|text| {
            format!(
                "dropped_text contient « {text} » : un élément daté d'un CV est une formation, une expérience \
                 ou un projet, qui a un champ dans le schéma. Range-le dans example_data et reproduis sa mise \
                 en page dans le template."
            )
        })
        .collect()
}

/// Indique si un texte contient une année (quatre chiffres commençant par
/// 19 ou 20, isolés).
fn contains_year(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.windows(4).enumerate().any(|(i, w)| {
        let is_year = w.iter().all(char::is_ascii_digit) && matches!((w[0], w[1]), ('1', '9') | ('2', '0'));
        let isolated_before = i == 0 || !chars[i - 1].is_ascii_digit();
        let isolated_after = chars.get(i + 4).is_none_or(|c| !c.is_ascii_digit());
        is_year && isolated_before && isolated_after
    })
}

/// Champs secondaires vidés pour le rendu « clairsemé » : coordonnées,
/// organisations, lieux et niveaux, que le profil d'un utilisateur peut
/// ne pas renseigner.
const SPARSE_FIELDS: [&str; 15] = [
    "email", "telephone", "ville", "adresse", "code_postal", "linkedin", "github", "portfolio", "entreprise", "lieu",
    "pour_qui", "ecole", "level", "location", "contact_role",
];

/// Données d'exemple dont les champs secondaires (`SPARSE_FIELDS`) sont
/// vidés, partout où ils apparaissent : un template correct n'affiche
/// alors aucun séparateur orphelin.
///
/// # Arguments
///
/// * `example` - Données d'exemple sérialisées (telles que rendues).
///
/// # Returns
///
/// Données clairsemées sérialisées.
fn sparse_data(example: &[u8]) -> Result<Vec<u8>, AppError> {
    fn clear(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, field) in map.iter_mut() {
                    if SPARSE_FIELDS.contains(&key.as_str()) && field.is_string() {
                        *field = serde_json::Value::String(String::new());
                    } else {
                        clear(field);
                    }
                }
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(clear),
            _ => {}
        }
    }
    let mut data: serde_json::Value = serde_json::from_slice(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple illisibles : {e}")))?;
    clear(&mut data);
    serde_json::to_vec(&data).map_err(|e| AppError::Validation(format!("Données clairsemées non sérialisables : {e}")))
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

/// Vérifie que les données d'exemple respectent le schéma canonique typé,
/// puis construit les données vides (mêmes langue, listes vides, chaînes
/// vides) servant à détecter le contenu écrit en dur.
///
/// # Returns
///
/// Données vides sérialisées, ou un problème à renvoyer en feedback.
fn blank_data_json(example_data: &serde_json::Value, doc_type: &str) -> Result<String, String> {
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
            let mut blank = findmyjob_core::models::CvData { lang: example.data.lang, ..Default::default() };
            blank.labels.months_short = vec![String::new(); 12];
            to_json(serde_json::to_string(&blank))
        }
        _ => {
            let example: LmExampleData = serde_json::from_value(example_data.clone())
                .map_err(|e| format!("example_data ne respecte pas le schéma de la lettre : {e}"))?;
            check_langs(&example.data.lang, &example.detected_lang)?;
            let blank = findmyjob_core::models::LmData { lang: example.data.lang, ..Default::default() };
            to_json(serde_json::to_string(&blank))
        }
    }
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
/// pages, aucun mot en trop, et chaque mot manquant couvert par le texte
/// déclaré supprimé (ou par la date calculée de la référence).
fn compare_with_reference(
    context: &ValidationContext<'_>,
    rendered: &CompiledDocument,
    dropped_text: &[String],
) -> Vec<String> {
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
    let uncovered: Vec<String> = diff
        .uncovered_missing(dropped_text)
        .into_iter()
        .filter(|word| !(context.computes_date && is_date_word(word)))
        .collect();
    let extra: Vec<String> =
        diff.extra.iter().filter(|word| !(context.computes_date && is_date_word(word))).cloned().collect();
    if !uncovered.is_empty() || !extra.is_empty() {
        problems.push(describe_text_diff(&uncovered, &extra));
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
    let cleaned = word.trim_matches(|c: char| c == '.' || c == ',').to_lowercase();
    if !cleaned.is_empty() && cleaned.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    cleaned.chars().count() >= 3 && MONTH_NAMES.iter().any(|month| month.starts_with(&cleaned))
}

/// Formule un écart de texte pour le feedback.
fn describe_text_diff(uncovered: &[String], extra: &[String]) -> String {
    format!(
        "Rendu avec example_data, le texte diffère de la référence. \
         Mots de la référence absents du rendu et non déclarés dans dropped_text : {}. \
         Mots du rendu absents de la référence : {}.",
        quote_words(uncovered),
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
