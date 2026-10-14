//! Sélection des entrées du profil à faire figurer dans les documents, à
//! partir du matching : toutes les expériences, et les N projets les plus
//! pertinents (N = nombre de projets de la référence).

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::models::{ExperienceView, MatchedItem, MatchingResult, ProfileDto, ProjetView};

/// Entrée retenue, avec ce que le matching en dit.
///
/// # Attributes
///
/// * `item` - Entrée du profil.
/// * `notes` - Jugement du matching, `None` si le matching l'a omise.
#[derive(Serialize, Clone, Debug)]
pub struct Selected<T> {
    pub item: T,
    pub notes: Option<MatchedItem>,
}

/// Entrées retenues, dans l'ordre d'affichage du profil (le plus récent en
/// premier).
///
/// # Attributes
///
/// * `experiences` - Toutes les expériences du profil.
/// * `projects` - Les projets retenus.
#[derive(Serialize, Clone, Debug)]
pub struct Selection {
    pub experiences: Vec<Selected<ExperienceView>>,
    pub projects: Vec<Selected<ProjetView>>,
}

impl Selection {
    /// Identifiants des expériences retenues, dans l'ordre d'affichage.
    pub fn experience_ids(&self) -> Vec<i64> {
        self.experiences.iter().map(|s| s.item.id).collect()
    }

    /// Identifiants des projets retenus, dans l'ordre d'affichage.
    pub fn project_ids(&self) -> Vec<i64> {
        self.projects.iter().map(|s| s.item.id).collect()
    }
}

/// Construit la sélection.
///
/// Les projets retenus sont les `project_count` premiers du matching dont
/// l'identifiant existe dans le profil ; s'il en manque, la sélection est
/// complétée par les projets les plus récents du profil non encore retenus.
/// Les entrées retenues gardent l'ordre du profil.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `matching` - Résultat du matching.
/// * `project_count` - Nombre de projets à retenir.
///
/// # Returns
///
/// `Selection`.
pub fn select(profile: &ProfileDto, matching: &MatchingResult, project_count: usize) -> Selection {
    let notes_for = |notes: &[MatchedItem], id: i64| notes.iter().find(|n| n.id == id).cloned();

    let experiences = profile
        .experiences
        .iter()
        .map(|e| Selected { item: e.clone(), notes: notes_for(&matching.relevant_experiences, e.id) })
        .collect();

    // 1. Projets désignés par le matching, dans son ordre de pertinence
    let mut chosen: Vec<i64> = Vec::new();
    for matched in &matching.relevant_projects {
        let known = profile.projets.iter().any(|p| p.id == matched.id);
        if known && !chosen.contains(&matched.id) && chosen.len() < project_count {
            chosen.push(matched.id);
        }
    }
    // 2. Complément par les projets les plus récents du profil
    for projet in &profile.projets {
        if chosen.len() >= project_count {
            break;
        }
        if !chosen.contains(&projet.id) {
            chosen.push(projet.id);
        }
    }

    let projects = profile
        .projets
        .iter()
        .filter(|p| chosen.contains(&p.id))
        .map(|p| Selected { item: p.clone(), notes: notes_for(&matching.relevant_projects, p.id) })
        .collect();

    Selection { experiences, projects }
}
