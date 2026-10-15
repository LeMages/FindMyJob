//! Résultat du matching profil / offre (étape 2 du pipeline), typé.
//!
//! Le matching choisit les projets à afficher et oriente le résumé ; il ne
//! produit aucun texte repris dans les documents. `match_type` sert à
//! choisir quoi dire à la rédaction.

// Importation des crates externes
use serde::{Deserialize, Serialize};

/// Compétence du profil qui répond à l'offre.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MatchedSkill {
    pub skill: String,
    pub match_type: String,
}

/// Stratégie de matching produite par le LLM.
///
/// # Attributes
///
/// * `project_ids` - Projets retenus, du plus au moins pertinent. Absent de
///   la réponse (liste vide) quand aucun projet n'est à choisir.
/// * `relevant_skills` - Compétences du profil qui répondent à l'offre.
/// * `summary_strategy` - Angle du résumé et de l'accroche.
/// * `gaps` - Compétences demandées absentes du profil (jamais à inventer).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MatchingResult {
    #[serde(default)]
    pub project_ids: Vec<i64>,
    pub relevant_skills: Vec<MatchedSkill>,
    pub summary_strategy: String,
    pub gaps: Vec<String>,
}
