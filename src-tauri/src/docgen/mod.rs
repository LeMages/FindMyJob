//! Pipeline de génération de documents.
//!
//! Assemblage déterministe des documents générés : chargement des
//! références templatisées, construction des données (`cv_data`,
//! `lm_data`) depuis le profil et l'offre, libellés, compilation ajustée au
//! nombre de pages, et trace de la génération.
//!
//! Logique pure (sans LLM, sans base, sans Typst) : budgets de longueur,
//! sélection des entrées du profil, linter de la prose, formatage des
//! périodes et des dates, comparaison de textes rendus.

pub mod budgets;
pub mod build_cv_data;
pub mod build_lm_data;
pub mod date_format;
pub mod document_fallback;
pub mod example_facts;
pub mod fixed_zones;
pub mod forbidden_phrases;
pub mod guidance_consistency;
pub mod identity;
pub mod labels_cache;
pub mod lint;
pub mod page_fit;
pub mod period_format;
pub mod recipient;
pub mod reference_template;
pub mod selection;
pub mod sentinel_data;
pub mod text_diff;
pub mod trace;
pub mod words;

#[cfg(test)]
mod tests;
