//! Cohérence des consignes extraites à la templatisation
//! (`example_data.guidance`) avec la prose des données d'exemple : chaque
//! zone appartient à un champ de prose, et les zones d'un champ, mises bout
//! à bout, redonnent exactement ce champ.

// Importation des modules internes
use crate::docgen::example_facts::word_keys;
use crate::docgen::text_diff::diff_words;
use crate::models::{ProseField, ProseGuidance};

/// Nombre de mots en écart cités par problème.
const MAX_WORDS_REPORTED: usize = 15;

/// Problèmes de cohérence des consignes, formulés pour le feedback du LLM.
///
/// # Arguments
///
/// * `guidance` - Consignes extraites.
/// * `fields` - Champs de prose des données d'exemple (chemin = nom de
///   champ attendu dans `guidance.zones`).
///
/// # Returns
///
/// Problèmes relevés (vide si les consignes sont cohérentes).
pub fn guidance_problems(guidance: &ProseGuidance, fields: &[ProseField]) -> Vec<String> {
    let mut problems = Vec::new();
    let known: Vec<&str> = fields.iter().map(|f| f.path.as_str()).collect();

    // ------------------------------------------------------------------
    // 1. Chaque zone appartient à un champ de prose et a un texte
    // ------------------------------------------------------------------
    for zone in &guidance.zones {
        if !known.contains(&zone.field.as_str()) {
            problems.push(format!(
                "guidance.zones cite le champ « {} », qui n'est pas un champ de prose. Champs possibles : {}.",
                zone.field,
                known.join(", ")
            ));
        } else if zone.text.trim().is_empty() {
            problems.push(format!("guidance.zones contient une zone vide pour « {} ».", zone.field));
        }
    }

    // ------------------------------------------------------------------
    // 2. Les zones d'un champ redonnent exactement sa valeur
    // ------------------------------------------------------------------
    for field in fields {
        let joined = guidance.zones_of(&field.path).map(|z| z.text.as_str()).collect::<Vec<_>>().join(" ");
        if joined.is_empty() || word_keys(&joined) == word_keys(&field.text) {
            continue;
        }
        problems.push(describe_mismatch(&field.path, &field.text, &joined));
    }
    problems
}

/// Formule l'écart entre un champ et ses zones.
fn describe_mismatch(path: &str, value: &str, joined: &str) -> String {
    let quote = |words: &[String]| {
        let shown: Vec<String> = words.iter().take(MAX_WORDS_REPORTED).map(|w| format!("\"{w}\"")).collect();
        if shown.is_empty() { "aucun".to_string() } else { shown.join(", ") }
    };
    match diff_words(value, joined) {
        Ok(diff) => format!(
            "Les zones de « {path} » mises bout à bout ne redonnent pas sa valeur dans example_data. \
             Mots du champ absents des zones : {}. Mots des zones absents du champ : {}.",
            quote(&diff.missing),
            quote(&diff.extra)
        ),
        Err(e) => format!("Les zones de « {path} » ne redonnent pas sa valeur dans example_data ({e})."),
    }
}
