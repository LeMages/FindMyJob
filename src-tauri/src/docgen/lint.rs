//! Linter de la prose générée : caractères interdits, formules interdites
//! et respect des budgets de longueur. Fonctions pures, sans LLM.
//!
//! Le linter ne corrige rien, sauf `strip_dashes` / `strip_forbidden_marks`,
//! appliqués en dernier recours après le dernier essai de rédaction.
//!
//! Une marque ou une formule que la prose de la référence emploie elle-même
//! est autorisée : la référence est la source de vérité du style.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::docgen::budgets::{CvBudget, LmBudget, WordRange};
use crate::docgen::forbidden_phrases::{FORBIDDEN_PHRASES_EN, FORBIDDEN_PHRASES_FR};
use crate::docgen::words::count_words;
use crate::models::{CvContent, LmContent, ProseField};

/// Caractères et marques interdits dans la prose, avec leur description.
const FORBIDDEN_MARKS: [(&str, &str); 4] = [
    ("\u{2014}", "tiret cadratin (—)"),
    ("\u{2013}", "tiret demi-cadratin (–)"),
    ("**", "gras Markdown (**)"),
    ("`", "backtick (`)"),
];

/// Violation relevée par le linter.
///
/// # Attributes
///
/// * `field` - Chemin du champ concerné (ex. `paragraphs[2]`).
/// * `message` - Description de la violation, formulée comme une consigne.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Violation {
    pub field: String,
    pub message: String,
}

impl Violation {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self { field: field.into(), message: message.into() }
    }
}

/// Marques interdites que la prose de la référence n'emploie pas.
fn forbidden_marks(example_prose: &str) -> Vec<(&'static str, &'static str)> {
    FORBIDDEN_MARKS.into_iter().filter(|(mark, _)| !example_prose.contains(mark)).collect()
}

/// Vérifie les caractères et formules interdits dans chaque champ de prose.
///
/// # Arguments
///
/// * `fields` - Champs de prose du contenu rédigé.
/// * `example_prose` - Prose de la référence : une marque ou une formule
///   qui y figure est autorisée.
///
/// # Returns
///
/// Violations relevées, dans l'ordre des champs.
pub fn lint_prose(fields: &[ProseField], example_prose: &str) -> Vec<Violation> {
    let marks = forbidden_marks(example_prose);
    let example = example_prose.to_lowercase();
    let allowed_phrases: Vec<&str> = FORBIDDEN_PHRASES_FR
        .iter()
        .chain(FORBIDDEN_PHRASES_EN)
        .copied()
        .filter(|phrase| !example.contains(phrase))
        .collect();

    let mut violations = Vec::new();
    for field in fields {
        for (mark, label) in &marks {
            if field.text.contains(mark) {
                violations.push(Violation::new(&field.path, format!("contient un {label}, interdit")));
            }
        }
        let lowered = field.text.to_lowercase();
        for phrase in &allowed_phrases {
            if lowered.contains(phrase) {
                violations.push(Violation::new(&field.path, format!("contient la formule interdite « {phrase} »")));
            }
        }
    }
    violations
}

/// Vérifie une plage de mots.
fn check_words(field: &str, text: &str, range: &WordRange) -> Option<Violation> {
    let words = count_words(text);
    (!range.contains(words)).then(|| {
        Violation::new(field, format!("fait {words} mots, attendu entre {} et {} mots", range.min, range.max))
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
/// virgules, un tiret isolé devient un trait d'union.
///
/// # Arguments
///
/// * `text` - Texte à nettoyer.
///
/// # Returns
///
/// Texte sans tiret cadratin, demi-cadratin, gras Markdown ni backtick.
pub fn strip_dashes(text: &str) -> String {
    strip_forbidden_marks(text, "")
}

/// Comme `strip_dashes`, mais ne retire que les marques que la prose de la
/// référence n'emploie pas.
///
/// # Arguments
///
/// * `text` - Texte à nettoyer.
/// * `example_prose` - Prose de la référence.
///
/// # Returns
///
/// Texte sans les marques que la référence n'emploie pas.
pub fn strip_forbidden_marks(text: &str, example_prose: &str) -> String {
    let marks: Vec<&str> = forbidden_marks(example_prose).into_iter().map(|(mark, _)| mark).collect();
    let mut cleaned = text.to_string();
    for dash in ["\u{2014}", "\u{2013}"].into_iter().filter(|dash| marks.contains(dash)) {
        cleaned = cleaned.replace(&format!(" {dash} "), ", ").replace(dash, "-");
    }
    for mark in ["**", "`"].into_iter().filter(|mark| marks.contains(mark)) {
        cleaned = cleaned.replace(mark, "");
    }
    cleaned.replace(" ,", ",")
}
