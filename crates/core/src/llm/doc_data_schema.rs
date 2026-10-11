//! Schémas JSON (structured outputs) des données canoniques de documents
//! (`models::doc_data`), utilisés pour contraindre la sortie des appels LLM
//! qui produisent ou traduisent ces données.
//!
//! Les structured outputs exigent `additionalProperties: false` et des
//! propriétés toutes requises : un champ absent de la référence est donc
//! rendu par une chaîne vide ou une liste vide, jamais omis. La taille de la
//! grammaire compilée étant plafonnée par l'API, les valeurs à ensemble fixe
//! sont de simples chaînes, vérifiées côté Rust.

// Importation des crates externes
use serde_json::{json, Map, Value};

/// Schéma d'une chaîne.
fn string() -> Value {
    json!({ "type": "string" })
}

/// Schéma d'une liste de chaînes.
fn string_list() -> Value {
    json!({ "type": "array", "items": string() })
}

/// Schéma d'une liste d'objets.
fn list_of(item: Value) -> Value {
    json!({ "type": "array", "items": item })
}

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

/// Schéma de `Identity`.
fn identity_schema() -> Value {
    object(&[
        ("prenom", string()),
        ("nom", string()),
        ("email", string()),
        ("telephone", string()),
        ("adresse", string()),
        ("linkedin", string()),
        ("github", string()),
        ("portfolio", string()),
        ("permis", json!({ "type": "boolean" })),
    ])
}

/// Schéma de la langue d'un document (`"fr"` ou `"en"`). Volontairement
/// une simple chaîne : les `enum` alourdissent la grammaire compilée des
/// structured outputs, dont la taille est plafonnée par l'API ; la valeur
/// est vérifiée côté Rust.
fn lang_schema() -> Value {
    string()
}

/// Schéma de `CvLabels`.
pub fn cv_labels_schema() -> Value {
    object(&[
        ("section_profile", string()),
        ("section_skills", string()),
        ("section_experiences", string()),
        ("section_projects", string()),
        ("section_formations", string()),
        ("section_langues", string()),
        ("interets_label", string()),
        ("permis", string()),
        ("present", string()),
        ("months_short", string_list()),
    ])
}

/// Schéma de `LmLabels`.
pub fn lm_labels_schema() -> Value {
    object(&[
        ("subject_prefix", string()),
        ("salutation", string()),
        ("closing", string()),
        ("date_prefix", string()),
    ])
}

/// Schéma de `CvExampleData` (contenu d'un CV de référence + métadonnées).
pub fn cv_example_data_schema() -> Value {
    // Chaîne libre plutôt qu'`enum` (voir `lang_schema`) : `"month"` ou
    // `"year"`, vérifié à la désérialisation en `PeriodGranularity`.
    let granularity = string();
    object(&[
        ("identity", identity_schema()),
        ("lang", lang_schema()),
        ("title", string()),
        ("summary", string()),
        (
            "skills_groups",
            list_of(object(&[("category", string()), ("skills", string_list())])),
        ),
        (
            "experiences",
            list_of(object(&[
                ("poste", string()),
                ("entreprise", string()),
                ("lieu", string()),
                ("period", string()),
                ("bullets", string_list()),
            ])),
        ),
        (
            "projects",
            list_of(object(&[
                ("titre", string()),
                ("pour_qui", string()),
                ("period", string()),
                ("bullets", string_list()),
            ])),
        ),
        (
            "formations",
            list_of(object(&[
                ("titre", string()),
                ("ecole", string()),
                ("period", string()),
                ("details", string_list()),
            ])),
        ),
        ("langues", list_of(object(&[("nom", string()), ("level", string())]))),
        ("interets", string_list()),
        ("labels", cv_labels_schema()),
        ("detected_lang", lang_schema()),
        (
            "period_style",
            object(&[
                ("experiences", granularity.clone()),
                ("projects", granularity.clone()),
                ("formations", granularity),
            ]),
        ),
    ])
}

/// Schéma de `LmExampleData` (contenu d'une lettre de référence + langue détectée).
pub fn lm_example_data_schema() -> Value {
    object(&[
        ("identity", identity_schema()),
        ("lang", lang_schema()),
        (
            "recipient",
            object(&[
                ("company", string()),
                ("location", string()),
                ("contact_name", string()),
                ("contact_role", string()),
            ]),
        ),
        ("date", string()),
        ("subject", string()),
        ("paragraphs", string_list()),
        ("labels", lm_labels_schema()),
        ("detected_lang", lang_schema()),
    ])
}
