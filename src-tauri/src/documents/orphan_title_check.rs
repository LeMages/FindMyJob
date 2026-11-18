//! Contrôle du titre orphelin d'un template de CV : rendu avec des listes
//! vides (expériences, projets, formations, compétences, langues) mais les
//! libellés et les textes de la référence présents, un titre de section
//! n'est affiché que si sa section contient une unité fixe (un texte en
//! dur de la référence). Sinon le profil, vide sur cette catégorie, laisserait
//! un titre sans contenu.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des modules internes
use crate::docgen::example_facts::word_keys;
use crate::error::AppError;

/// Listes du CV vidées pour le rendu, avec le libellé de leur titre.
const LIST_SECTIONS: [(&str, &str); 5] = [
    ("experiences", "section_experiences"),
    ("projects", "section_projects"),
    ("formations", "section_formations"),
    ("skills_groups", "section_skills"),
    ("langues", "section_langues"),
];

/// Données d'exemple dont les listes du CV sont vides ; libellés, textes,
/// résumé et identité sont conservés.
///
/// # Arguments
///
/// * `example` - Données d'exemple sérialisées (telles que rendues).
///
/// # Returns
///
/// Données sérialisées, ou `AppError::Validation` si les données sont illisibles.
pub fn empty_lists_data(example: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut data: serde_json::Value = serde_json::from_slice(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple illisibles : {e}")))?;
    for (list, _) in LIST_SECTIONS {
        if let Some(field) = data.get_mut(list) {
            *field = serde_json::Value::Array(Vec::new());
        }
    }
    serde_json::to_vec(&data).map_err(|e| AppError::Validation(format!("Données de rendu non sérialisables : {e}")))
}

/// Position (en mots) de la première occurrence de `run` dans `haystack`.
fn find_run(haystack: &[String], run: &[String], from: usize) -> Option<usize> {
    if run.is_empty() || haystack.len() < run.len() {
        return None;
    }
    (from..=haystack.len() - run.len()).find(|&i| haystack[i..i + run.len()] == *run)
}

/// Problèmes de titres orphelins du rendu à listes vides.
///
/// Un titre est orphelin quand rien de ce qui le suit jusqu'au titre
/// suivant (ou la fin du document) n'est un texte en dur de la référence.
///
/// # Arguments
///
/// * `rendered_text` - Texte du rendu à listes vides.
/// * `example` - Données d'exemple (source des titres, `labels.section_*`).
/// * `texts` - Textes en dur de la référence, par identifiant.
///
/// # Returns
///
/// Un problème par titre orphelin (vide si aucun).
pub fn orphan_title_problems(
    rendered_text: &str,
    example: &serde_json::Value,
    texts: &BTreeMap<String, String>,
) -> Vec<String> {
    let rendered = word_keys(rendered_text);
    // ------------------------------------------------------------------
    // 1. Position de chaque titre non vide dans le rendu
    // ------------------------------------------------------------------
    let mut found: Vec<(usize, usize, &str, &str)> = Vec::new();
    for (list, label) in LIST_SECTIONS {
        let Some(title) = example["labels"][label].as_str().filter(|t| !t.trim().is_empty()) else {
            continue;
        };
        let keys = word_keys(title);
        if let Some(start) = find_run(&rendered, &keys, 0) {
            found.push((start, start + keys.len(), list, title));
        }
    }
    found.sort_by_key(|(start, ..)| *start);

    // ------------------------------------------------------------------
    // 2. Section de chaque titre : jusqu'au titre suivant ou la fin
    // ------------------------------------------------------------------
    let fixed: Vec<Vec<String>> = texts.values().map(|t| word_keys(t)).collect();
    let mut problems = Vec::new();
    for (i, (_, end, list, title)) in found.iter().enumerate() {
        let section_end = found.get(i + 1).map_or(rendered.len(), |next| next.0);
        let section = &rendered[(*end).min(section_end)..section_end];
        if fixed.iter().any(|text| find_run(section, text, 0).is_some()) {
            continue;
        }
        problems.push(format!(
            "Rendu avec les listes vides (`{list}` vide, libellés et textes de la référence présents), le titre « {title} » \
             reste affiché alors que sa section est vide. Un titre de section n'est affiché que si la section contient \
             un élément de sa liste ou un texte fixe de la référence : place-le dans un `if` sur la liste (ou sur le texte)."
        ));
    }
    problems
}
