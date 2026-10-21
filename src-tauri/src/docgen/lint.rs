//! Linter de la prose générée : caractères interdits, formules interdites
//! et respect des budgets de longueur. Fonctions pures, sans LLM.
//!
//! Le linter ne corrige rien, sauf `strip_dashes`, appliqué en dernier
//! recours après le dernier essai de rédaction.
//!
//! Aucune marque n'est jamais autorisée, même si la référence l'emploie.
//! Une formule interdite que la prose de la référence emploie elle-même
//! est autorisée : la référence est la source de vérité du style.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::docgen::budgets::{CvBudget, LmBudget, WordRange};
use crate::docgen::dash_normalization::{dash_marks, normalize_dashes, DashContext};
use crate::docgen::forbidden_phrases::{FORBIDDEN_PHRASES_EN, FORBIDDEN_PHRASES_FR};
use crate::docgen::words::count_words;
use crate::models::{CvContent, LmContent, ProseField};

/// Marques Markdown interdites dans la prose, avec leur description. Les
/// tirets interdits sont relevés par `dash_marks`.
const FORBIDDEN_MARKDOWN: [(&str, &str); 2] = [("**", "gras Markdown (**)"), ("`", "backtick (`)")];

/// Violation relevée par le linter.
///
/// # Attributes
///
/// * `field` - Chemin du champ concerné (ex. `paragraphs[2]`).
/// * `message` - Description de la violation, formulée comme une consigne.
/// * `advisory` - Vrai pour un écart signalé sans relance (texte un peu
///   court) : il ne justifie pas un nouvel appel au LLM.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Violation {
    pub field: String,
    pub message: String,
    pub advisory: bool,
}

impl Violation {
    /// Violation qui justifie une relance.
    ///
    /// # Arguments
    ///
    /// * `field` - Chemin du champ concerné.
    /// * `message` - Description, formulée comme une consigne.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self { field: field.into(), message: message.into(), advisory: false }
    }

    /// Écart signalé sans relance.
    fn advisory(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self { field: field.into(), message: message.into(), advisory: true }
    }
}

/// Violations des marques interdites (tirets, Markdown) d'un champ ; un
/// tiret est signalé avec le substitut à employer.
fn mark_violations(field: &ProseField) -> Vec<Violation> {
    let dashes = dash_marks(&field.text)
        .into_iter()
        .map(|mark| format!("contient {}, interdit : {}", mark.label(), mark.substitute()));
    let markdown = FORBIDDEN_MARKDOWN
        .iter()
        .filter(|(mark, _)| field.text.contains(mark))
        .map(|(_, label)| format!("contient un {label}, interdit"));
    dashes.chain(markdown).map(|message| Violation::new(&field.path, message)).collect()
}

/// Vérifie les caractères et formules interdits dans chaque champ de prose.
///
/// # Arguments
///
/// * `fields` - Champs de prose du contenu rédigé.
/// * `example_prose` - Prose de la référence : une formule qui y figure
///   est autorisée (jamais une marque).
///
/// # Returns
///
/// Violations relevées, dans l'ordre des champs.
pub fn lint_prose(fields: &[ProseField], example_prose: &str) -> Vec<Violation> {
    let example = example_prose.to_lowercase();
    let allowed_phrases: Vec<&str> = FORBIDDEN_PHRASES_FR
        .iter()
        .chain(FORBIDDEN_PHRASES_EN)
        .copied()
        .filter(|phrase| !example.contains(phrase))
        .collect();

    let mut violations = Vec::new();
    for field in fields {
        violations.extend(mark_violations(field));
        let lowered = field.text.to_lowercase();
        for phrase in &allowed_phrases {
            if lowered.contains(phrase) {
                violations.push(Violation::new(&field.path, format!("contient la formule interdite « {phrase} »")));
            }
        }
    }
    violations
}

/// Vérifie une plage de mots : trop long est une violation qui relance,
/// trop court un écart signalé sans relance.
fn check_words(field: &str, text: &str, range: &WordRange) -> Option<Violation> {
    let words = count_words(text);
    if words > range.max {
        return Some(Violation::new(
            field,
            format!("fait {words} mots, au-delà du maximum de {} mots : raccourcis-le", range.max),
        ));
    }
    (words < range.min).then(|| {
        Violation::advisory(field, format!("fait {words} mots, sous les {} mots visés (toléré)", range.target))
    })
}

/// Vérifie un nombre d'éléments exact.
fn check_count(field: &str, actual: usize, expected: usize, unit: &str) -> Option<Violation> {
    (actual != expected).then(|| Violation::new(field, format!("contient {actual} {unit}, attendu exactement {expected}")))
}

/// Vérifie le contenu d'un CV contre son budget.
///
/// # Arguments
///
/// * `content` - Contenu rédigé.
/// * `budget` - Budget du CV.
///
/// # Returns
///
/// Violations relevées.
pub fn lint_cv_budget(content: &CvContent, budget: &CvBudget) -> Vec<Violation> {
    let mut violations = Vec::new();
    violations.extend(check_words("summary", &content.summary, &budget.summary_words));
    violations.extend(check_count(
        "skills_groups",
        content.skills_groups.len(),
        budget.skills_group_count,
        "groupes",
    ));
    violations
}

/// Vérifie le contenu d'une lettre contre son budget.
///
/// # Arguments
///
/// * `content` - Contenu rédigé.
/// * `budget` - Budget de la lettre.
///
/// # Returns
///
/// Violations relevées.
pub fn lint_lm_budget(content: &LmContent, budget: &LmBudget) -> Vec<Violation> {
    let mut violations = Vec::new();
    violations.extend(check_words("subject", &content.subject, &budget.subject_words));
    violations.extend(check_count(
        "paragraphs",
        content.paragraphs.len(),
        budget.paragraph_count(),
        "paragraphes",
    ));
    for (i, (paragraph, range)) in content.paragraphs.iter().zip(&budget.paragraph_words).enumerate() {
        violations.extend(check_words(&format!("paragraphs[{i}]"), paragraph, range));
    }
    violations
}

/// Remplace mécaniquement les tirets interdits et retire les marques
/// Markdown d'un texte : une incise entre tirets devient une incise entre
/// virgules, un tiret collé devient un trait d'union (normalisation de
/// `docgen::dash_normalization`, en mode paragraphe).
///
/// # Arguments
///
/// * `text` - Texte à nettoyer.
///
/// # Returns
///
/// Texte sans « — », « – », « --- », « -- », gras Markdown ni backtick.
pub fn strip_dashes(text: &str) -> String {
    let mut cleaned = text.to_string();
    for (mark, _) in FORBIDDEN_MARKDOWN {
        cleaned = cleaned.replace(mark, "");
    }
    normalize_dashes(&cleaned, DashContext::Paragraph)
}
