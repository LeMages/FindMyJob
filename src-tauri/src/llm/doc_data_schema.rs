//! Brique commune des schémas JSON de sortie structurée (structured
//! outputs) des appels LLM : objet strict dont toutes les propriétés sont
//! requises.
//!
//! Les structured outputs exigent `additionalProperties: false` et des
//! propriétés toutes requises. La taille de la grammaire compilée étant
//! plafonnée par l'API, les valeurs à ensemble fixe sont de simples chaînes,
//! vérifiées côté Rust.

// Importation des crates externes
use serde_json::{json, Map, Value};

/// Schéma d'un objet strict dont toutes les propriétés sont requises.
///
/// # Arguments
///
/// * `props` - Couples (nom de propriété, schéma), dans l'ordre d'affichage.
///
/// # Returns
///
/// Schéma `object` avec `required` = toutes les propriétés et
/// `additionalProperties: false`.
pub(crate) fn object(props: &[(&str, Value)]) -> Value {
    let mut properties = Map::new();
    for (name, schema) in props {
        properties.insert((*name).to_string(), schema.clone());
    }
    let required: Vec<&str> = props.iter().map(|(name, _)| *name).collect();
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}
