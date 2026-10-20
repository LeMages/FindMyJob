//! Budgets de longueur des documents générés, calculés depuis le contenu
//! de la référence (`example_data.json`).
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
//!   d'exemple (plafond, jamais dépassé) ;
//! - les mots des zones `[FIXE]` d'un champ ne sont jamais réduits quand
//!   le document dépasse son nombre de pages : seule la partie rédigée
//!   raccourcit.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::docgen::words::count_words;
use crate::models::{CvData, LmData, ProseGuidance};

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
/// * `fixed` - Mots recopiés de zones `[FIXE]`, jamais réduits.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordRange {
    pub target: usize,
    pub min: usize,
    pub max: usize,
    pub fixed: usize,
}

impl WordRange {
    /// Plage autour de `target` : de 30 % en dessous à 10 % au-dessus.
    pub fn around(target: usize) -> Self {
        Self { target, min: lower_bound(target), max: upper_bound(target), fixed: 0 }
    }

    /// Indique si un nombre de mots est dans la plage.
    pub fn contains(&self, words: usize) -> bool {
        (self.min..=self.max).contains(&words)
    }

    /// Même plage, dont `fixed` mots viennent de zones fixes (bornés par la
    /// cible).
    pub fn with_fixed(self, fixed: usize) -> Self {
        Self { fixed: fixed.min(self.target), ..self }
    }

    /// Plage réduite d'un facteur (ex. `0.85`), bornes comprises : seuls
    /// les mots au-delà de `fixed` sont réduits.
    pub fn scaled(&self, factor: f64) -> Self {
        let fixed = self.fixed;
        let scale = |n: usize| match n.checked_sub(fixed) {
            Some(free) => fixed + ((free as f64) * factor).round() as usize,
            None => n,
        };
        Self { target: scale(self.target), min: scale(self.min), max: scale(self.max).max(1), fixed }
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
    /// Même budget, avec les mots des zones fixes du résumé (voir
    /// `WordRange::with_fixed`).
    ///
    /// # Arguments
    ///
    /// * `guidance` - Consignes de la référence.
    ///
    /// # Returns
    ///
    /// Budget dont les zones fixes ne seront jamais réduites.
    pub fn with_fixed_words(mut self, guidance: &ProseGuidance) -> Self {
        self.summary_words = self.summary_words.with_fixed(fixed_words(guidance, "summary"));
        self
    }

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

    /// Même budget, avec les mots des zones fixes de l'objet et de chaque
    /// paragraphe (voir `WordRange::with_fixed`).
    ///
    /// # Arguments
    ///
    /// * `guidance` - Consignes de la référence.
    ///
    /// # Returns
    ///
    /// Budget dont les zones fixes ne seront jamais réduites.
    pub fn with_fixed_words(mut self, guidance: &ProseGuidance) -> Self {
        self.subject_words = self.subject_words.with_fixed(fixed_words(guidance, "subject"));
        for (i, range) in self.paragraph_words.iter_mut().enumerate() {
            *range = range.with_fixed(fixed_words(guidance, &format!("paragraphs[{i}]")));
        }
        self
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

/// Nombre de mots des zones fixes d'un champ de prose.
fn fixed_words(guidance: &ProseGuidance, field: &str) -> usize {
    guidance.fixed_texts(field).map(count_words).sum()
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
