//! Traduction des libellés fixes d'un document (titres de sections,
//! formules de la lettre, abréviations de mois) quand la langue de l'offre
//! diffère de celle de la référence. Le résultat est mis en cache par
//! l'appelant (une traduction par version de référence et par langue).

// Importation des crates externes
use serde::{de::DeserializeOwned, Serialize};

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::{cv_labels_schema, lm_labels_schema};
use crate::models::doc_lang::language_name;
use crate::models::{CvLabels, LmLabels};
use crate::llm::target::ClaudeTarget;

/// Version du prompt de traduction des libellés : un cache écrit avec une
/// autre version est retraduit. À incrémenter à chaque modification du prompt.
pub const LABELS_PROMPT_VERSION: u32 = 2;

/// Traduit un jeu de libellés selon un schéma donné.
async fn translate<T: Serialize + DeserializeOwned>(
    labels: &T,
    doc_label: &str,
    schema: serde_json::Value,
    from: &str,
    to: &str,
    target: &ClaudeTarget,
) -> Result<T, AppError> {
    let system_prompt = format!(
        "Tu traduis les libellés fixes d'un document de type {doc_label}. Langue source : {}. Langue cible : {}.\n\
         Utilise les formules usuelles de la langue cible pour ce type de document (formule d'appel, formule de politesse), pas une traduction mot à mot.\n\
         Garde la même casse, la même ponctuation et les mêmes espaces en début et en fin de chaque valeur.\n\
         Une valeur vide reste vide.",
        language_name(from),
        language_name(to)
    );
    let source = serde_json::to_string_pretty(labels)
        .map_err(|e| AppError::Llm(format!("Libellés non sérialisables : {e}")))?;
    let raw = call_claude("traduction:libellés", &system_prompt, &format!("LIBELLÉS :\n{source}"), Some(schema), target).await?;
    serde_json::from_str(raw.trim())
        .map_err(|e| AppError::Llm(format!("La traduction des libellés n'a pas retourné le JSON attendu : {e}\nRéponse brute : {raw}")))
}

/// Traduit les libellés d'un CV.
///
/// # Arguments
///
/// * `labels` - Libellés de la référence.
/// * `from` - Langue de la référence.
/// * `to` - Langue cible.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Libellés traduits, à valider par l'appelant (`documents::labels_cache`).
pub async fn translate_cv_labels(labels: &CvLabels, from: &str, to: &str, target: &ClaudeTarget) -> Result<CvLabels, AppError> {
    translate(labels, "CV", cv_labels_schema(), from, to, target).await
}

/// Traduit les libellés d'une lettre de motivation.
///
/// # Arguments
///
/// * `labels` - Libellés de la référence.
/// * `from` - Langue de la référence.
/// * `to` - Langue cible.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Libellés traduits.
pub async fn translate_lm_labels(labels: &LmLabels, from: &str, to: &str, target: &ClaudeTarget) -> Result<LmLabels, AppError> {
    translate(labels, "lettre de motivation", lm_labels_schema(), from, to, target).await
}
