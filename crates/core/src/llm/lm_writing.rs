//! Rédaction des champs de prose de la lettre de motivation (objet,
//! paragraphes) via l'API Claude, avec revue et retry
//! (`llm::writing_loop`). Expéditeur, destinataire, date, formule d'appel
//! et formule de politesse sont remplis par Rust.

// Importation des crates externes
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::budgets::LmBudget;
use crate::docgen::lint::{lint_lm_budget, lint_prose, strip_dashes};
use crate::docgen::selection::Selection;
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::object;
use crate::llm::profile_facts::profile_facts_block;
use crate::llm::writing_loop::{write_with_retries, Review, WritingFeedback, WritingOutcome};
use crate::llm::writing_rules::{language_rule, style_model_block, with_feedback, COMMON_RULES};
use crate::models::{LmContent, LmData, MatchingResult, ProfileDto};
use crate::llm::target::ClaudeTarget;

/// Consignes propres à la lettre.
const LM_RULES: &str = "Un paragraphe ne commence pas par une phrase nominale qui annonce son argument : il entre directement dans le propos, comme le paragraphe de même rang du modèle.
Les paragraphes ne contiennent ni la formule d'appel ni la formule de politesse finale : elles sont ajoutées à part.";

/// Entrées de la rédaction de la lettre.
///
/// # Attributes
///
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `matching` - Résultat du matching.
/// * `selection` - Entrées du profil retenues.
/// * `profile` - Profil complet du candidat.
/// * `example` - Contenu de la référence (modèle de style).
/// * `budget` - Budget de longueur.
/// * `company` - Entreprise destinataire (vide si inconnue).
/// * `target_lang` - Langue de la lettre (celle de l'offre).
/// * `example_lang` - Langue de la référence.
pub struct LmWritingInput<'a> {
    pub offer_analysis: &'a str,
    pub matching: &'a MatchingResult,
    pub selection: &'a Selection,
    pub profile: &'a ProfileDto,
    pub example: &'a LmData,
    pub budget: LmBudget,
    pub company: &'a str,
    pub target_lang: &'a str,
    pub example_lang: &'a str,
}

/// Schéma de sortie de la rédaction de la lettre.
fn lm_content_schema() -> Value {
    let text = json!({ "type": "string", "minLength": 1 });
    object(&[
        ("subject", text.clone()),
        ("paragraphs", json!({ "type": "array", "items": text, "minItems": 1 })),
    ])
}

/// Prose de la référence : modèle de style et base des formules autorisées.
pub fn example_prose(example: &LmData) -> String {
    let paragraphs = example
        .paragraphs
        .iter()
        .enumerate()
        .map(|(i, p)| format!("Paragraphe {} : {p}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Objet : {}\n{paragraphs}", example.subject)
}

/// Bloc des budgets de la lettre, paragraphe par paragraphe.
fn budget_block(budget: &LmBudget) -> String {
    let mut lines = vec![
        "BUDGETS (limites dures)".to_string(),
        format!("subject : entre {} et {} mots.", budget.subject_words.min, budget.subject_words.max),
        format!("paragraphs : exactement {} paragraphes.", budget.paragraph_count()),
    ];
    for (i, range) in budget.paragraph_words.iter().enumerate() {
        lines.push(format!("Paragraphe {} : entre {} et {} mots.", i + 1, range.min, range.max));
    }
    lines.join("\n")
}

/// Prompt système de la rédaction de la lettre.
fn system_prompt(input: &LmWritingInput<'_>) -> String {
    format!(
        "Tu rédiges l'objet et les paragraphes du corps d'une lettre de motivation adaptée à une offre.\n\n\
         {COMMON_RULES}\n{LM_RULES}\n{}\n\n\
         Description des champs.\n\
         subject : l'objet de la lettre, sans son préfixe (« Objet : » est ajouté à part).\n\
         paragraphs : les paragraphes du corps, dans l'ordre. Chacun joue le même rôle que le paragraphe de même rang du modèle.",
        language_rule(input.target_lang, input.example_lang)
    )
}

/// Message utilisateur de la rédaction de la lettre.
fn user_message(input: &LmWritingInput<'_>) -> String {
    let company = if input.company.trim().is_empty() { "non renseignée" } else { input.company };
    format!(
        "ENTREPRISE DESTINATAIRE : {company}\n\nANALYSE DE L'OFFRE :\n{}\n\n{}\n\n{}\n\n{}",
        input.offer_analysis,
        profile_facts_block(input.profile, input.selection, input.matching),
        style_model_block(&example_prose(input.example)),
        budget_block(&input.budget)
    )
}

/// Revue complète d'un contenu de lettre.
fn review(content: &LmContent, budget: &LmBudget, example_prose: &str) -> Review {
    let fields = content.prose_fields();
    let blocking = fields.iter().filter(|f| f.text.trim().is_empty()).map(|f| format!("{} est vide.", f.path)).collect();
    let mut violations = lint_prose(&fields, example_prose);
    violations.extend(lint_lm_budget(content, budget));
    Review { blocking, violations }
}

/// Appelle le LLM une fois.
async fn call_once(
    input: &LmWritingInput<'_>,
    feedback: Option<WritingFeedback>,
    target: &ClaudeTarget,
) -> Result<LmContent, AppError> {
    let message = with_feedback(user_message(input), feedback.as_ref());
    let raw = call_claude(&system_prompt(input), &message, Some(lm_content_schema()), target).await?;
    serde_json::from_str(raw.trim()).map_err(|e| {
        AppError::Llm(format!("La rédaction de la lettre n'a pas retourné le JSON attendu : {e}\nRéponse brute : {raw}"))
    })
}

/// Rédige l'objet et les paragraphes de la lettre, avec revue et jusqu'à
/// 2 retries.
///
/// # Arguments
///
/// * `input` - Entrées de la rédaction.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// `WritingOutcome<LmContent>`.
pub async fn write_lm(input: &LmWritingInput<'_>, target: &ClaudeTarget) -> Result<WritingOutcome<LmContent>, AppError> {
    let prose = example_prose(input.example);
    write_with_retries(
        "lettre",
        |feedback| call_once(input, feedback, target),
        |content| review(content, &input.budget, &prose),
        |content| content.map_prose(strip_dashes),
    )
    .await
}
