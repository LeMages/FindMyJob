//! Mise en correspondance du profil candidat avec une offre analysée, via
//! l'API Claude (étape 2 du pipeline de génération de documents). Choisit
//! les N projets les plus pertinents (par identifiant, validés en Rust) et
//! les compétences qui répondent à l'offre. Le matching ne rédige aucun
//! texte repris dans les documents.

// Importation des crates externes
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::selection::project_id_problems;
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::object;
use crate::llm::target::ClaudeTarget;
use crate::llm::validated_call::call_until_valid;
use crate::llm::writing_loop::WritingFeedback;
use crate::llm::writing_rules::with_feedback;
use crate::models::{MatchingResult, ProfileDto};

/// Schéma JSON attendu pour la stratégie de matching, utilisé comme
/// structured output (`output_config.format`).
///
/// `project_ids` n'existe que s'il y a des projets à choisir : une `enum`
/// vide n'est pas un schéma valide. Le nombre d'éléments n'est pas
/// contraint par le schéma (les structured outputs ne bornent pas une
/// liste au-delà de 1) : il est vérifié en Rust.
pub fn matching_schema(profile: &ProfileDto, expected: usize) -> Value {
    let skill = object(&[
        ("skill", json!({ "type": "string" })),
        ("match_type", json!({ "type": "string", "enum": ["exact", "related", "transferable"] })),
    ]);
    let mut props = Vec::new();
    if expected > 0 {
        let ids: Vec<i64> = profile.projets.iter().map(|p| p.id).collect();
        props.push(("project_ids", json!({ "type": "array", "items": { "type": "integer", "enum": ids } })));
    }
    props.extend([
        ("relevant_skills", json!({ "type": "array", "items": skill })),
        ("summary_strategy", json!({ "type": "string" })),
        ("gaps", json!({ "type": "array", "items": { "type": "string" } })),
    ]);
    object(&props)
}

/// Prompt système du matching.
fn system_prompt(expected: usize) -> String {
    let projects = if expected > 0 {
        format!(
            "- project_ids : les identifiants des {expected} projets du profil les plus pertinents pour l'offre, exactement {expected}, chacun une seule fois, du plus pertinent au moins pertinent.\n"
        )
    } else {
        String::new()
    };
    format!(
        "Tu es un stratège de candidature. Tu compares le profil du candidat à l'offre et tu décides quoi mettre en avant.\n\n\
Description des champs à renseigner :\n\
{projects}\
- relevant_skills : les compétences du profil qui répondent à l'offre, avec le type de correspondance (exact pour une compétence identique, related pour une compétence proche, transferable pour une compétence transférable).\n\
- summary_strategy : en deux ou trois phrases, l'angle à donner au résumé et à l'accroche pour cette offre.\n\
- gaps : les compétences demandées par l'offre que le profil n'a pas. Elles servent à savoir quoi ne jamais affirmer.\n\n\
Tout ce que tu retiens vient du profil : n'invente rien."
    )
}

/// Appelle le LLM une fois.
async fn call_once(
    system_prompt: &str,
    user_message: &str,
    schema: &Value,
    feedback: Option<WritingFeedback>,
    target: &ClaudeTarget,
) -> Result<MatchingResult, AppError> {
    let message = with_feedback(user_message.to_string(), feedback.as_ref());
    let raw = call_claude("matching", system_prompt, &message, Some(schema.clone()), target).await?;
    serde_json::from_str(raw.trim()).map_err(|e| {
        AppError::Llm(format!("Le matching n'a pas retourné le JSON attendu : {e}\nRéponse brute : {raw}"))
    })
}

/// Met en correspondance le profil du candidat avec une offre analysée, via
/// l'API Claude, pour choisir les projets et l'angle du CV et de la lettre.
///
/// # Arguments
///
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `profile` - Profil complet du candidat.
/// * `expected` - Nombre de projets à choisir
///   (`docgen::selection::expected_project_count`).
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// `MatchingResult` dont `project_ids` est valide (projets du profil, sans
/// doublon, exactement `expected`), ou `AppError::Llm` après 3 essais
/// invalides.
pub async fn match_profile(
    offer_analysis: &str,
    profile: &ProfileDto,
    expected: usize,
    target: &ClaudeTarget,
) -> Result<MatchingResult, AppError> {
    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil : {e}")))?;
    let user_message = format!("Analyse d'offre :\n{offer_analysis}\n\nProfil du candidat :\n{profile_json}");
    let (system_prompt, schema) = (system_prompt(expected), matching_schema(profile, expected));

    call_until_valid(
        "Le matching",
        |feedback| call_once(&system_prompt, &user_message, &schema, feedback, target),
        |result| project_id_problems(&result.project_ids, profile, expected),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProjetView, UserView};

    /// Profil fictif avec les projets d'identifiants donnés.
    fn profile(project_ids: &[i64]) -> ProfileDto {
        let projet = |id: i64| ProjetView {
            id,
            titre: format!("Projet {id}"),
            description: None,
            stack: None,
            url: None,
            pour_qui: None,
            date_debut: None,
            date_fin: None,
        };
        ProfileDto {
            user: UserView {
                id: 1,
                nom: None,
                prenom: None,
                email: None,
                telephone: None,
                ville: None,
                zip_code: None,
                permis: None,
                description: None,
                linkedin: None,
                github: None,
                portfolio: None,
                interets: None,
                langue_profil: "fr".to_string(),
                created_at: chrono::Utc::now(),
            },
            experiences: Vec::new(),
            formations: Vec::new(),
            projets: project_ids.iter().map(|id| projet(*id)).collect(),
            skills: Vec::new(),
            langues: Vec::new(),
        }
    }

    #[test]
    fn project_ids_limites_aux_projets_du_profil() {
        let schema = matching_schema(&profile(&[3, 7]), 2);
        let ids = &schema["properties"]["project_ids"];
        assert_eq!(ids["items"]["enum"], json!([3, 7]));
        assert_eq!(ids["items"]["type"], "integer");
        assert!(schema["required"].as_array().is_some_and(|r| r.contains(&json!("project_ids"))));
        assert!(schema["properties"].get("relevant_experiences").is_none());
        assert!(system_prompt(2).contains("exactement 2"));
    }

    #[test]
    fn pas_de_project_ids_sans_projet_a_choisir() {
        for (ids, expected) in [(&[][..], 0), (&[3, 7][..], 0)] {
            let schema = matching_schema(&profile(ids), expected);
            assert!(schema["properties"].get("project_ids").is_none(), "pas d'enum vide");
            assert!(!system_prompt(expected).contains("project_ids"));
        }
    }

    #[test]
    fn reponse_sans_project_ids_donne_une_selection_vide() {
        let raw = r#"{"relevant_skills": [], "summary_strategy": "Angle.", "gaps": []}"#;
        let result: MatchingResult = serde_json::from_str(raw).expect("réponse sans projets");
        assert!(result.project_ids.is_empty());
        assert!(project_id_problems(&result.project_ids, &profile(&[]), 0).is_empty());
    }
}
