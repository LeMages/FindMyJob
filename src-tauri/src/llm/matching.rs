//! Mise en correspondance du profil candidat avec une offre analysée, via
//! l'API Claude — étape 2 du pipeline de génération de documents. Décide
//! quelles expériences, projets et compétences mettre en avant.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::models::ProfileDto;

/// Schéma JSON attendu pour la stratégie de matching, utilisé comme
/// structured output (`output_config.format`).
fn matching_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "relevant_experiences": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "number" },
                        "relevance": { "type": "string", "enum": ["high", "medium", "low"] },
                        "highlights": { "type": "array", "items": { "type": "string" } },
                        "keywords_covered": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["id", "relevance", "highlights", "keywords_covered"],
                    "additionalProperties": false
                }
            },
            "relevant_projects": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "number" },
                        "relevance": { "type": "string", "enum": ["high", "medium", "low"] },
                        "highlights": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["id", "relevance", "highlights"],
                    "additionalProperties": false
                }
            },
            "relevant_skills": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "skill": { "type": "string" },
                        "match_type": { "type": "string", "enum": ["exact", "related", "transferable"] }
                    },
                    "required": ["skill", "match_type"],
                    "additionalProperties": false
                }
            },
            "summary_strategy": { "type": "string" },
            "gaps": { "type": "array", "items": { "type": "string" } }
        },
        "required": [
            "relevant_experiences", "relevant_projects", "relevant_skills",
            "summary_strategy", "gaps"
        ],
        "additionalProperties": false
    })
}

/// Met en correspondance le profil du candidat avec une offre analysée, via
/// l'API Claude, pour décider quoi mettre en avant dans le CV et la lettre.
///
/// # Arguments
///
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `profile` - Profil complet du candidat.
///
/// # Returns
///
/// Le JSON de stratégie de matching (sérialisé de façon canonique). Toute
/// erreur d'appel, de refus ou de format inattendu remonte telle quelle.
pub async fn match_profile(offer_analysis: &str, profile: &ProfileDto) -> Result<String, AppError> {
    let system_prompt = r#"Tu es un stratège de candidature. Analyse le profil du candidat par rapport à l'offre et décide quoi mettre en avant.

Description des champs à renseigner :
- relevant_experiences : pour chaque expérience du profil pertinente pour l'offre, son id, son niveau de pertinence, les aspects à mettre en avant, et les mots-clés de l'offre qu'elle couvre
- relevant_projects : même logique pour les projets du profil
- relevant_skills : compétences du profil qui correspondent à l'offre, avec le type de correspondance (exact = compétence identique, related = compétence proche, transferable = compétence transférable)
- summary_strategy : en 2-3 phrases, quel angle donner au résumé/à l'accroche du candidat pour cette offre
- gaps : compétences demandées par l'offre que le candidat n'a pas, pour en être conscient — ne jamais en inventer la maîtrise ailleurs dans le pipeline"#;

    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil: {e}")))?;

    let user_message = format!(
        "Analyse d'offre :\n{offer_analysis}\n\nProfil du candidat :\n{profile_json}"
    );

    let raw_response = call_claude(system_prompt, &user_message, Some(matching_schema())).await?;

    // Le structured output garantit normalement un JSON conforme au schéma ;
    // ce parsing reste un filet de sécurité plutôt qu'une validation de forme.
    let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "Le matching n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })?;

    serde_json::to_string(&parsed)
        .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")))
}
