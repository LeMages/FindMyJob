//! Budgets de longueur des documents générés, calculés depuis le contenu
//! de la référence (`example_data.json`).
//!
//! Règles :
//! - les nombres d'éléments (paragraphes, projets, groupes de compétences,
//!   bullets par entrée) sont repris tels quels, sans tolérance ;
//! - les nombres de mots acceptent 10 % d'écart autour de la valeur de
//!   l'exemple ;
//! - le nombre de pages est celui du template rendu avec ses données
//!   d'exemple (plafond, jamais dépassé).

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::docgen::words::count_words;
use crate::models::{CvData, LmData};

/// Écart toléré sur un nombre de mots, en pourcentage de la valeur cible.
/// Calculs en entiers : en flottant, 100 × 1,1 s'arrondit au-dessus à 111.
pub const WORD_TOLERANCE_PERCENT: usize = 10;

/// `n` diminué de la tolérance, arrondi à l'entier inférieur.
fn lower_bound(n: usize) -> usize {
    n * (100 - WORD_TOLERANCE_PERCENT) / 100
}

/// `n` augmenté de la tolérance, arrondi à l'entier supérieur.
fn upper_bound(n: usize) -> usize {
    (n * (100 + WORD_TOLERANCE_PERCENT)).div_ceil(100)
}

/// Nombre de bullets par entrée utilisé quand l'exemple n'a aucune entrée
/// de ce type (aucune valeur de rang à reprendre).
const FALLBACK_BULLETS: usize = 2;

/// Plage de mots autorisée pour un champ.
///
/// # Attributes
///
/// * `target` - Nombre de mots visé (valeur de l'exemple).
/// * `min` - Minimum accepté.
/// * `max` - Maximum accepté.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordRange {
    pub target: usize,
    pub min: usize,
    pub max: usize,
}

impl WordRange {
    /// Plage centrée sur `target`, à ±10 %.
    pub fn around(target: usize) -> Self {
        Self { target, min: lower_bound(target), max: upper_bound(target) }
    }

    /// Plage entre deux bornes d'exemple : de 10 % sous `low` à 10 % au-dessus
    /// de `high`, avec `target` comme valeur visée.
    pub fn spanning(low: usize, target: usize, high: usize) -> Self {
        Self { target, min: lower_bound(low), max: upper_bound(high) }
    }

    /// Indique si un nombre de mots est dans la plage.
    pub fn contains(&self, words: usize) -> bool {
        (self.min..=self.max).contains(&words)
    }

    /// Plage réduite d'un facteur (ex. `0.85`), bornes comprises.
    pub fn scaled(&self, factor: f64) -> Self {
        let scale = |n: usize| ((n as f64) * factor).round() as usize;
        Self { target: scale(self.target), min: scale(self.min), max: scale(self.max).max(1) }
    }
}

/// Budget d'un CV.
///
/// # Attributes
///
/// * `page_count` - Nombre de pages maximal.
/// * `summary_words` - Plage de mots du résumé.
/// * `experience_bullets` - Bullets de chaque expérience de l'exemple, par rang.
/// * `project_bullets` - Bullets de chaque projet de l'exemple, par rang.
/// * `bullet_words` - Plage de mots d'un bullet (bornes = plus court et plus
///   long bullet de l'exemple, ±10 % ; cible = moyenne).
/// * `project_count` - Nombre de projets.
/// * `skills_group_count` - Nombre de groupes de compétences.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct CvBudget {
    pub page_count: usize,
    pub summary_words: WordRange,
    pub experience_bullets: Vec<usize>,
    pub project_bullets: Vec<usize>,
    pub bullet_words: WordRange,
    pub project_count: usize,
    pub skills_group_count: usize,
}

impl CvBudget {
    /// Nombre de bullets attendu pour l'expérience de rang `rank` (0-based).
    pub fn experience_bullets_at(&self, rank: usize) -> usize {
        per_rank(&self.experience_bullets, rank)
    }

    /// Nombre de bullets attendu pour le projet de rang `rank` (0-based).
    pub fn project_bullets_at(&self, rank: usize) -> usize {
        per_rank(&self.project_bullets, rank)
    }

    /// Budget réduit d'un facteur (ex. `0.85`) quand le document dépasse son
    /// nombre de pages : mots et bullets par entrée diminuent (au moins un
    /// bullet), la structure (nombre de projets, de groupes) ne change pas.
    pub fn scaled(&self, factor: f64) -> Self {
        let scale_count = |n: &usize| (((*n as f64) * factor).round() as usize).max(1);
        Self {
            page_count: self.page_count,
            summary_words: self.summary_words.scaled(factor),
            experience_bullets: self.experience_bullets.iter().map(scale_count).collect(),
            project_bullets: self.project_bullets.iter().map(scale_count).collect(),
            bullet_words: self.bullet_words.scaled(factor),
            project_count: self.project_count,
            skills_group_count: self.skills_group_count,
        }
    }
}

/// Budget d'une lettre de motivation.
///
/// # Attributes
///
/// * `page_count` - Nombre de pages maximal.
/// * `paragraph_words` - Plage de mots de chaque paragraphe, par rang ; sa
///   longueur est le nombre de paragraphes.
/// * `subject_words` - Plage de mots de l'objet.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct LmBudget {
    pub page_count: usize,
    pub paragraph_words: Vec<WordRange>,
    pub subject_words: WordRange,
}

impl LmBudget {
    /// Nombre de paragraphes attendu.
    pub fn paragraph_count(&self) -> usize {
        self.paragraph_words.len()
    }

    /// Budget réduit d'un facteur (ex. `0.85`) : les mots diminuent, le
    /// nombre de paragraphes ne change pas.
    pub fn scaled(&self, factor: f64) -> Self {
        Self {
            page_count: self.page_count,
            paragraph_words: self.paragraph_words.iter().map(|r| r.scaled(factor)).collect(),
            subject_words: self.subject_words.scaled(factor),
        }
    }
}

/// Valeur de rang `rank`, ou la dernière valeur si la liste est plus courte.
fn per_rank(values: &[usize], rank: usize) -> usize {
    values.get(rank).or(values.last()).copied().unwrap_or(FALLBACK_BULLETS)
}

/// Calcule le budget d'un CV depuis le contenu de sa référence.
///
/// # Arguments
///
/// * `example` - Contenu de la référence (`example_data.json`).
/// * `page_count` - Nombre de pages du template rendu avec cet exemple.
///
/// # Returns
///
/// `CvBudget`.
pub fn cv_budget(example: &CvData, page_count: usize) -> CvBudget {
    let bullets: Vec<usize> = example
        .experiences
        .iter()
        .flat_map(|e| e.bullets.iter())
        .chain(example.projects.iter().flat_map(|p| p.bullets.iter()))
        .map(|b| count_words(b))
        .collect();
    let bullet_words = match (bullets.iter().min(), bullets.iter().max()) {
        (Some(&low), Some(&high)) => {
            let mean = (bullets.iter().sum::<usize>() as f64 / bullets.len() as f64).round() as usize;
            WordRange::spanning(low, mean, high)
        }
        _ => WordRange::around(0),
    };

    CvBudget {
        page_count,
        summary_words: WordRange::around(count_words(&example.summary)),
        experience_bullets: example.experiences.iter().map(|e| e.bullets.len()).collect(),
        project_bullets: example.projects.iter().map(|p| p.bullets.len()).collect(),
        bullet_words,
        project_count: example.projects.len(),
        skills_group_count: example.skills_groups.len(),
    }
}

/// Calcule le budget d'une lettre depuis le contenu de sa référence.
///
/// # Arguments
///
/// * `example` - Contenu de la référence (`example_data.json`).
/// * `page_count` - Nombre de pages du template rendu avec cet exemple.
///
/// # Returns
///
/// `LmBudget`.
pub fn lm_budget(example: &LmData, page_count: usize) -> LmBudget {
    LmBudget {
        page_count,
        paragraph_words: example.paragraphs.iter().map(|p| WordRange::around(count_words(p))).collect(),
        subject_words: WordRange::around(count_words(&example.subject)),
    }
}
