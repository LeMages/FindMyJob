//! Étape A de la templatisation : le contenu d'une référence Typst rangé
//! dans le schéma canonique (`example_data`), sous structured output.
//!
//! Le schéma ne contient que les données : ni template, ni consignes (lues
//! par Rust dans le balisage), ni textes en dur (calculés par Rust), ce qui
//! garde la grammaire compilée sous le plafond de l'API.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::{cv_example_data_schema, lm_example_data_schema};
use crate::llm::target::ClaudeTarget;
use crate::llm::templatize_rules::{
    doc_type_label, ADDRESS_DATA_RULE, CV_DATA_FIELDS, EXAMPLE_DATA_RULES, LM_DATA_FIELDS,
};

/// Retour de validation des données d'exemple précédentes.
///
/// # Attributes
///
/// * `previous` - Données d'exemple refusées, JSON indenté.
/// * `problems` - Problèmes relevés.
pub struct ExampleFeedback {
    pub previous: String,
    pub problems: Vec<String>,
}

/// Schéma de sortie de l'étape A pour un type de document.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Le schéma, ou `None` pour un type inconnu.
pub fn example_output_schema(doc_type: &str) -> Option<Value> {
    match doc_type {
        "cv" => Some(cv_example_data_schema()),
        "lettre_motivation" => Some(lm_example_data_schema()),
        _ => None,
    }
}

/// Range le contenu d'une référence dans le schéma canonique.
///
/// # Arguments
///
/// * `reference_typ` - Source Typst de la référence.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `feedback` - Retour de l'essai précédent, `None` au premier essai.
/// * `target` - Clé API et modèle Claude.
///
/// # Returns
///
/// Les données d'exemple brutes (JSON), à valider par l'appelant.
pub async fn extract_example_data(
    reference_typ: &str,
    doc_type: &str,
    feedback: Option<&ExampleFeedback>,
    target: &ClaudeTarget,
) -> Result<Value, AppError> {
    let schema = example_output_schema(doc_type)
        .ok_or_else(|| AppError::Validation(format!("Type de document inconnu : {doc_type}")))?;
    let fields = if doc_type == "cv" { CV_DATA_FIELDS } else { LM_DATA_FIELDS };
    let label = doc_type_label(doc_type);
    let system = format!(
        "Tu lis un document Typst ({label}) et tu ranges son contenu dans le schéma JSON imposé (example_data). \
         Ces données servent ensuite à écrire un template qui, rendu avec elles, redonne exactement l'original.\n\n\
         {EXAMPLE_DATA_RULES}\n\n{fields}\n\n{ADDRESS_DATA_RULE}"
    );
    let user = user_message(reference_typ, feedback);
    let raw = call_claude("templatisation:exemple", &system, &user, Some(schema), target).await?;
    serde_json::from_str(raw.trim())
        .map_err(|e| AppError::Llm(format!("Données d'exemple : la réponse n'est pas du JSON valide ({e}).")))
}

/// Message utilisateur : document original, puis retour de l'essai précédent.
fn user_message(reference_typ: &str, feedback: Option<&ExampleFeedback>) -> String {
    let mut message = format!("DOCUMENT ORIGINAL :\n```typst\n{reference_typ}\n```");
    if let Some(feedback) = feedback {
        let problems: Vec<String> = feedback.problems.iter().map(|p| format!("- {p}")).collect();
        message.push_str(&format!(
            "\n\nTes données précédentes ont été refusées. Corrige-les en gardant ce qui est juste.\n\n\
             PROBLÈMES :\n{}\n\nDONNÉES PRÉCÉDENTES :\n```json\n{}\n```",
            problems.join("\n"),
            feedback.previous
        ));
    }
    message
}
