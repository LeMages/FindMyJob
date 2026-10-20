//! Libellés fixes des documents dans la langue de l'offre : ceux de la
//! référence quand les langues coïncident, sinon une traduction LLM mise
//! en cache dans le dossier de la version (`labels.<lang>.<doc>.json`).
//!
//! Le cache porte la version du prompt de traduction et la langue source ;
//! il est validé à la lecture comme à l'écriture (12 mois, aucun libellé
//! vidé). Un cache périmé ou invalide est signalé dans les traces puis
//! retraduit, jamais réutilisé.

// Importation des modules standard
use std::future::Future;
use std::path::Path;

// Importation des crates externes
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// Importation des modules internes
use crate::error::AppError;
use crate::llm::label_translation::{translate_cv_labels, translate_lm_labels, LABELS_PROMPT_VERSION};
use crate::llm::ClaudeTarget;
use crate::models::{CvLabels, LmLabels};
use crate::trace_log;

/// Libellés résolus, et s'ils ont été traduits.
///
/// # Attributes
///
/// * `labels` - Libellés à utiliser.
/// * `translated` - Vrai si les libellés viennent d'une traduction (cache ou LLM).
pub struct ResolvedLabels<T> {
    pub labels: T,
    pub translated: bool,
}

/// Fichier de cache d'une traduction de libellés.
///
/// # Attributes
///
/// * `prompt_version` - Version du prompt qui l'a produite.
/// * `source_lang` - Langue des libellés traduits.
/// * `labels` - Libellés traduits.
#[derive(Serialize, Deserialize)]
struct CachedLabels<T> {
    prompt_version: u32,
    source_lang: String,
    labels: T,
}

/// Résout des libellés génériquement : référence, cache valide, puis traduction.
async fn resolve<T, Fut>(
    example: &T,
    langs: (&str, &str),
    cache_file: &Path,
    validate: impl Fn(&T) -> Vec<String>,
    translate: impl FnOnce() -> Fut,
) -> Result<ResolvedLabels<T>, AppError>
where
    T: Clone + Serialize + DeserializeOwned,
    Fut: Future<Output = Result<T, AppError>>,
{
    let (example_lang, target_lang) = langs;
    if example_lang == target_lang {
        return Ok(ResolvedLabels { labels: example.clone(), translated: false });
    }
    if let Some(labels) = read_cache(cache_file, example_lang, &validate) {
        return Ok(ResolvedLabels { labels, translated: true });
    }

    let labels = translate().await?;
    let problems = validate(&labels);
    if !problems.is_empty() {
        return Err(AppError::Llm(format!("La traduction des libellés est invalide : {}", problems.join(" ; "))));
    }
    let cached = CachedLabels { prompt_version: LABELS_PROMPT_VERSION, source_lang: example_lang.to_string(), labels };
    let json = serde_json::to_string_pretty(&cached)
        .map_err(|e| AppError::Validation(format!("Libellés non sérialisables : {e}")))?;
    if let Err(e) = std::fs::write(cache_file, json) {
        // L'écriture du cache n'est pas nécessaire à la génération en cours.
        trace_log::log(&format!("[labels] écriture du cache {} impossible : {e}", cache_file.display()));
    }
    Ok(ResolvedLabels { labels: cached.labels, translated: true })
}

/// Lit un cache s'il est à jour et valide. Un cache absent n'est pas
/// signalé ; un cache illisible, périmé ou invalide l'est, puis ignoré.
fn read_cache<T: DeserializeOwned>(cache_file: &Path, source_lang: &str, validate: &impl Fn(&T) -> Vec<String>) -> Option<T> {
    let content = std::fs::read_to_string(cache_file).ok()?;
    let reason = match serde_json::from_str::<CachedLabels<T>>(&content) {
        Ok(cached) if cached.prompt_version != LABELS_PROMPT_VERSION => {
            format!("version de prompt {} au lieu de {LABELS_PROMPT_VERSION}", cached.prompt_version)
        }
        Ok(cached) if cached.source_lang != source_lang => {
            format!("langue source {} au lieu de {source_lang}", cached.source_lang)
        }
        Ok(cached) => {
            let problems = validate(&cached.labels);
            if problems.is_empty() {
                return Some(cached.labels);
            }
            problems.join(" ; ")
        }
        Err(e) => format!("illisible ou d'un ancien format ({e})"),
    };
    trace_log::log(&format!("[labels] cache {} non réutilisé : {reason}", cache_file.display()));
    None
}

/// Problèmes de libellés traduits : un libellé non vide dans la référence
/// ne doit pas être vidé.
fn emptied(pairs: &[(&str, &str, &str)]) -> Vec<String> {
    pairs
        .iter()
        .filter(|(_, source, translated)| !source.trim().is_empty() && translated.trim().is_empty())
        .map(|(name, _, _)| format!("{name} est vide"))
        .collect()
}

/// Valide des libellés de CV traduits.
fn cv_problems(example: &CvLabels, labels: &CvLabels) -> Vec<String> {
    let mut problems = emptied(&[
        ("section_profile", &example.section_profile, &labels.section_profile),
        ("section_skills", &example.section_skills, &labels.section_skills),
        ("section_experiences", &example.section_experiences, &labels.section_experiences),
        ("section_projects", &example.section_projects, &labels.section_projects),
        ("section_formations", &example.section_formations, &labels.section_formations),
        ("section_langues", &example.section_langues, &labels.section_langues),
        ("interets_label", &example.interets_label, &labels.interets_label),
        ("permis", &example.permis, &labels.permis),
        ("present", &example.present, &labels.present),
    ]);
    if labels.months_short.len() != 12 {
        problems.push(format!("{} abréviations de mois au lieu de 12", labels.months_short.len()));
    }
    problems
}

/// Valide des libellés de lettre traduits. `date_prefix` peut être vidé :
/// l'anglais n'a pas d'équivalent au « le » de « Thyez, le 13 octobre ».
fn lm_problems(example: &LmLabels, labels: &LmLabels) -> Vec<String> {
    emptied(&[
        ("subject_prefix", &example.subject_prefix, &labels.subject_prefix),
        ("salutation", &example.salutation, &labels.salutation),
        ("closing", &example.closing, &labels.closing),
    ])
}

/// Libellés du CV dans la langue cible.
///
/// # Arguments
///
/// * `example` - Libellés de la référence.
/// * `example_lang` - Langue de la référence.
/// * `target_lang` - Langue de l'offre.
/// * `dir` - Dossier de la version de référence (emplacement du cache).
/// * `target` - Clé API et modèle Claude à utiliser pour une traduction.
///
/// # Returns
///
/// `ResolvedLabels<CvLabels>`.
pub async fn resolve_cv_labels(
    example: &CvLabels,
    example_lang: &str,
    target_lang: &str,
    dir: &Path,
    target: &ClaudeTarget,
) -> Result<ResolvedLabels<CvLabels>, AppError> {
    let cache = dir.join(format!("labels.{target_lang}.cv.json"));
    resolve(
        example,
        (example_lang, target_lang),
        &cache,
        |labels| cv_problems(example, labels),
        || translate_cv_labels(example, example_lang, target_lang, target),
    )
    .await
}

/// Libellés de la lettre dans la langue cible.
///
/// Le template colle `date_prefix` à la date tel quel : l'espace final de
/// la référence (« le ») est reporté sur la traduction si elle l'a perdu.
///
/// # Arguments
///
/// * `example` - Libellés de la référence.
/// * `example_lang` - Langue de la référence.
/// * `target_lang` - Langue de l'offre.
/// * `dir` - Dossier de la version de référence (emplacement du cache).
/// * `target` - Clé API et modèle Claude à utiliser pour une traduction.
///
/// # Returns
///
/// `ResolvedLabels<LmLabels>`.
pub async fn resolve_lm_labels(
    example: &LmLabels,
    example_lang: &str,
    target_lang: &str,
    dir: &Path,
    target: &ClaudeTarget,
) -> Result<ResolvedLabels<LmLabels>, AppError> {
    let cache = dir.join(format!("labels.{target_lang}.lm.json"));
    let mut resolved = resolve(
        example,
        (example_lang, target_lang),
        &cache,
        |labels| lm_problems(example, labels),
        || translate_lm_labels(example, example_lang, target_lang, target),
    )
    .await?;

    let prefix = &mut resolved.labels.date_prefix;
    if example.date_prefix.ends_with(' ') && !prefix.trim().is_empty() && !prefix.ends_with(' ') {
        prefix.push(' ');
    }
    Ok(resolved)
}
