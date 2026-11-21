//! Données des rendus de contrôle d'une référence, toutes dérivées de ses
//! valeurs d'exemple et injectées par `sys.inputs.data` :
//! - l'exemple dans chaque langue prise en charge ;
//! - des données vides ;
//! - des données sentinelles (un marqueur par chaîne) ;
//! - l'exemple aux champs secondaires vides (séparateurs orphelins) ;
//! - (CV) l'exemple aux listes vides (titres orphelins) et avec un symbole
//!   de liste sentinelle ;
//! - (lettre) l'exemple au destinataire réduit à l'entreprise et la ville.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::documents::list_marker_check::{pick_sentinel, sentinel_marker_data};
use crate::documents::recipient_check::sparse_recipient_data;
use crate::documents::reference_contract::example_schema::{blank, sentinel, shape_for};
use crate::documents::sparse_check::{cleared_data, SPARSE_FIELDS};
use crate::error::AppError;
use crate::models::doc_lang::SUPPORTED_LANGS;

/// Listes du CV vidées pour le contrôle des titres orphelins.
const CV_LISTS: [&str; 5] = ["experiences", "projects", "formations", "skills_groups", "langues"];

/// Rendu de contrôle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderKind {
    /// Exemple dans une langue.
    Language(&'static str),
    /// Données vides.
    Blank,
    /// Données sentinelles.
    Sentinel,
    /// Champs secondaires vides.
    Sparse,
    /// Listes du CV vides.
    EmptyLists,
    /// Symbole de liste sentinelle.
    MarkerSentinel(&'static str),
    /// Destinataire réduit à l'entreprise et la ville.
    SparseRecipient,
}

impl RenderKind {
    /// Nom du rendu, cité dans le chemin d'une erreur de compilation.
    pub fn name(self) -> String {
        match self {
            RenderKind::Language(lang) => format!("render.language.{lang}"),
            RenderKind::Blank => "render.blank".to_string(),
            RenderKind::Sentinel => "render.sentinel".to_string(),
            RenderKind::Sparse => "render.sparse".to_string(),
            RenderKind::EmptyLists => "render.empty_lists".to_string(),
            RenderKind::MarkerSentinel(_) => "render.list_marker".to_string(),
            RenderKind::SparseRecipient => "render.recipient".to_string(),
        }
    }
}

/// Sérialise des données de rendu.
fn to_json(value: &Value) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|e| AppError::Validation(format!("Données de rendu non sérialisables : {e}")))
}

/// Convertit des données sérialisées en texte JSON.
fn bytes_to_json(bytes: Vec<u8>) -> Result<String, AppError> {
    String::from_utf8(bytes).map_err(|e| AppError::Validation(format!("Données de rendu non UTF-8 : {e}")))
}

/// Exemple aux listes du CV vides.
fn empty_lists(example: &Value) -> Value {
    let mut data = example.clone();
    for list in CV_LISTS {
        if let Some(field) = data.get_mut(list) {
            *field = Value::Array(Vec::new());
        }
    }
    data
}

/// Données de tous les rendus de contrôle.
///
/// # Arguments
///
/// * `example` - Valeurs d'exemple (conformes au schéma).
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `example_text` - Texte du rendu de l'exemple (choix d'un symbole
///   sentinelle absent du document).
///
/// # Returns
///
/// Rendus et leurs données JSON, dans l'ordre des contrôles.
pub fn control_renders(example: &Value, doc_type: &str, example_text: &str) -> Result<Vec<(RenderKind, String)>, AppError> {
    let shape = shape_for(doc_type);
    let lang = example["lang"].as_str().unwrap_or_default();
    let example_bytes = serde_json::to_vec(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple non sérialisables : {e}")))?;

    let mut renders = Vec::new();
    for language in SUPPORTED_LANGS {
        let mut localized = example.clone();
        localized["lang"] = Value::String(language.to_string());
        renders.push((RenderKind::Language(language), to_json(&localized)?));
    }
    renders.push((RenderKind::Blank, to_json(&blank(&shape, lang))?));
    renders.push((RenderKind::Sentinel, to_json(&sentinel(&shape, lang))?));
    renders.push((RenderKind::Sparse, bytes_to_json(cleared_data(&example_bytes, &SPARSE_FIELDS)?)?));
    if doc_type == "cv" {
        renders.push((RenderKind::EmptyLists, to_json(&empty_lists(example))?));
        // Un symbole absent du document ne peut venir que de data.list_marker.
        if let Some(marker) = pick_sentinel(example_text) {
            renders.push((RenderKind::MarkerSentinel(marker), bytes_to_json(sentinel_marker_data(&example_bytes, marker)?)?));
        }
    } else {
        renders.push((RenderKind::SparseRecipient, bytes_to_json(sparse_recipient_data(&example_bytes)?)?));
    }
    Ok(renders)
}

/// Exemple dont un seul champ secondaire est vidé, pour chaque champ :
/// désigne le champ responsable d'un séparateur orphelin.
///
/// # Arguments
///
/// * `example` - Valeurs d'exemple.
///
/// # Returns
///
/// Champ vidé et données JSON, pour chaque champ secondaire.
pub fn single_field_cleared(example: &Value) -> Result<Vec<(&'static str, String)>, AppError> {
    let example_bytes = serde_json::to_vec(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple non sérialisables : {e}")))?;
    SPARSE_FIELDS
        .into_iter()
        .map(|field| Ok((field, bytes_to_json(cleared_data(&example_bytes, &[field])?)?)))
        .collect()
}
