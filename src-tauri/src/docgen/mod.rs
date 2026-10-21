//! Logique pure de la génération de documents (sans LLM, sans base, sans
//! fichier, sans Typst) : construction des données (`cv_data`, `lm_data`)
//! depuis le profil et l'offre, budgets de longueur, sélection des entrées
//! du profil, textes des entrées et validation de leur traduction,
//! traduction des textes de la référence (découpage et validation),
//! linter de la prose et zones [FIXE], normalisation des tirets du texte
//! rendu, données sentinelles, formatage des périodes et des dates,
//! comparaison de textes rendus.
//!
//! Ce qui lit la base, écrit des fichiers, appelle le LLM ou compile du
//! Typst (références, caches de traduction, ajustement aux pages, trace)
//! vit dans `documents`. `docgen` ne dépend pas de `llm` : la dépendance va
//! dans un seul sens (`llm` → `docgen`).

pub mod budgets;
pub mod build_cv_data;
pub mod build_lm_data;
pub mod dash_normalization;
pub mod data_normalization;
pub mod date_format;
pub mod document_fallback;
pub mod entry_texts;
pub mod example_facts;
pub mod fixed_zones;
pub mod forbidden_phrases;
pub mod identity;
pub mod lint;
pub mod period_format;
pub mod recipient;
pub mod reference_texts_translation;
pub mod selection;
pub mod sentinel_data;
pub mod text_diff;
pub mod translation_check;
pub mod words;

#[cfg(test)]
mod tests;
