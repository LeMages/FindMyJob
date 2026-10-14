//! Boucle de rédaction avec retry : appel LLM, revue (validation bloquante
//! et linter), puis nouvel appel avec la sortie précédente et la liste des
//! problèmes, deux fois au maximum.
//!
//! Après le dernier essai, une validation bloquante en échec fait échouer
//! la rédaction ; les violations du linter, elles, ne bloquent pas : les
//! tirets restants sont remplacés mécaniquement (`finalize`) et le reste
//! est journalisé.

// Importation des modules standard
use std::future::Future;

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::docgen::lint::Violation;
use crate::error::AppError;
use crate::trace_log;

/// Nombre total d'appels LLM : un essai + 2 retries avec feedback.
pub const MAX_WRITING_ATTEMPTS: u8 = 3;

/// Retour de l'essai précédent, transmis au LLM.
///
/// # Attributes
///
/// * `previous_output` - Sortie JSON de l'essai précédent.
/// * `problems` - Problèmes relevés, une ligne chacun.
#[derive(Clone, Debug)]
pub struct WritingFeedback {
    pub previous_output: String,
    pub problems: Vec<String>,
}

/// Revue d'une sortie de rédaction.
///
/// # Attributes
///
/// * `blocking` - Problèmes de validation (identifiants, compétences, champs
///   vides) : un document ne peut pas être assemblé avec.
/// * `violations` - Violations du linter (caractères, formules, budgets).
#[derive(Default, Debug)]
pub struct Review {
    pub blocking: Vec<String>,
    pub violations: Vec<Violation>,
}

impl Review {
    fn is_clean(&self) -> bool {
        self.blocking.is_empty() && self.violations.is_empty()
    }

    fn problems(&self) -> Vec<String> {
        self.blocking
            .iter()
            .cloned()
            .chain(self.violations.iter().map(|v| format!("{} : {}", v.field, v.message)))
            .collect()
    }
}

/// Trace d'un essai de rédaction.
///
/// # Attributes
///
/// * `attempt` - Numéro de l'essai (1-based).
/// * `problems` - Problèmes relevés à la revue (vide si l'essai est accepté).
#[derive(Serialize, Clone, Debug)]
pub struct AttemptReport {
    pub attempt: u8,
    pub problems: Vec<String>,
}

/// Résultat d'une rédaction.
///
/// # Attributes
///
/// * `content` - Contenu retenu (après remplacement mécanique éventuel).
/// * `attempts` - Trace de chaque essai.
/// * `residual_violations` - Violations restantes, journalisées sans bloquer.
#[derive(Serialize, Clone, Debug)]
pub struct WritingOutcome<T> {
    pub content: T,
    pub attempts: Vec<AttemptReport>,
    pub residual_violations: Vec<Violation>,
}

/// Exécute la boucle de rédaction.
///
/// # Arguments
///
/// * `label` - Nom du document, pour les journaux et les erreurs.
/// * `call` - Appel LLM, avec le retour de l'essai précédent.
/// * `review` - Revue d'une sortie.
/// * `finalize` - Nettoyage mécanique appliqué après le dernier essai.
///
/// # Returns
///
/// `WritingOutcome`, ou une erreur si la validation bloquante échoue
/// encore après le dernier essai.
pub async fn write_with_retries<T, Call, Fut>(
    label: &str,
    call: Call,
    review: impl Fn(&T) -> Review,
    finalize: impl Fn(&mut T),
) -> Result<WritingOutcome<T>, AppError>
where
    T: Serialize,
    Call: Fn(Option<WritingFeedback>) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let mut attempts = Vec::new();
    let mut feedback: Option<WritingFeedback> = None;
    let mut last: Option<(T, Review)> = None;

    for attempt in 1..=MAX_WRITING_ATTEMPTS {
        let content = call(feedback.clone()).await?;
        let verdict = review(&content);
        attempts.push(AttemptReport { attempt, problems: verdict.problems() });
        if verdict.is_clean() {
            return Ok(WritingOutcome { content, attempts, residual_violations: Vec::new() });
        }

        trace_log::log(&format!("[writing] {label} : essai {attempt}/{MAX_WRITING_ATTEMPTS}, problèmes : {:?}", verdict.problems()));
        let previous_output = serde_json::to_string_pretty(&content)
            .map_err(|e| AppError::Llm(format!("Sortie de rédaction non sérialisable ({label}) : {e}")))?;
        feedback = Some(WritingFeedback { previous_output, problems: verdict.problems() });
        last = Some((content, verdict));
    }

    // Toujours Some : la boucle fait au moins un tour sans retour anticipé.
    let (mut content, verdict) = last.ok_or_else(|| AppError::Llm(format!("Aucune rédaction produite ({label}).")))?;
    if !verdict.blocking.is_empty() {
        return Err(AppError::Llm(format!(
            "La rédaction ({label}) reste invalide après {MAX_WRITING_ATTEMPTS} essais : {}",
            verdict.blocking.join(" ; ")
        )));
    }

    finalize(&mut content);
    let residual = review(&content).violations;
    if !residual.is_empty() {
        trace_log::log(&format!("[writing] {label} : violations conservées après nettoyage : {residual:?}"));
    }
    Ok(WritingOutcome { content, attempts, residual_violations: residual })
}
