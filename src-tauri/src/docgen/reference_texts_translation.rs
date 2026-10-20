//! Traduction des textes d'une référence (`ReferenceTexts`) : découpage en
//! éléments à traduire, validation de la traduction et reconstruction.
//! Fonctions pures, sans LLM.
//!
//! Sont traduits : les textes en dur (`texts`), les consignes générales,
//! la consigne de chaque zone et le texte des zones `[FIXE]` (recopiées
//! dans la langue du document). Le texte des autres zones reste celui de
//! la référence : c'est un modèle de style, réécrit à chaque génération.

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::models::{ReferenceTexts, ZoneMode};

/// Allongement maximal d'une traduction, en pourcentage de l'original :
/// l'anglais vers le français allonge souvent de 15 à 30 %.
const LENGTH_TOLERANCE_PERCENT: usize = 50;
/// Allongement toléré en caractères, quelle que soit la longueur.
const SHORT_TEXT_MARGIN_CHARS: usize = 20;
/// En dessous de ce nombre de mots, une traduction identique à l'original
/// est acceptée (« CV », un nom propre, un sigle).
const MIN_WORDS_TO_DIFFER: usize = 4;

/// Préfixe des éléments « texte en dur ».
const TEXT_PREFIX: &str = "texte:";
/// Élément des consignes générales.
const GENERAL_ID: &str = "consignes";
/// Préfixe des éléments « consigne d'une zone ».
const INSTRUCTION_PREFIX: &str = "zone_consigne:";
/// Préfixe des éléments « texte d'une zone [FIXE] ».
const FIXED_PREFIX: &str = "zone_fixe:";

/// Élément à traduire.
///
/// # Attributes
///
/// * `id` - Identifiant stable de l'élément.
/// * `text` - Texte.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TranslationItem {
    pub id: String,
    pub text: String,
}

/// Éléments à traduire d'une référence, dans un ordre stable.
///
/// # Arguments
///
/// * `texts` - Textes de la référence.
///
/// # Returns
///
/// Les éléments non vides.
pub fn translatable_items(texts: &ReferenceTexts) -> Vec<TranslationItem> {
    let mut items: Vec<TranslationItem> =
        texts.texts.iter().map(|(id, text)| item(format!("{TEXT_PREFIX}{id}"), text)).collect();
    items.push(item(GENERAL_ID.to_string(), &texts.guidance.general));
    for (i, zone) in texts.guidance.zones.iter().enumerate() {
        items.push(item(format!("{INSTRUCTION_PREFIX}{i}"), &zone.instruction));
        if zone.mode == ZoneMode::Fixe {
            items.push(item(format!("{FIXED_PREFIX}{i}"), &zone.text));
        }
    }
    items.retain(|i| !i.text.trim().is_empty());
    items
}

/// Construit un élément.
fn item(id: String, text: &str) -> TranslationItem {
    TranslationItem { id, text: text.to_string() }
}

/// Textes de la référence dans la langue cible, à partir des éléments
/// traduits (validés par `translation_problems`).
///
/// # Arguments
///
/// * `texts` - Textes d'origine.
/// * `translated` - Éléments traduits.
/// * `lang` - Langue cible.
///
/// # Returns
///
/// Les textes traduits ; un élément absent garde son texte d'origine.
pub fn apply_translation(texts: &ReferenceTexts, translated: &[TranslationItem], lang: &str) -> ReferenceTexts {
    let find = |id: &str| translated.iter().find(|t| t.id == id).map(|t| t.text.trim().to_string());
    let mut result = texts.clone();
    result.lang = lang.to_string();
    for (id, text) in result.texts.iter_mut() {
        if let Some(t) = find(&format!("{TEXT_PREFIX}{id}")) {
            *text = t;
        }
    }
    if let Some(t) = find(GENERAL_ID) {
        result.guidance.general = t;
    }
    for (i, zone) in result.guidance.zones.iter_mut().enumerate() {
        if let Some(t) = find(&format!("{INSTRUCTION_PREFIX}{i}")) {
            zone.instruction = t;
        }
        if let Some(t) = find(&format!("{FIXED_PREFIX}{i}")) {
            zone.text = t;
        }
    }
    result
}

/// Problèmes d'une traduction : mêmes identifiants, une fois chacun, textes
/// non vides, traduits (pas recopiés) et de longueur bornée.
///
/// # Arguments
///
/// * `original` - Éléments d'origine.
/// * `translated` - Éléments traduits.
///
/// # Returns
///
/// Problèmes formulés pour le LLM ; vide si la traduction est valide.
pub fn translation_problems(original: &[TranslationItem], translated: &[TranslationItem]) -> Vec<String> {
    let mut problems = Vec::new();
    for (rank, item) in translated.iter().enumerate() {
        if !original.iter().any(|o| o.id == item.id) {
            problems.push(format!("l'identifiant « {} » n'existe pas dans l'original.", item.id));
        } else if translated[..rank].iter().any(|t| t.id == item.id) {
            problems.push(format!("l'identifiant « {} » apparaît plusieurs fois.", item.id));
        }
    }
    for source in original {
        match translated.iter().find(|t| t.id == source.id) {
            None => problems.push(format!("l'identifiant « {} » manque.", source.id)),
            Some(t) => problems.extend(text_problem(source, &t.text)),
        }
    }
    problems
}

/// Problème d'un texte traduit, s'il y en a un.
fn text_problem(source: &TranslationItem, translated: &str) -> Option<String> {
    let translated = translated.trim();
    let id = &source.id;
    if translated.is_empty() {
        return Some(format!("{id} : la traduction est vide."));
    }
    let words = source.text.split_whitespace().count();
    if words >= MIN_WORDS_TO_DIFFER && translated == source.text.trim() {
        return Some(format!("{id} : le texte est recopié au lieu d'être traduit."));
    }
    let original_chars = source.text.trim().chars().count();
    let max = (original_chars * (100 + LENGTH_TOLERANCE_PERCENT)).div_ceil(100) + SHORT_TEXT_MARGIN_CHARS;
    let chars = translated.chars().count();
    (chars > max).then(|| format!("{id} : la traduction fait {chars} caractères, au plus {max} : traduis sans rien ajouter."))
}
