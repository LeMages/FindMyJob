//! Correspondance vers les types de contrat du formulaire (`stage`,
//! `alternance`, `interim`, `cdd`, `cdi`) depuis le JSON-LD d'une offre.
//!
//! Règle : on ne devine jamais. `employmentType` n'est fiable que pour
//! `INTERN` ; sinon on cherche un mot-clé explicite dans le titre, et un
//! titre qui en cite plusieurs types différents ne donne rien.

// Importation des crates externes
use serde_json::Value;

/// Mots-clés recherchés dans le titre (mots entiers, en minuscules, sans
/// accents), avec le type de contrat correspondant.
const TITLE_KEYWORDS: &[(&str, &str)] = &[
    ("stage", "stage"),
    ("stagiaire", "stage"),
    ("intern", "stage"),
    ("internship", "stage"),
    ("alternance", "alternance"),
    ("alternant", "alternance"),
    ("alternante", "alternance"),
    ("apprentissage", "alternance"),
    ("cdi", "cdi"),
    ("cdd", "cdd"),
    ("interim", "interim"),
];

/// Détermine le type de contrat d'une offre.
///
/// # Arguments
///
/// * `employment_type` - Valeur JSON-LD `employmentType` (chaîne, tableau, ou absente).
/// * `title` - Intitulé du poste.
///
/// # Returns
///
/// Le type de contrat du formulaire, ou `None` si rien d'explicite.
pub fn resolve_contract_type(employment_type: Option<&Value>, title: Option<&str>) -> Option<&'static str> {
    if employment_type.is_some_and(is_intern) {
        return Some("stage");
    }
    title.and_then(contract_type_from_title)
}

/// Cherche un type de contrat explicite dans un titre d'offre.
///
/// # Arguments
///
/// * `title` - Intitulé du poste.
///
/// # Returns
///
/// Le type trouvé, ou `None` si aucun mot-clé ou si plusieurs types
/// différents sont cités (ex. « Stage ou alternance »).
pub fn contract_type_from_title(title: &str) -> Option<&'static str> {
    let normalized = fold_accents(&title.to_lowercase());
    let mut found: Option<&'static str> = None;
    for word in normalized.split(|c: char| !c.is_alphanumeric()) {
        let Some((_, contract)) = TITLE_KEYWORDS.iter().find(|(keyword, _)| *keyword == word) else {
            continue;
        };
        match found {
            Some(previous) if previous != *contract => return None,
            _ => found = Some(contract),
        }
    }
    found
}

/// Indique si `employmentType` vaut `INTERN` (seul ou dans un tableau).
fn is_intern(value: &Value) -> bool {
    match value {
        Value::String(s) => s.trim().eq_ignore_ascii_case("INTERN"),
        Value::Array(items) => items.iter().any(is_intern),
        _ => false,
    }
}

/// Retire les accents français courants (le titre « Intérim » doit matcher
/// « interim »).
fn fold_accents(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}
