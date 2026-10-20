//! Zones `[FIXE]` de la prose à la rédaction : une zone fixe de la référence
//! est recopiée mot pour mot, seules les informations du profil (ville,
//! école, entreprise…) peuvent y changer. Ce module vérifie qu'un contenu
//! rédigé les respecte, et protège leur texte des nettoyages mécaniques.

// Importation des modules standard
use std::collections::HashSet;

// Importation des modules internes
use crate::docgen::example_facts::word_keys;
use crate::docgen::lint::Violation;
use crate::docgen::text_diff::{diff_words, normalize_quotes, word_key};
use crate::models::{ProseField, ProseGuidance};

/// Nombre de mots manquants cités par violation.
const MAX_WORDS_REPORTED: usize = 15;

/// Vérifie que chaque zone fixe se retrouve dans le champ rédigé
/// correspondant. Un mot de la zone peut manquer seulement s'il fait partie
/// d'une information de la référence (il a été remplacé par celle du profil).
///
/// # Arguments
///
/// * `guidance` - Consignes de la référence (champs aux chemins du contenu
///   rédigé).
/// * `fields` - Champs de prose du contenu rédigé.
/// * `information_values` - Informations des données d'exemple (voir
///   `example_facts`).
///
/// # Returns
///
/// Une violation par zone fixe non respectée.
pub fn fixed_zone_violations(
    guidance: &ProseGuidance,
    fields: &[ProseField],
    information_values: &[String],
) -> Vec<Violation> {
    let replaceable: HashSet<String> = information_values.iter().flat_map(|v| word_keys(v)).collect();
    let mut violations = Vec::new();
    for field in fields {
        for zone in guidance.fixed_texts(&field.path) {
            let Ok(diff) = diff_words(zone, &field.text) else {
                continue;
            };
            let missing: Vec<&String> = diff
                .missing
                .iter()
                .filter(|word| !replaceable.contains(&word_key(&normalize_quotes(word)).to_lowercase()))
                .collect();
            if missing.is_empty() {
                continue;
            }
            let shown: Vec<String> = missing.iter().take(MAX_WORDS_REPORTED).map(|w| format!("\"{w}\"")).collect();
            violations.push(Violation {
                advisory: false,
                field: field.path.clone(),
                message: format!(
                    "doit reprendre mot pour mot la zone [FIXE] « {zone} » (seules les informations du profil \
                     peuvent y changer) ; mots manquants : {}",
                    shown.join(", ")
                ),
            });
        }
    }
    violations
}

/// Champs de prose dont le texte des zones fixes est retiré : le linter ne
/// juge que le texte rédigé, pas celui recopié de la référence.
///
/// # Arguments
///
/// * `guidance` - Consignes de la référence.
/// * `fields` - Champs de prose du contenu rédigé.
///
/// # Returns
///
/// Champs sans le texte de leurs zones fixes.
pub fn without_fixed_text(guidance: &ProseGuidance, fields: &[ProseField]) -> Vec<ProseField> {
    fields
        .iter()
        .map(|field| {
            let text = guidance.fixed_texts(&field.path).fold(field.text.clone(), |text, zone| text.replacen(zone, " ", 1));
            ProseField { path: field.path.clone(), text }
        })
        .collect()
}

/// Applique une transformation au texte d'un champ, hors du texte de ses
/// zones fixes (recopié tel quel).
///
/// # Arguments
///
/// * `text` - Texte du champ.
/// * `fixed` - Textes des zones fixes du champ.
/// * `transform` - Transformation (ex. retrait des tirets interdits).
///
/// # Returns
///
/// Texte transformé, zones fixes intactes.
pub fn transform_outside_fixed(text: &str, fixed: &[&str], transform: impl Fn(&str) -> String) -> String {
    // Occurrences des zones fixes, sans chevauchement, dans l'ordre du texte.
    let mut spans: Vec<(usize, usize)> = fixed
        .iter()
        .filter(|zone| !zone.is_empty())
        .filter_map(|zone| text.find(zone).map(|start| (start, start + zone.len())))
        .collect();
    spans.sort_unstable();
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;
    for (start, end) in spans {
        if start < cursor {
            continue;
        }
        result.push_str(&transform(&text[cursor..start]));
        result.push_str(&text[start..end]);
        cursor = end;
    }
    result.push_str(&transform(&text[cursor..]));
    result
}
