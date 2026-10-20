//! Textes du profil tels qu'affichés par le CV : lignes de la description
//! des expériences, projets et formations, organisation d'un projet
//! (`pour_qui`), nom et niveau des langues, centres d'intérêt. Ce sont les
//! seuls textes du profil qu'une traduction peut modifier ; intitulés,
//! entreprises, écoles, lieux, stack et liens (noms officiels) sont
//! toujours lus tels quels dans le profil.

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::docgen::identity::{lines, text};
use crate::docgen::selection::Selection;
use crate::models::ProfileDto;

/// Lignes de description d'une expérience ou d'une formation.
///
/// # Attributes
///
/// * `id` - Identifiant de l'entrée dans le profil.
/// * `lines` - Lignes de la description, dans l'ordre.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryLines {
    pub id: i64,
    pub lines: Vec<String>,
}

/// Textes d'un projet.
///
/// # Attributes
///
/// * `id` - Identifiant du projet dans le profil.
/// * `pour_qui` - Organisation du projet (chaîne vide si absente).
/// * `lines` - Lignes de la description, dans l'ordre.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectTexts {
    pub id: i64,
    pub pour_qui: String,
    pub lines: Vec<String>,
}

/// Textes d'une langue parlée.
///
/// # Attributes
///
/// * `id` - Identifiant de la langue dans le profil.
/// * `nom` - Nom de la langue (« Anglais »).
/// * `level` - Niveau (chaîne vide si absent).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageTexts {
    pub id: i64,
    pub nom: String,
    pub level: String,
}

/// Textes du profil affichés par le CV.
///
/// # Attributes
///
/// * `experiences` - Toutes les expériences retenues.
/// * `projects` - Les projets retenus.
/// * `formations` - Toutes les formations du profil.
/// * `langues` - Les langues parlées du profil (nom non vide).
/// * `interets` - Les centres d'intérêt du profil, un par élément.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CvEntryTexts {
    pub experiences: Vec<EntryLines>,
    pub projects: Vec<ProjectTexts>,
    pub formations: Vec<EntryLines>,
    pub langues: Vec<LanguageTexts>,
    pub interets: Vec<String>,
}

impl CvEntryTexts {
    /// Lignes d'une expérience, vide si elle est absente.
    pub fn experience_lines(&self, id: i64) -> Vec<String> {
        self.experiences.iter().find(|e| e.id == id).map(|e| e.lines.clone()).unwrap_or_default()
    }

    /// Lignes d'une formation, vide si elle est absente.
    pub fn formation_lines(&self, id: i64) -> Vec<String> {
        self.formations.iter().find(|f| f.id == id).map(|f| f.lines.clone()).unwrap_or_default()
    }

    /// Textes d'un projet, `None` s'il est absent.
    pub fn project(&self, id: i64) -> Option<&ProjectTexts> {
        self.projects.iter().find(|p| p.id == id)
    }

    /// Indique si aucun texte n'est présent : rien à traduire.
    pub fn is_blank(&self) -> bool {
        self.experiences.iter().chain(&self.formations).all(|e| e.lines.is_empty())
            && self.projects.iter().all(|p| p.lines.is_empty() && p.pour_qui.is_empty())
            && self.langues.is_empty()
            && self.interets.is_empty()
    }
}

/// Textes des entrées dans la langue du profil.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat (formations).
/// * `selection` - Expériences et projets retenus.
///
/// # Returns
///
/// `CvEntryTexts`, une ligne de description = une ligne du profil (puces
/// de début de ligne retirées, lignes vides ignorées).
pub fn source_texts(profile: &ProfileDto, selection: &Selection) -> CvEntryTexts {
    CvEntryTexts {
        experiences: selection
            .experiences
            .iter()
            .map(|e| EntryLines { id: e.id, lines: lines(e.description.as_ref()) })
            .collect(),
        projects: selection
            .projects
            .iter()
            .map(|p| ProjectTexts { id: p.id, pour_qui: text(p.pour_qui.as_ref()), lines: lines(p.description.as_ref()) })
            .collect(),
        formations: profile
            .formations
            .iter()
            .map(|f| EntryLines { id: f.id, lines: lines(f.description.as_ref()) })
            .collect(),
        langues: profile
            .langues
            .iter()
            .filter(|l| !text(l.nom.as_ref()).is_empty())
            .map(|l| LanguageTexts { id: l.id, nom: text(l.nom.as_ref()), level: text(l.level.as_ref()) })
            .collect(),
        interets: lines(profile.user.interets.as_ref()),
    }
}
