//! Contrat FindMyJob d'une référence (CV ou lettre) : schéma des valeurs
//! d'exemple, règles du fichier, métadonnée, emplacements rédigés, rendus
//! de contrôle et validation déterministe à l'import.

pub mod example_schema;
pub mod metadata;
pub mod render_checks;
pub mod render_data;
pub mod slots;
pub mod source_rules;
pub mod validate;

pub use validate::{validate_reference, ValidatedReference};
