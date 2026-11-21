//! Profil du candidat tel qu'il est sérialisé dans les prompts qui reçoivent
//! le profil entier (matching, messages de candidature). Les `*View` servent
//! aussi l'IPC vers le frontend, qui doit recevoir le contexte détaillé :
//! ce DTO dédié décide, appel par appel, de ce qui part au LLM.
//!
//! Les champs gardent le nom et l'ordre de ceux des `*View`, si bien qu'un
//! profil sans contexte détaillé produit exactement le même JSON qu'avant
//! l'ajout du contexte. Le contexte d'une expérience n'y figure jamais ;
//! celui d'un projet seulement en mode `ProjectContexts::Include`.

// Importation des crates externes
use chrono::NaiveDate;
use serde::Serialize;

// Importation des modules internes
use crate::error::AppError;
use crate::models::{ExperienceView, FormationView, LangView, ProfileDto, ProjetView, SkillView, UserView};

/// Ce que le prompt reçoit du contexte détaillé des projets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectContexts {
    /// Contexte des projets fourni (matching : aide au choix des projets).
    Include,
    /// Aucun contexte (messages de candidature).
    Exclude,
}

/// Expérience telle qu'envoyée au LLM : jamais de contexte détaillé.
#[derive(Serialize)]
struct PromptExperience<'a> {
    id: i64,
    poste: &'a str,
    entreprise: &'a Option<String>,
    lieu: &'a Option<String>,
    description: &'a Option<String>,
    date_debut: Option<NaiveDate>,
    date_fin: Option<NaiveDate>,
    type_contrat: &'a Option<String>,
}

/// Projet tel qu'envoyé au LLM ; `contexte` est omis quand il est absent ou
/// exclu, pour garder le JSON d'avant l'ajout du champ.
#[derive(Serialize)]
struct PromptProjet<'a> {
    id: i64,
    titre: &'a str,
    description: &'a Option<String>,
    stack: &'a Option<String>,
    url: &'a Option<String>,
    pour_qui: &'a Option<String>,
    date_debut: Option<NaiveDate>,
    date_fin: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contexte: Option<&'a str>,
}

/// Profil tel qu'envoyé au LLM, dans l'ordre des champs de `ProfileDto`.
#[derive(Serialize)]
struct PromptProfile<'a> {
    user: &'a UserView,
    experiences: Vec<PromptExperience<'a>>,
    formations: &'a [FormationView],
    projets: Vec<PromptProjet<'a>>,
    skills: &'a [SkillView],
    langues: &'a [LangView],
}

/// Expérience sans son contexte détaillé.
fn prompt_experience(e: &ExperienceView) -> PromptExperience<'_> {
    PromptExperience {
        id: e.id,
        poste: &e.poste,
        entreprise: &e.entreprise,
        lieu: &e.lieu,
        description: &e.description,
        date_debut: e.date_debut,
        date_fin: e.date_fin,
        type_contrat: &e.type_contrat,
    }
}

/// Projet, avec son contexte détaillé seulement si `contexts` l'inclut.
fn prompt_projet(p: &ProjetView, contexts: ProjectContexts) -> PromptProjet<'_> {
    let contexte = match contexts {
        ProjectContexts::Include => p.contexte.as_deref().map(str::trim).filter(|c| !c.is_empty()),
        ProjectContexts::Exclude => None,
    };
    PromptProjet {
        id: p.id,
        titre: &p.titre,
        description: &p.description,
        stack: &p.stack,
        url: &p.url,
        pour_qui: &p.pour_qui,
        date_debut: p.date_debut,
        date_fin: p.date_fin,
        contexte,
    }
}

/// Sérialise le profil pour un prompt.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `contexts` - Inclusion du contexte détaillé des projets.
///
/// # Returns
///
/// JSON compact du profil, sans aucun contexte d'expérience, ou
/// `AppError::Llm` si la sérialisation échoue.
pub fn profile_prompt_json(profile: &ProfileDto, contexts: ProjectContexts) -> Result<String, AppError> {
    let prompt = PromptProfile {
        user: &profile.user,
        experiences: profile.experiences.iter().map(prompt_experience).collect(),
        formations: &profile.formations,
        projets: profile.projets.iter().map(|p| prompt_projet(p, contexts)).collect(),
        skills: &profile.skills,
        langues: &profile.langues,
    };
    serde_json::to_string(&prompt).map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil : {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::test_profile::{experience, profile_with, projet};

    #[test]
    fn sans_contexte_json_identique_a_celui_du_profil() {
        let profile = profile_with(vec![experience(1, None)], vec![projet(2, None)]);
        // Le JSON d'avant l'ajout du champ : celui du profil, sans les `contexte` nuls.
        let before = serde_json::to_string(&profile).expect("profil").replace(r#","contexte":null"#, "");
        for contexts in [ProjectContexts::Include, ProjectContexts::Exclude] {
            assert_eq!(profile_prompt_json(&profile, contexts).expect("prompt"), before);
        }
    }

    #[test]
    fn contexte_des_projets_seul_et_seulement_si_inclus() {
        let profile = profile_with(
            vec![experience(1, Some("CTX-EXPERIENCE"))],
            vec![projet(2, Some("  CTX-PROJET  ")), projet(3, Some("   "))],
        );
        let included = profile_prompt_json(&profile, ProjectContexts::Include).expect("prompt");
        assert!(included.contains(r#""contexte":"CTX-PROJET""#), "{included}");
        assert_eq!(included.matches("contexte").count(), 1, "contexte blanc omis : {included}");
        assert!(!included.contains("CTX-EXPERIENCE"), "jamais le contexte d'une expérience : {included}");

        let excluded = profile_prompt_json(&profile, ProjectContexts::Exclude).expect("prompt");
        assert!(!excluded.contains("contexte"), "{excluded}");
    }
}
