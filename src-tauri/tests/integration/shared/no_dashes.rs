//! Contrôle commun aux tests de génération : aucun tiret cadratin,
//! demi-cadratin, « --- » ni « -- » dans un document généré.

// Importation des crates externes
use serde_json::Value;

/// Marques qui ne doivent jamais apparaître dans un document généré.
pub const FORBIDDEN_DASHES: [&str; 4] = ["\u{2014}", "\u{2013}", "---", "--"];

/// Chaînes d'une valeur JSON (récursif ; `data.json` a une profondeur de 3).
fn all_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Array(items) => items.iter().for_each(|item| all_strings(item, out)),
        Value::Object(fields) => fields.values().for_each(|field| all_strings(field, out)),
        _ => {}
    }
}

/// Vérifie qu'aucune chaîne d'un `data.json` ne contient de tiret interdit.
///
/// # Arguments
///
/// * `label` - Nom du document, pour le message d'échec.
/// * `data` - `data.json` généré.
pub fn assert_no_dash_in_data(label: &str, data: &Value) {
    let mut strings = Vec::new();
    all_strings(data, &mut strings);
    for text in strings {
        assert!(!FORBIDDEN_DASHES.iter().any(|mark| text.contains(mark)), "{label} : tiret interdit dans « {text} »");
    }
}

/// Vérifie qu'un texte rendu ne contient aucun tiret interdit.
///
/// # Arguments
///
/// * `label` - Nom du document, pour le message d'échec.
/// * `text` - Texte rendu du PDF.
pub fn assert_no_dash_in_text(label: &str, text: &str) {
    for mark in FORBIDDEN_DASHES {
        assert!(!text.contains(mark), "{label} : « {mark} » dans le rendu : {text}");
    }
}
