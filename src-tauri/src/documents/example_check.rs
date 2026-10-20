//! Validation des données d'exemple (étape A de la templatisation) et ce
//! que la suite en tire : consignes rattachées, textes en dur, valeurs à ne
//! pas trouver en dur dans le template.
//!
//! Tous les problèmes d'un essai sont rapportés ensemble. Seule une sortie
//! qui ne respecte pas le schéma typé arrête les contrôles : sans données
//! typées, les autres n'ont pas de sens.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::example_facts::{
    cv_information_values, cv_prose_values, lm_information_values, lm_prose_values, word_keys,
};
use crate::documents::markup::hard_texts::hard_texts;
use crate::documents::markup::{attach_zones, ReferenceMarkup};
use crate::models::{CvExampleData, LmExampleData, ProseField, ProseGuidance};

/// Langues de document prises en charge.
const SUPPORTED_LANGS: [&str; 2] = ["fr", "en"];
/// Chemins dont la valeur n'a pas à figurer dans l'original : libellés
/// (affichés seulement si leur section ou leur champ l'est : mois, période
/// en cours, permis…), langue, granularité des périodes, textes en dur.
const NOT_DISPLAYED_PATHS: [&str; 4] = ["labels", "lang", "period_style", "texts"];
/// Au-delà de ce nombre de mots, une valeur n'est pas cherchée telle quelle
/// dans l'original : un long texte peut y être coupé par une césure, et la
/// prose est de toute façon comparée mot à mot au rendu du template.
const MAX_CHECKED_VALUE_WORDS: usize = 12;
/// Profondeur maximale parcourue dans les données d'exemple.
const MAX_DEPTH: usize = 16;

/// Données d'exemple validées et ce que la suite en tire.
///
/// # Attributes
///
/// * `example` - Données d'exemple, avec `texts` rempli.
/// * `lang` - Langue de la référence.
/// * `guidance` - Consignes rattachées aux champs de prose.
/// * `texts` - Textes en dur de la référence, par identifiant.
/// * `values` - Valeurs qui ont un champ (informations et prose), qu'aucun
///   texte en dur du template ne doit reproduire.
/// * `paragraphs` - Paragraphes du corps d'une lettre (vide pour un CV).
pub struct CheckedExample {
    pub example: Value,
    pub lang: String,
    pub guidance: ProseGuidance,
    pub texts: BTreeMap<String, String>,
    pub values: Vec<String>,
    pub paragraphs: Vec<String>,
}

/// Contexte de la validation.
///
/// # Attributes
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `reference_text` - Texte rendu de la référence.
/// * `computes_date` - Vrai si la référence calcule sa date.
/// * `markup` - Balisage lu dans la référence.
pub struct ExampleContext<'a> {
    pub doc_type: &'a str,
    pub reference_text: &'a str,
    pub computes_date: bool,
    pub markup: &'a ReferenceMarkup,
}

/// Valide des données d'exemple produites par le LLM.
///
/// # Arguments
///
/// * `raw` - Données d'exemple brutes.
/// * `context` - Type de document, référence et balisage.
///
/// # Returns
///
/// `CheckedExample`, ou tous les problèmes relevés.
pub fn check_example(raw: &Value, context: &ExampleContext<'_>) -> Result<CheckedExample, Vec<String>> {
    let typed = TypedExample::parse(raw, context.doc_type).map_err(|problem| vec![problem])?;
    let mut problems = Vec::new();
    if !SUPPORTED_LANGS.contains(&typed.lang()) {
        problems.push(format!("lang vaut \"{}\", attendu \"fr\" ou \"en\".", typed.lang()));
    }
    problems.extend(typed.specific_problems());
    problems.extend(absent_values(raw, context));

    let guidance = match attach_zones(&context.markup.general, &context.markup.zones, &typed.prose_fields()) {
        Ok(guidance) => guidance,
        Err(errors) => {
            problems.extend(errors.iter().map(|e| format!("Balisage de la référence, {e}")));
            ProseGuidance::default()
        }
    };
    if !problems.is_empty() {
        return Err(problems);
    }

    let texts = hard_texts(&context.markup.visible_texts, &all_strings(raw), context.reference_text);
    let mut example = raw.clone();
    if let Some(map) = example.as_object_mut() {
        map.insert("texts".to_string(), serde_json::to_value(&texts).unwrap_or_default());
    }
    Ok(CheckedExample {
        example,
        lang: typed.lang().to_string(),
        guidance,
        texts,
        values: typed.values(),
        paragraphs: typed.paragraphs(),
    })
}

/// Données d'exemple typées selon le type de document.
enum TypedExample {
    Cv(Box<CvExampleData>),
    Lm(Box<LmExampleData>),
}

impl TypedExample {
    /// Désérialise les données selon le type de document.
    fn parse(raw: &Value, doc_type: &str) -> Result<Self, String> {
        match doc_type {
            "cv" => serde_json::from_value(raw.clone())
                .map(|e| Self::Cv(Box::new(e)))
                .map_err(|e| format!("Les données ne respectent pas le schéma du CV : {e}")),
            _ => serde_json::from_value(raw.clone())
                .map(|e| Self::Lm(Box::new(e)))
                .map_err(|e| format!("Les données ne respectent pas le schéma de la lettre : {e}")),
        }
    }

    /// Langue déclarée.
    fn lang(&self) -> &str {
        match self {
            Self::Cv(e) => &e.data.lang,
            Self::Lm(e) => &e.data.lang,
        }
    }

    /// Problèmes propres au type de document.
    fn specific_problems(&self) -> Vec<String> {
        match self {
            Self::Cv(e) => {
                let months = e.data.labels.months_short.len();
                if months == 12 {
                    Vec::new()
                } else {
                    vec![format!("labels.months_short doit contenir 12 abréviations, il en contient {months}.")]
                }
            }
            Self::Lm(e) => recipient_problems(&e.data.recipient.location, &e.data.recipient.extra_lines),
        }
    }

    /// Champs de prose, aux chemins des consignes.
    fn prose_fields(&self) -> Vec<ProseField> {
        let field = |path: &str, text: &str| ProseField { path: path.to_string(), text: text.to_string() };
        match self {
            Self::Cv(e) => vec![field("identity.title", &e.data.identity.title), field("summary", &e.data.summary)],
            Self::Lm(e) => std::iter::once(field("subject", &e.data.subject))
                .chain(e.data.paragraphs.iter().enumerate().map(|(i, p)| field(&format!("paragraphs[{i}]"), p)))
                .collect(),
        }
    }

    /// Valeurs qui ont un champ, à ne pas trouver en dur dans le template.
    fn values(&self) -> Vec<String> {
        match self {
            Self::Cv(e) => [cv_information_values(&e.data), cv_prose_values(&e.data)].concat(),
            Self::Lm(e) => [lm_information_values(&e.data), lm_prose_values(&e.data)].concat(),
        }
    }

    /// Paragraphes du corps d'une lettre.
    fn paragraphs(&self) -> Vec<String> {
        match self {
            Self::Cv(_) => Vec::new(),
            Self::Lm(e) => e.data.paragraphs.clone(),
        }
    }
}

/// Problèmes du destinataire : la ligne de la ville rangée dans
/// `extra_lines` au lieu de `location`.
fn recipient_problems(location: &str, extra_lines: &[String]) -> Vec<String> {
    let looks_like_city = |line: &str| {
        let mut parts = line.split_whitespace();
        let first = parts.next().unwrap_or_default();
        first.len() >= 4 && first.chars().all(|c| c.is_ascii_digit()) && parts.next().is_some()
    };
    let misplaced: Vec<&String> = extra_lines.iter().filter(|line| looks_like_city(line)).collect();
    if location.trim().is_empty() && !extra_lines.is_empty() {
        return vec![format!(
            "recipient.location est vide alors que le destinataire a des lignes ({}) : la ligne qui porte la ville \
             (par exemple \"1201 Genève\") va dans recipient.location, pas dans extra_lines.",
            extra_lines.join(" / ")
        )];
    }
    misplaced
        .into_iter()
        .map(|line| format!("recipient.extra_lines contient \"{line}\", qui ressemble à la ligne de la ville : elle va dans recipient.location."))
        .collect()
}

/// Valeurs des données d'exemple absentes du texte de l'original : le LLM
/// les a inventées ou modifiées.
fn absent_values(raw: &Value, context: &ExampleContext<'_>) -> Vec<String> {
    let rendered = word_keys(context.reference_text);
    let mut strings = Vec::new();
    collect_strings(raw, "", &mut strings, 0);
    strings
        .into_iter()
        .filter(|(path, _)| !NOT_DISPLAYED_PATHS.iter().any(|p| path.starts_with(p)))
        .filter(|(path, _)| !(context.computes_date && path == "date"))
        .filter(|(_, value)| {
            let keys = word_keys(value);
            !keys.is_empty() && keys.len() <= MAX_CHECKED_VALUE_WORDS && !rendered.windows(keys.len()).any(|window| window == keys.as_slice())
        })
        .map(|(path, value)| {
            format!("{path} vaut « {value} », qui n'apparaît pas tel quel dans l'original : recopie la valeur affichée.")
        })
        .collect()
}

/// Toutes les chaînes non vides des données.
fn all_strings(raw: &Value) -> Vec<String> {
    let mut strings = Vec::new();
    collect_strings(raw, "", &mut strings, 0);
    strings.into_iter().map(|(_, value)| value).collect()
}

/// Parcourt les données et collecte `(chemin, chaîne)` pour chaque chaîne
/// non vide.
fn collect_strings(value: &Value, path: &str, out: &mut Vec<(String, String)>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    match value {
        Value::String(text) if !text.trim().is_empty() => out.push((path.to_string(), text.clone())),
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                collect_strings(item, &format!("{path}[{i}]"), out, depth + 1);
            }
        }
        Value::Object(map) => {
            for (key, field) in map {
                let child = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                collect_strings(field, &child, out, depth + 1);
            }
        }
        _ => {}
    }
}
