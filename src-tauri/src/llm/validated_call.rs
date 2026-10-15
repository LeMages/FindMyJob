//! Appel LLM dont la sortie est validée en Rust : en cas de problème, nouvel
//! appel avec la sortie précédente et la liste des problèmes (même
//! mécanique que `writing_rules::with_feedback`), puis échec clair après le
//! dernier essai. Sans linter ni nettoyage mécanique, contrairement à la
//! boucle de rédaction (`writing_loop`) : toute sortie invalide est rejetée.

// Importation des modules standard
use std::future::Future;

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::error::AppError;
use crate::llm::writing_loop::{WritingFeedback, MAX_WRITING_ATTEMPTS};
use crate::trace_log;

/// Appelle le LLM jusqu'à obtenir une sortie valide, en
/// `MAX_WRITING_ATTEMPTS` essais au plus.
///
/// # Arguments
///
/// * `label` - Sujet de l'appel, en tête de phrase (ex. « Le matching »),
///   pour les journaux et le message d'erreur.
/// * `call` - Appel LLM, avec le retour de l'essai précédent.
/// * `validate` - Problèmes d'une sortie, vide si elle est valide.
///
/// # Returns
///
/// La première sortie valide, ou `AppError::Llm` si la dernière reste
/// invalide.
pub async fn call_until_valid<T, Call, Fut>(
    label: &str,
    call: Call,
    validate: impl Fn(&T) -> Vec<String>,
) -> Result<T, AppError>
where
    T: Serialize,
    Call: Fn(Option<WritingFeedback>) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let mut feedback: Option<WritingFeedback> = None;
    let mut problems: Vec<String> = Vec::new();

    for attempt in 1..=MAX_WRITING_ATTEMPTS {
        let output = call(feedback.take()).await?;
        problems = validate(&output);
        if problems.is_empty() {
            return Ok(output);
        }

        trace_log::log(&format!("[llm] {label} : essai {attempt}/{MAX_WRITING_ATTEMPTS}, problèmes : {problems:?}"));
        let previous_output = serde_json::to_string_pretty(&output)
            .map_err(|e| AppError::Llm(format!("Sortie non sérialisable ({label}) : {e}")))?;
        feedback = Some(WritingFeedback { previous_output, problems: problems.clone() });
    }

    Err(AppError::Llm(format!(
        "{label} reste invalide après {MAX_WRITING_ATTEMPTS} essais : {}",
        problems.join(" ; ")
    )))
}
