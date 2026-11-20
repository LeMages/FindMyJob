//! Marqueurs du rendu « sentinelle » d'une référence : chaque champ texte
//! des données porte un marqueur unique (`__F1__`, `__F2__`…, voir
//! `documents::reference_contract::example_schema::sentinel`). Tout le
//! texte venu des données est alors remplacé par des marqueurs : ce qui
//! reste du rendu est le texte écrit en dur dans le template.
//!
//! Le texte en dur est légitime (libellés, phrases fixes), sauf s'il
//! reproduit une valeur qui a un champ dans le schéma (nom, ville,
//! entreprise…) : cette valeur ne serait jamais remplacée par celle du
//! profil.

// Importation des modules internes
use crate::docgen::example_facts::word_keys;

/// Début d'un marqueur, suivi de son numéro.
pub const MARKER_START: &str = "__F";
/// Fin d'un marqueur.
pub const MARKER_END: &str = "__";

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

/// Remplace chaque marqueur du texte par une barre isolée : deux mots en
/// dur séparés par une valeur des données ne se suivent plus.
pub fn strip_markers(text: &str) -> String {
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
