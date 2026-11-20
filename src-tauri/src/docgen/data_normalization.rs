//! Normalisation des tirets de toutes les chaînes d'un `data.json` (CV ou
//! lettre), juste avant la compilation : prose rédigée, profil, zones
//! [FIXE], textes de la référence, traductions, libellés, périodes. Les
//! données du profil en base ne sont jamais modifiées : seul le rendu l'est.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::dash_normalization::{normalize_dashes, DashContext};

/// Champs (ou listes) de phrases : une incise entre tirets y devient une
/// incise entre virgules. Tout autre champ est traité comme un champ court.
const PARAGRAPH_KEYS: [&str; 5] = ["summary", "paragraphs", "intro", "bullets", "details"];
/// Champs jamais touchés : coordonnées (déjà épargnées jeton par jeton,
/// elles sont aussi sautées en entier par prudence), langue, et symbole des
/// listes, choisi par la session (`-` doit rester un tiret simple).
const UNTOUCHED_KEYS: [&str; 8] =
    ["email", "telephone", "linkedin", "github", "portfolio", "url", "lang", "list_marker"];
/// Profondeur maximale parcourue : `data.json` en a 3 ; au-delà, la valeur
/// est laissée telle quelle plutôt que de risquer une récursion sans fin.
const MAX_DEPTH: usize = 16;

/// Normalise une chaîne ligne par ligne : un `\n` (intro sur plusieurs
/// lignes, suite d'un élément de liste) reste un retour à la ligne, même
/// quand la ligne suivante commence par un tiret (`– détail` ne fusionne
/// pas avec la ligne précédente).
fn normalize_lines(text: &str, context: DashContext) -> String {
    text.split('\n').map(|line| normalize_dashes(line, context)).collect::<Vec<_>>().join("\n")
}

/// Normalise récursivement une valeur, avec le contexte hérité de sa clé.
fn normalize_value(value: &mut Value, context: DashContext, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    match value {
        Value::String(text) => *text = normalize_lines(text, context),
        Value::Array(items) => items.iter_mut().for_each(|item| normalize_value(item, context, depth + 1)),
        Value::Object(fields) => {
            for (key, field) in fields.iter_mut() {
                if UNTOUCHED_KEYS.contains(&key.as_str()) {
                    continue;
                }
                let context =
                    if PARAGRAPH_KEYS.contains(&key.as_str()) { DashContext::Paragraph } else { DashContext::Short };
                normalize_value(field, context, depth + 1);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

/// Normalise les tirets de toutes les chaînes des données d'un document.
///
/// # Arguments
///
/// * `data` - Données du template (`data.json`), modifiées en place.
pub fn normalize_document_data(data: &mut Value) {
    normalize_value(data, DashContext::Short, 0);
}
