//! Rédaction du contenu du CV et de la lettre de motivation via l'API
//! Claude — étapes 3a/3b du pipeline de génération de documents. Produit du
//! texte structuré (pas de Typst) : c'est `llm::assemble` qui le transforme
//! ensuite en document.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::models::ProfileDto;

/// Règles communes aux deux rédactions, rappelées dans chaque prompt système
/// pour garantir l'honnêteté du contenu généré.
const RULES_HONNETETE: &str = "Règles strictes :\n\
- Tu ne peux que reformuler ce qui existe dans le profil, JAMAIS inventer une compétence, une expérience ou un projet\n\
- Adapte le wording pour reprendre les mots-clés de l'offre quand c'est honnête\n\
- Rédige dans la langue de l'offre (champ `language` de l'analyse d'offre)";

/// Schéma JSON attendu pour le contenu du CV.
fn cv_content_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "header": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" }
                },
                "required": ["title"],
                "additionalProperties": false
            },
            "summary": { "type": "string" },
            "experiences": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "number" },
                        "bullets": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["id", "bullets"],
                    "additionalProperties": false
                }
            },
            "projects": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "number" },
                        "description": { "type": "string" }
                    },
                    "required": ["id", "description"],
                    "additionalProperties": false
                }
            },
            "skills_grouping": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "category": { "type": "string" },
                        "skills": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["category", "skills"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["header", "summary", "experiences", "projects", "skills_grouping"],
        "additionalProperties": false
    })
}

/// Schéma JSON attendu pour le contenu de la lettre de motivation.
fn lm_content_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "opening": { "type": "string" },
            "body_paragraphs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "argument": { "type": "string" },
                        "evidence": { "type": "string" }
                    },
                    "required": ["argument", "evidence"],
                    "additionalProperties": false
                }
            },
            "closing": { "type": "string" },
            "tone_notes": { "type": "string" }
        },
        "required": ["opening", "body_paragraphs", "closing", "tone_notes"],
        "additionalProperties": false
    })
}

/// Rédige le contenu textuel d'un CV adapté à l'offre, à partir de la
/// stratégie de matching et du profil du candidat.
///
/// # Arguments
///
/// * `matching` - JSON produit par `llm::matching::match_profile`.
/// * `profile` - Profil complet du candidat.
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
///
/// # Returns
///
/// Le JSON de contenu du CV (sérialisé de façon canonique).
pub async fn generate_cv_content(
    matching: &str,
    profile: &ProfileDto,
    offer_analysis: &str,
) -> Result<String, AppError> {
    let system_prompt = format!(
        "Tu rédiges le contenu d'un CV adapté à l'offre.\n\n{RULES_HONNETETE}\n\
- Les éléments à faible pertinence (`relevance: low`) peuvent être raccourcis mais pas supprimés\n\n\
Description des champs à renseigner :\n\
- header.title : titre professionnel adapté à l'offre\n\
- summary : résumé/accroche en 3-4 lignes, adapté à l'offre\n\
- experiences : pour chaque expérience du profil, ses bullet points reformulés (repris de la description originale, jamais inventés)\n\
- projects : pour chaque projet du profil, sa description reformulée\n\
- skills_grouping : compétences du profil regroupées par catégorie pertinente pour l'offre"
    );

    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil: {e}")))?;

    let user_message = format!(
        "Analyse d'offre :\n{offer_analysis}\n\nStratégie de matching :\n{matching}\n\nProfil du candidat :\n{profile_json}"
    );

    let raw_response = call_claude(&system_prompt, &user_message, Some(cv_content_schema())).await?;

    let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "La rédaction du CV n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })?;

    serde_json::to_string(&parsed)
        .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")))
}

/// Rédige le contenu textuel d'une lettre de motivation adaptée à l'offre, à
/// partir de la stratégie de matching et du profil du candidat.
///
/// # Arguments
///
/// * `matching` - JSON produit par `llm::matching::match_profile`.
/// * `profile` - Profil complet du candidat.
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
///
/// # Returns
///
/// Le JSON de contenu de la lettre (sérialisé de façon canonique).
pub async fn generate_lm_content(
    matching: &str,
    profile: &ProfileDto,
    offer_analysis: &str,
) -> Result<String, AppError> {
    let system_prompt = format!(
        "Tu rédiges le contenu d'une lettre de motivation.\n\n{RULES_HONNETETE}\n\
- Structure : accroche, 2-3 paragraphes argumentatifs, conclusion avec appel à l'action\n\
- Chaque argument doit s'appuyer sur une expérience ou un projet concret du profil\n\n\
Description des champs à renseigner :\n\
- opening : accroche — pourquoi ce poste, cette entreprise\n\
- body_paragraphs : pour chaque paragraphe, l'argument principal et l'expérience/le projet qui le soutient\n\
- closing : conclusion et appel à l'action\n\
- tone_notes : ton adopté et pourquoi"
    );

    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil: {e}")))?;

    let user_message = format!(
        "Analyse d'offre :\n{offer_analysis}\n\nStratégie de matching :\n{matching}\n\nProfil du candidat :\n{profile_json}"
    );

    let raw_response = call_claude(&system_prompt, &user_message, Some(lm_content_schema())).await?;

    let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "La rédaction de la lettre n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })?;

    serde_json::to_string(&parsed)
        .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")))
}
