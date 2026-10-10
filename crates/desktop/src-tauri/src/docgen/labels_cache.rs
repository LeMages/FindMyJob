//! Libellés fixes des documents dans la langue de l'offre : ceux de la
//! référence quand les langues coïncident, sinon une traduction LLM mise
//! en cache dans le dossier de la version (`labels.<lang>.<doc>.json`).

// Importation des modules standard
use std::future::Future;
use std::path::Path;

// Importation des crates externes
use serde::{de::DeserializeOwned, Serialize};

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::llm::label_translation::{translate_cv_labels, translate_lm_labels};
use findmyjob_core::models::{CvLabels, LmLabels};
use findmyjob_core::trace_log;

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

/// Résout des libellés génériquement : référence, cache, puis traduction.
async fn resolve<T, Fut>(
    example: &T,
    example_lang: &str,
    target_lang: &str,
    cache_file: &Path,
    translate: impl FnOnce() -> Fut,
) -> Result<ResolvedLabels<T>, AppError>
where
    T: Clone + Serialize + DeserializeOwned,
    Fut: Future<Output = Result<T, AppError>>,
{
    if example_lang == target_lang {
        return Ok(ResolvedLabels { labels: example.clone(), translated: false });
    }

    // Un cache illisible est ignoré (puis réécrit) plutôt que bloquant :
    // il ne contient qu'une traduction recalculable.
    if let Ok(cached) = std::fs::read_to_string(cache_file) {
        match serde_json::from_str(&cached) {
            Ok(labels) => return Ok(ResolvedLabels { labels, translated: true }),
            Err(e) => trace_log::log(&format!("[labels] cache illisible {} : {e}", cache_file.display())),
        }
    }

    let labels = translate().await?;
    let json = serde_json::to_string_pretty(&labels)
        .map_err(|e| AppError::Validation(format!("Libellés non sérialisables : {e}")))?;
    if let Err(e) = std::fs::write(cache_file, json) {
        // L'écriture du cache n'est pas nécessaire à la génération en cours.
        trace_log::log(&format!("[labels] écriture du cache {} impossible : {e}", cache_file.display()));
    }
    Ok(ResolvedLabels { labels, translated: true })
}

/// Libellés du CV dans la langue cible.
///
/// # Arguments
///
/// * `example` - Libellés de la référence.
/// * `example_lang` - Langue de la référence.
/// * `target_lang` - Langue de l'offre.
/// * `dir` - Dossier de la version de référence (emplacement du cache).
/// * `model` - Identifiant du modèle Claude à utiliser pour une traduction.
///
/// # Returns
///
/// `ResolvedLabels<CvLabels>`.
pub async fn resolve_cv_labels(
    example: &CvLabels,
    example_lang: &str,
    target_lang: &str,
    dir: &Path,
    model: &str,
) -> Result<ResolvedLabels<CvLabels>, AppError> {
    let cache = dir.join(format!("labels.{target_lang}.cv.json"));
    resolve(example, example_lang, target_lang, &cache, || {
        translate_cv_labels(example, example_lang, target_lang, model)
    })
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
/// * `model` - Identifiant du modèle Claude à utiliser pour une traduction.
///
/// # Returns
///
/// `ResolvedLabels<LmLabels>`.
pub async fn resolve_lm_labels(
    example: &LmLabels,
    example_lang: &str,
    target_lang: &str,
    dir: &Path,
    model: &str,
) -> Result<ResolvedLabels<LmLabels>, AppError> {
    let cache = dir.join(format!("labels.{target_lang}.lm.json"));
    let mut resolved = resolve(example, example_lang, target_lang, &cache, || {
        translate_lm_labels(example, example_lang, target_lang, model)
    })
    .await?;

    let prefix = &mut resolved.labels.date_prefix;
    if example.date_prefix.ends_with(' ') && !prefix.trim().is_empty() && !prefix.ends_with(' ') {
        prefix.push(' ');
    }
    Ok(resolved)
}
