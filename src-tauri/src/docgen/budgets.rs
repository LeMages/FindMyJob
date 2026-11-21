//! Budgets de longueur des documents générés, calculés depuis les valeurs
//! d'exemple de la référence (`contract.json`).
//!
//! Seule la prose rédigée a un budget : titre et résumé du CV, groupes de
//! compétences, objet et paragraphes de la lettre. Les expériences, projets
//! et formations viennent du profil tels quels : aucun budget ne les
//! réduit, même quand le document dépasse son nombre de pages.
//!
//! Règles :
//! - les nombres d'éléments (paragraphes, groupes de compétences) sont
//!   repris tels quels, sans tolérance ;
//! - les nombres de mots sont asymétriques : 10 % au-dessus de la valeur
//!   de l'exemple au plus (limite stricte, qui relance la rédaction : le
//!   document doit tenir dans ses pages), 30 % en dessous au plus (limite
//!   souple, signalée sans relance : un texte un peu court ne casse rien) ;
//! - le nombre de pages est celui du template rendu avec ses données
//!   d'exemple (plafond, jamais dépassé). Les parties fixes du document
//!   sont du texte du template : elles ne comptent dans aucun budget.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::docgen::words::count_words;
use crate::models::{CvData, LmData};

/// Dépassement toléré sur un nombre de mots, en pourcentage de la valeur
/// cible (limite stricte). Calculs en entiers : en flottant, 100 × 1,1
/// s'arrondit au-dessus à 111.
pub const LONGER_TOLERANCE_PERCENT: usize = 10;
/// Manque toléré sur un nombre de mots, en pourcentage de la valeur cible
/// (limite souple, sans relance).
pub const SHORTER_TOLERANCE_PERCENT: usize = 30;

/// `n` diminué de la tolérance basse, arrondi à l'entier inférieur.
fn lower_bound(n: usize) -> usize {
    n * (100 - SHORTER_TOLERANCE_PERCENT) / 100
}

/// `n` augmenté de la tolérance haute, arrondi à l'entier supérieur.
fn upper_bound(n: usize) -> usize {
    (n * (100 + LONGER_TOLERANCE_PERCENT)).div_ceil(100)
}

/// Plage de mots autorisée pour un champ.
///
/// # Attributes
///
/// * `target` - Nombre de mots visé (valeur de l'exemple).
/// * `min` - Minimum souple : en dessous, le texte est signalé sans relance.
/// * `max` - Maximum strict : au-dessus, la rédaction est relancée.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordRange {
    pub target: usize,
    pub min: usize,
    pub max: usize,
}

impl WordRange {
    /// Plage autour de `target` : de 30 % en dessous à 10 % au-dessus.
    pub fn around(target: usize) -> Self {
        Self { target, min: lower_bound(target), max: upper_bound(target) }
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
/// * `skills_group_count` - Nombre de groupes de compétences.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct CvBudget {
    pub page_count: usize,
    pub summary_words: WordRange,
    pub skills_group_count: usize,
}

impl CvBudget {
    /// Budget réduit d'un facteur (ex. `0.85`) quand le document dépasse son
    /// nombre de pages : seul le résumé raccourcit, le nombre de groupes de
    /// compétences ne change pas.
    pub fn scaled(&self, factor: f64) -> Self {
        Self {
            page_count: self.page_count,
            summary_words: self.summary_words.scaled(factor),
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

/// Calcule le budget d'un CV depuis le contenu de sa référence.
///
/// # Arguments
///
/// * `example` - Valeurs d'exemple de la référence (`contract.json`).
/// * `page_count` - Nombre de pages du template rendu avec cet exemple.
///
/// # Returns
///
/// `CvBudget`.
pub fn cv_budget(example: &CvData, page_count: usize) -> CvBudget {
    CvBudget {
        page_count,
        summary_words: WordRange::around(count_words(&example.summary)),
        skills_group_count: example.skills_groups.len(),
    }
}

/// Calcule le budget d'une lettre depuis le contenu de sa référence.
///
/// # Arguments
///
/// * `example` - Valeurs d'exemple de la référence (`contract.json`).
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
