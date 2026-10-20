//! Rendu « sentinelle » d'un template : données où chaque champ texte porte
//! un marqueur unique (`__F1__`, `__F2__`…) et chaque liste un seul élément.
//! Tout le texte venu des données est alors remplacé par des marqueurs : ce
//! qui reste du rendu est le texte écrit en dur dans le template.
//!
//! Le texte en dur est légitime (la référence est la source de vérité du
//! texte), sauf s'il reproduit une valeur qui a un champ dans le schéma
//! (nom, ville, entreprise, résumé…) : cette valeur ne serait jamais
//! remplacée par celle du profil. Tous les champs sont remplis à la fois,
//! si bien qu'une branche conditionnée à un champ (`if data.summary != ""`)
//! est rendue elle aussi.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::example_facts::word_keys;
use crate::docgen::text_diff::word_key;
use crate::models::{
    CvData, CvExperience, CvFormation, CvLabels, CvLangue, CvProject, Identity, LmData, SkillsGroup,
};

/// Noms de mois (anglais puis français, en minuscules). Une date affichée
/// par le document en contient un, entier ou abrégé, sans qu'il soit un
/// contenu écrit en dur.
pub const MONTH_NAMES: [&str; 24] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september", "october",
    "november", "december", "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août",
    "septembre", "octobre", "novembre", "décembre",
];
/// Début d'un marqueur, suivi de son numéro.
const MARKER_START: &str = "__F";
/// Fin d'un marqueur.
const MARKER_END: &str = "__";
/// Champ jamais marqué : la langue pilote `set text(lang: …)` et doit
/// rester une langue valide.
const UNMARKED_KEY: &str = "lang";
/// Profondeur maximale parcourue dans les données : le schéma canonique en
/// a moins de cinq niveaux.
const MAX_DEPTH: usize = 16;

/// Données sentinelles d'un CV : une entrée par liste (listes imbriquées
/// comprises), 12 abréviations de mois, permis affiché, et un marqueur dans
/// chaque chaîne sauf `lang`.
///
/// # Arguments
///
/// * `lang` - Langue du document (`"fr"` ou `"en"`), gardée telle quelle.
/// * `text_ids` - Identifiants des textes en dur de la référence (`data.texts`).
///
/// # Returns
///
/// Données au schéma `CvData`, en JSON.
pub fn sentinel_cv_data(lang: &str, text_ids: &[String]) -> Result<Value, serde_json::Error> {
    let data = CvData {
        identity: Identity { permis: true, ..Default::default() },
        lang: lang.to_string(),
        skills_groups: vec![SkillsGroup { skills: vec![String::new()], ..Default::default() }],
        experiences: vec![CvExperience { bullets: vec![String::new()], ..Default::default() }],
        projects: vec![CvProject { bullets: vec![String::new()], ..Default::default() }],
        formations: vec![CvFormation { details: vec![String::new()], ..Default::default() }],
        langues: vec![CvLangue::default()],
        interets: vec![String::new()],
        labels: CvLabels { months_short: vec![String::new(); 12], ..Default::default() },
        texts: empty_texts(text_ids),
        ..Default::default()
    };
    Ok(marked(serde_json::to_value(data)?))
}

/// Données sentinelles d'une lettre : un paragraphe, permis affiché, et un
/// marqueur dans chaque chaîne sauf `lang`.
///
/// # Arguments
///
/// * `lang` - Langue du document (`"fr"` ou `"en"`), gardée telle quelle.
/// * `text_ids` - Identifiants des textes en dur de la référence (`data.texts`).
///
/// # Returns
///
/// Données au schéma `LmData`, en JSON.
pub fn sentinel_lm_data(lang: &str, text_ids: &[String]) -> Result<Value, serde_json::Error> {
    let data = LmData {
        identity: Identity { permis: true, ..Default::default() },
        lang: lang.to_string(),
        paragraphs: vec![String::new()],
        texts: empty_texts(text_ids),
        ..Default::default()
    };
    Ok(marked(serde_json::to_value(data)?))
}

/// Textes en dur vides (marqués ensuite), par identifiant.
fn empty_texts(text_ids: &[String]) -> BTreeMap<String, String> {
    text_ids.iter().map(|id| (id.clone(), String::new())).collect()
}

/// Remplace chaque chaîne des données (sauf `lang`) par un marqueur unique.
fn marked(mut data: Value) -> Value {
    let mut counter = 0;
    mark_strings(&mut data, &mut counter, 0);
    data
}

/// Parcourt les données en profondeur (bornée par `MAX_DEPTH`) et numérote
/// les marqueurs dans l'ordre de parcours.
fn mark_strings(value: &mut Value, counter: &mut usize, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    match value {
        Value::String(text) => {
            *counter += 1;
            *text = format!("{MARKER_START}{counter}{MARKER_END}");
        }
        Value::Array(items) => items.iter_mut().for_each(|item| mark_strings(item, counter, depth + 1)),
        Value::Object(map) => {
            for (_, field) in map.iter_mut().filter(|(key, _)| key.as_str() != UNMARKED_KEY) {
                mark_strings(field, counter, depth + 1);
            }
        }
        _ => {}
    }
}

/// Valeurs du schéma écrites en dur dans un template : celles dont les
/// mots se retrouvent, à la suite, dans le texte d'un rendu sentinelle une
/// fois ses marqueurs (quelle que soit la casse, `upper`/`lower` du
/// template) retirés.
///
/// # Arguments
///
/// * `text` - Texte rendu avec les données sentinelles.
/// * `values` - Valeurs des données d'exemple qui ont un champ (voir
///   `example_facts`).
///
/// # Returns
///
/// Valeurs retrouvées dans le texte en dur, dans l'ordre de `values`.
pub fn hardcoded_values(text: &str, values: &[String]) -> Vec<String> {
    let hardcoded = word_keys(&strip_markers(text));
    values
        .iter()
        .filter(|value| {
            let keys = word_keys(value);
            !keys.is_empty() && hardcoded.windows(keys.len()).any(|window| window == keys.as_slice())
        })
        .cloned()
        .collect()
}

/// Textes restés écrits en dur dans un rendu sentinelle : suites de mots
/// entre deux marqueurs qui contiennent une lettre (la ponctuation et les
/// séparateurs au bord sont retirés). Toutes les données étant marquées,
/// chacune est un texte que le template n'a pas lu dans `data`.
///
/// # Arguments
///
/// * `text` - Texte rendu avec les données sentinelles.
///
/// # Returns
///
/// Les textes en dur, dans l'ordre du rendu.
pub fn hardcoded_runs(text: &str) -> Vec<String> {
    let has_letter = |w: &&str| w.chars().any(char::is_alphabetic);
    strip_markers(text)
        .split(" | ")
        .filter_map(|segment| {
            let tokens: Vec<&str> = segment.split_whitespace().collect();
            let first = tokens.iter().position(has_letter)?;
            let last = tokens.iter().rposition(has_letter)?;
            Some(tokens[first..=last].join(" "))
        })
        .collect()
}

/// Remplace chaque marqueur du texte par une barre isolée : deux mots en
/// dur séparés par une valeur des données ne se suivent plus.
fn strip_markers(text: &str) -> String {
    let upper = text.to_uppercase();
    // `to_uppercase` peut changer la longueur de certains caractères : on ne
    // travaille sur la version majuscule que si les positions coïncident.
    let haystack = if upper.len() == text.len() { upper.as_str() } else { text };
    let mut result = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        match marker_len(&haystack[index..]) {
            Some(len) => {
                result.push_str(" | ");
                index += len;
            }
            None => {
                let c = text[index..].chars().next().expect("index sur une frontière de caractère");
                result.push(c);
                index += c.len_utf8();
            }
        }
    }
    result
}

/// Longueur du marqueur qui commence le texte, s'il y en a un.
fn marker_len(text: &str) -> Option<usize> {
    let rest = text.strip_prefix(MARKER_START)?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 || !rest[digits..].starts_with(MARKER_END) {
        return None;
    }
    Some(MARKER_START.len() + digits + MARKER_END.len())
}

/// Indique si un mot est un nom de mois, entier ou abrégé (au moins trois
/// lettres), ponctuation de bord ignorée.
///
/// # Arguments
///
/// * `word` - Mot rendu.
///
/// # Returns
///
/// `true` pour « octobre », « Oct. », « Janv. »…
pub fn is_month_word(word: &str) -> bool {
    let cleaned = word_key(word).to_lowercase();
    cleaned.chars().count() >= 3 && MONTH_NAMES.iter().any(|month| month.starts_with(&cleaned))
}
