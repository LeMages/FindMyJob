//! Textes d'une référence dans la langue d'un document : ceux de la
//! référence quand les langues coïncident (aucun appel), sinon une
//! traduction validée, mise en cache par version et par langue
//! (`reference_texts.<lang>.json`, dans le dossier de la version).
//!
//! Le cache porte la version du prompt de traduction et la langue source ;
//! il est revalidé à la lecture (mêmes éléments, aucun vidé). Un cache
//! périmé ou invalide est signalé dans les traces puis retraduit. Une
//! traduction en échec n'est jamais mise en cache.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::docgen::reference_texts_translation::{translatable_items, translation_problems};
use crate::error::AppError;
use crate::llm::reference_translation::{translate_reference_texts, REFERENCE_TEXTS_PROMPT_VERSION};
use crate::llm::ClaudeTarget;
use crate::models::ReferenceTexts;
use crate::trace_log;

/// Textes résolus, et s'ils ont été traduits.
///
/// # Attributes
///
/// * `texts` - Textes dans la langue du document.
/// * `translated` - Vrai s'ils viennent d'une traduction (cache ou LLM).
pub struct ResolvedTexts {
    pub texts: ReferenceTexts,
    pub translated: bool,
}

/// Fichier de cache d'une traduction.
///
/// # Attributes
///
/// * `prompt_version` - Version du prompt qui l'a produite.
/// * `source_lang` - Langue des textes traduits.
/// * `texts` - Textes traduits.
#[derive(Serialize, Deserialize)]
struct CachedTexts {
    prompt_version: u32,
    source_lang: String,
    texts: ReferenceTexts,
}

/// Textes de la référence dans la langue cible.
///
/// # Arguments
///
/// * `texts` - Textes de la référence, dans sa langue.
/// * `target_lang` - Langue du document.
/// * `dir` - Dossier de la version de référence (emplacement du cache).
/// * `target` - Clé API et modèle Claude, pour une traduction.
///
/// # Returns
///
/// `ResolvedTexts`.
pub async fn resolve_reference_texts(
    texts: &ReferenceTexts,
    target_lang: &str,
    dir: &Path,
    target: &ClaudeTarget,
) -> Result<ResolvedTexts, AppError> {
    if texts.lang == target_lang {
        return Ok(ResolvedTexts { texts: texts.clone(), translated: false });
    }
    let cache_file = dir.join(format!("reference_texts.{target_lang}.json"));
    if let Some(cached) = read_cache(&cache_file, texts) {
        return Ok(ResolvedTexts { texts: cached, translated: true });
    }

    let translated = translate_reference_texts(texts, target_lang, target).await?;
    let cached = CachedTexts {
        prompt_version: REFERENCE_TEXTS_PROMPT_VERSION,
        source_lang: texts.lang.clone(),
        texts: translated,
    };
    match serde_json::to_string_pretty(&cached) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&cache_file, json) {
                // Le cache n'est pas nécessaire à la génération en cours.
                trace_log::log(&format!("[textes] écriture du cache {} impossible : {e}", cache_file.display()));
            }
        }
        Err(e) => trace_log::log(&format!("[textes] cache non sérialisable : {e}")),
    }
    Ok(ResolvedTexts { texts: cached.texts, translated: true })
}

/// Lit un cache s'il est à jour et valide pour ces textes.
fn read_cache(cache_file: &Path, source: &ReferenceTexts) -> Option<ReferenceTexts> {
    let content = std::fs::read_to_string(cache_file).ok()?;
    let reason = match serde_json::from_str::<CachedTexts>(&content) {
        Ok(cached) if cached.prompt_version != REFERENCE_TEXTS_PROMPT_VERSION => {
            format!("version de prompt {} au lieu de {REFERENCE_TEXTS_PROMPT_VERSION}", cached.prompt_version)
        }
        Ok(cached) if cached.source_lang != source.lang => {
            format!("langue source {} au lieu de {}", cached.source_lang, source.lang)
        }
        Ok(cached) => {
            let problems = translation_problems(&translatable_items(source), &translatable_items(&cached.texts));
            if problems.is_empty() {
                return Some(cached.texts);
            }
            problems.join(" ; ")
        }
        Err(e) => format!("illisible ({e})"),
    };
    trace_log::log(&format!("[textes] cache {} non réutilisé : {reason}", cache_file.display()));
    None
}
