//! Résultat du matching profil / offre (étape 2 du pipeline), typé.
//!
//! Les champs `highlights`, `keywords_covered` et `match_type` servent à
//! choisir quoi dire à la rédaction ; ils ne sont pas du texte à reprendre.

// Importation des crates externes
use serde::{Deserialize, Serialize};

/// Entrée du profil (expérience ou projet) jugée par le matching.
///
/// # Attributes
///
/// * `id` - Identifiant de l'entrée dans le profil.
/// * `relevance` - `"high"`, `"medium"` ou `"low"`.
/// * `highlights` - Aspects du profil à mettre en avant pour cette offre.
/// * `keywords_covered` - Mots-clés de l'offre auxquels l'entrée répond.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MatchedItem {
    pub id: i64,
    pub relevance: String,
    pub highlights: Vec<String>,
    #[serde(default)]
    pub keywords_covered: Vec<String>,
}

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
/// * `relevant_experiences` - Toutes les expériences du profil, jugées.
/// * `relevant_projects` - Les projets retenus, du plus au moins pertinent.
/// * `relevant_skills` - Compétences du profil qui répondent à l'offre.
/// * `summary_strategy` - Angle du résumé et de l'accroche.
/// * `gaps` - Compétences demandées absentes du profil (jamais à inventer).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MatchingResult {
    pub relevant_experiences: Vec<MatchedItem>,
    pub relevant_projects: Vec<MatchedItem>,
    pub relevant_skills: Vec<MatchedSkill>,
    pub summary_strategy: String,
    pub gaps: Vec<String>,
}
