//! Textes en dur d'une référence : textes visibles qui ne sont ni une
//! donnée du profil, ni un libellé, ni de la prose rédigée (« Références
//! disponibles sur demande », « P.J. : CV »…). Ils reçoivent un identifiant
//! (`t1`, `t2`…) et le template les lit dans `data.texts`, ce qui les rend
//! traduisibles.
//!
//! Chaque texte visible du source (littéral, ou suite de texte en markup)
//! est comparé aux valeurs des données d'exemple : les parties couvertes
//! par une valeur sont retirées, ce qui reste et contient une lettre est un
//! texte en dur. Un candidat absent du rendu de la référence (argument d'une
//! fonction qui ne l'affiche pas) est écarté.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des modules internes
use crate::docgen::example_facts::word_keys;
use crate::docgen::text_diff::{normalize_quotes, words};
use crate::documents::markup::model::VisibleText;

/// Préfixe des identifiants de textes en dur.
pub const TEXT_ID_PREFIX: &str = "t";

/// Textes en dur d'une référence.
///
/// # Arguments
///
/// * `visible` - Textes visibles du source, dans l'ordre.
/// * `values` - Toutes les chaînes des données d'exemple (champs, libellés, prose).
/// * `reference_text` - Texte du rendu de la référence.
///
/// # Returns
///
/// Textes en dur par identifiant, numérotés dans l'ordre du source, sans doublon.
pub fn hard_texts(visible: &[VisibleText], values: &[String], reference_text: &str) -> BTreeMap<String, String> {
    let mut value_keys: Vec<Vec<String>> = values.iter().map(|v| word_keys(v)).filter(|k| !k.is_empty()).collect();
    // Les valeurs longues d'abord : une valeur courte ne doit pas couper une
    // valeur plus longue qui la contient.
    value_keys.sort_by_key(|keys| std::cmp::Reverse(keys.len()));
    let rendered = word_keys(reference_text);

    let mut texts = BTreeMap::new();
    for unit in visible {
        for candidate in uncovered_runs(&unit.text, &value_keys) {
            if contains_run(&rendered, &word_keys(&candidate)) {
                add_text(&mut texts, &candidate);
            }
        }
    }
    texts
}

/// Ajoute un texte en dur s'il n'y est pas déjà.
///
/// # Arguments
///
/// * `texts` - Textes en dur existants (modifiés).
/// * `text` - Texte à ajouter.
///
/// # Returns
///
/// L'identifiant du texte (existant ou nouveau).
pub fn add_text(texts: &mut BTreeMap<String, String>, text: &str) -> String {
    if let Some((id, _)) = texts.iter().find(|(_, existing)| existing.as_str() == text) {
        return id.clone();
    }
    let next = texts.keys().filter_map(|id| id.strip_prefix(TEXT_ID_PREFIX)?.parse::<usize>().ok()).max().unwrap_or(0) + 1;
    let id = format!("{TEXT_ID_PREFIX}{next}");
    texts.insert(id.clone(), text.to_string());
    id
}

/// Parties d'un texte visible qu'aucune valeur ne couvre et qui contiennent
/// une lettre.
fn uncovered_runs(text: &str, value_keys: &[Vec<String>]) -> Vec<String> {
    let normalized = normalize_quotes(text);
    let tokens = words(&normalized);
    let keys: Vec<String> = word_keys(&normalized);
    // Le texte entier fait partie d'une valeur (une ligne d'un paragraphe).
    if value_keys.iter().any(|value| contains_run(value, &keys)) {
        return Vec::new();
    }
    let mut covered = vec![false; keys.len()];
    for value in value_keys {
        mark_occurrences(&keys, value, &mut covered);
    }
    let mut runs = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for (token, is_covered) in tokens.iter().zip(&covered) {
        if *is_covered {
            push_run(&mut runs, &mut current);
        } else {
            current.push(token);
        }
    }
    push_run(&mut runs, &mut current);
    runs
}

/// Termine une suite de mots non couverts ; la garde si elle a une lettre.
fn push_run(runs: &mut Vec<String>, current: &mut Vec<&str>) {
    if current.iter().any(|token| token.chars().any(char::is_alphabetic)) {
        runs.push(current.join(" "));
    }
    current.clear();
}

/// Marque chaque occurrence de `value` dans `keys`.
fn mark_occurrences(keys: &[String], value: &[String], covered: &mut [bool]) {
    if value.is_empty() || value.len() > keys.len() {
        return;
    }
    for start in 0..=keys.len() - value.len() {
        if keys[start..start + value.len()] == *value {
            covered[start..start + value.len()].iter_mut().for_each(|c| *c = true);
        }
    }
}

/// Vrai si `run` apparaît, à la suite, dans `haystack`.
fn contains_run(haystack: &[String], run: &[String]) -> bool {
    !run.is_empty() && haystack.windows(run.len()).any(|window| window == run)
}
