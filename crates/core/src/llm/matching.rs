//! Mise en correspondance du profil candidat avec une offre analysée, via
//! l'API Claude (étape 2 du pipeline de génération de documents). Juge
//! toutes les expériences, retient les N projets les plus pertinents et les
//! compétences qui répondent à l'offre.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::models::{MatchingResult, ProfileDto};
use crate::llm::target::ClaudeTarget;

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
/// * `project_count` - Nombre de projets à retenir (celui de la référence).
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// `MatchingResult`. Les identifiants ne sont pas vérifiés ici : la
/// sélection (`docgen::selection::select`) ignore ceux absents du profil.
pub async fn match_profile(
    offer_analysis: &str,
    profile: &ProfileDto,
    project_count: usize,
    target: &ClaudeTarget,
) -> Result<MatchingResult, AppError> {
    let system_prompt = format!(
        "Tu es un stratège de candidature. Tu compares le profil du candidat à l'offre et tu décides quoi mettre en avant.\n\n\
Description des champs à renseigner :\n\
- relevant_experiences : toutes les expériences du profil, sans exception. Pour chacune, son id, son niveau de pertinence pour l'offre, les faits du profil à mettre en avant et les mots-clés de l'offre auxquels ces faits répondent réellement.\n\
- relevant_projects : les {project_count} projets du profil les plus pertinents pour l'offre (ou tous s'il y en a moins), du plus pertinent au moins pertinent, avec la même logique.\n\
- relevant_skills : les compétences du profil qui répondent à l'offre, avec le type de correspondance (exact pour une compétence identique, related pour une compétence proche, transferable pour une compétence transférable).\n\
- summary_strategy : en deux ou trois phrases, l'angle à donner au résumé et à l'accroche pour cette offre.\n\
- gaps : les compétences demandées par l'offre que le profil n'a pas. Elles servent à savoir quoi ne jamais affirmer.\n\n\
Les faits à mettre en avant viennent tous du profil : n'en invente aucun."
    );

    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil : {e}")))?;
    let user_message = format!("Analyse d'offre :\n{offer_analysis}\n\nProfil du candidat :\n{profile_json}");

    let raw_response = call_claude(&system_prompt, &user_message, Some(matching_schema()), target).await?;
    serde_json::from_str(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!("Le matching n'a pas retourné le JSON attendu : {e}\nRéponse brute : {raw_response}"))
    })
}
