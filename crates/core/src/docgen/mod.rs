//! Logique pure du pipeline de génération de documents (sans LLM, sans
//! base, sans Typst) : budgets de longueur, sélection des entrées du profil,
//! linter de la prose, formatage des périodes et des dates, comparaison de
//! textes rendus.

pub mod budgets;
pub mod date_format;
pub mod example_facts;
pub mod fixed_zones;
pub mod forbidden_phrases;
pub mod guidance_consistency;
pub mod lint;
pub mod period_format;
pub mod selection;
pub mod sentinel_data;
pub mod text_diff;
pub mod words;
