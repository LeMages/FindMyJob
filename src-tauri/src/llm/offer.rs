//! Analyse d'une offre d'emploi via l'API Claude — étape 1 du pipeline de
//! génération de documents. Le résultat (JSON) sert d'entrée à l'étape de
//! matching de profil (`llm::matching`).

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::target::ClaudeTarget;

/// Schéma JSON attendu pour l'analyse d'offre, utilisé comme structured
/// output (`output_config.format`).
fn offer_analysis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "job_title": { "type": "string" },
            "company_context": { "type": "string" },
            "required_skills": {
                "type": "object",
                "properties": {
                    "technical": { "type": "array", "items": { "type": "string" } },
                    "soft": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["technical", "soft"],
                "additionalProperties": false
            },
            "preferred_skills": {
                "type": "object",
                "properties": {
                    "technical": { "type": "array", "items": { "type": "string" } },
                    "soft": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["technical", "soft"],
                "additionalProperties": false
            },
            "experience_level": { "type": "string", "enum": ["junior", "mid", "senior", "lead"] },
            "key_responsibilities": { "type": "array", "items": { "type": "string" } },
            "keywords": { "type": "array", "items": { "type": "string" } },
            "language": { "type": "string", "enum": ["fr", "en"] }
        },
        "required": [
            "job_title", "company_context", "required_skills", "preferred_skills",
            "experience_level", "key_responsibilities", "keywords", "language"
        ],
        "additionalProperties": false
    })
}

/// Analyse une offre d'emploi via l'API Claude et en extrait une structure
/// exploitable par les étapes suivantes du pipeline (matching, rédaction).
///
/// # Arguments
///
/// * `description_offre` - Texte brut de l'offre d'emploi.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Le JSON d'analyse (sérialisé de façon canonique). Toute erreur d'appel,
/// de refus ou de format inattendu remonte telle quelle.
pub async fn analyze_offer(description_offre: &str, target: &ClaudeTarget) -> Result<String, AppError> {
    let system_prompt = r#"Tu es un expert en recrutement. Analyse cette offre d'emploi et retourne un JSON structuré.

Description des champs à renseigner :
- job_title : intitulé du poste
- company_context : secteur, taille, culture de l'entreprise si mentionnés (chaîne vide si rien n'est précisé)
- required_skills : compétences explicitement exigées, séparées en technical (techniques) et soft (comportementales)
- preferred_skills : compétences mentionnées comme un plus, non obligatoires, même découpage
- experience_level : niveau d'expérience visé
- key_responsibilities : missions principales du poste
- keywords : mots-clés importants de l'offre, à reprendre dans le CV/la lettre de motivation
- language : langue dans laquelle l'offre est rédigée"#;

    let user_message = format!("Analyse cette offre d'emploi :\n\n{description_offre}");

    let raw_response = call_claude("analyse d'offre", system_prompt, &user_message, Some(offer_analysis_schema()), target).await?;

    // Le structured output garantit normalement un JSON conforme au schéma ;
    // ce parsing reste un filet de sécurité plutôt qu'une validation de forme.
    let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "L'analyse de l'offre n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })?;

    serde_json::to_string(&parsed)
        .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")))
}
