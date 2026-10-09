//! Logique pure du pipeline de génération de documents (sans LLM, sans
//! base, sans Typst) : budgets de longueur, sélection des entrées du profil,
//! linter de la prose, formatage des périodes et des dates, comparaison de
//! textes rendus.

pub mod budgets;
pub mod date_format;
pub mod forbidden_phrases;
pub mod lint;
pub mod period_format;
pub mod selection;
pub mod text_diff;
pub mod words;
