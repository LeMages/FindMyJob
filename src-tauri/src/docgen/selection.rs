//! Sélection des entrées du profil à faire figurer dans les documents :
//! toutes les expériences, et les projets choisis par le matching (autant
//! que la référence en affiche, dans la limite de ceux du profil). Le
//! choix du matching est validé ici avant d'être utilisé.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::models::{ExperienceView, ProfileDto, ProjetView};

/// Entrées retenues, dans l'ordre d'affichage du profil (le plus récent en
/// premier).
///
/// # Attributes
///
/// * `experiences` - Toutes les expériences du profil.
/// * `projects` - Les projets retenus.
#[derive(Serialize, Clone, Debug)]
pub struct Selection {
    pub experiences: Vec<ExperienceView>,
    pub projects: Vec<ProjetView>,
}

impl Selection {
    /// Identifiants des expériences retenues, dans l'ordre d'affichage.
    pub fn experience_ids(&self) -> Vec<i64> {
        self.experiences.iter().map(|e| e.id).collect()
    }

    /// Identifiants des projets retenus, dans l'ordre d'affichage.
    pub fn project_ids(&self) -> Vec<i64> {
        self.projects.iter().map(|p| p.id).collect()
    }
}

/// Nombre de projets que le matching doit choisir : celui de la référence,
/// sans dépasser le nombre de projets du profil.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `reference_count` - Nombre de projets affichés par la référence.
///
/// # Returns
///
/// `min(reference_count, nombre de projets du profil)`.
pub fn expected_project_count(profile: &ProfileDto, reference_count: usize) -> usize {
    reference_count.min(profile.projets.len())
}

/// Vérifie les projets choisis par le matching : identifiants du profil,
/// sans doublon, en nombre exact.
///
/// # Arguments
///
/// * `ids` - Identifiants rendus par le matching.
/// * `profile` - Profil complet du candidat.
/// * `expected` - Nombre de projets attendu (`expected_project_count`).
///
/// # Returns
///
/// Problèmes relevés, formulés pour être renvoyés au LLM ; vide si le choix
/// est valide.
pub fn project_id_problems(ids: &[i64], profile: &ProfileDto, expected: usize) -> Vec<String> {
    let known: Vec<i64> = profile.projets.iter().map(|p| p.id).collect();
    let mut problems = Vec::new();
    for (rank, id) in ids.iter().enumerate() {
        if !known.contains(id) {
            problems.push(format!("project_ids : l'id {id} n'est pas un projet du profil (ids possibles : {known:?})."));
        } else if ids[..rank].contains(id) {
            problems.push(format!("project_ids : l'id {id} apparaît plusieurs fois."));
        }
    }
    if ids.len() != expected {
        problems.push(format!("project_ids contient {} projets, attendu exactement {expected}.", ids.len()));
    }
    problems
}

/// Construit la sélection.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `project_ids` - Projets retenus, déjà validés (`project_id_problems`).
///
/// # Returns
///
/// `Selection`, entrées dans l'ordre du profil.
pub fn select(profile: &ProfileDto, project_ids: &[i64]) -> Selection {
    let projects = profile.projets.iter().filter(|p| project_ids.contains(&p.id)).cloned().collect();
    Selection { experiences: profile.experiences.clone(), projects }
}
