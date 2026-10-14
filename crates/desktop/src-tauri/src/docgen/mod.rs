//! Assemblage déterministe des documents générés : chargement des
//! références templatisées, construction des données (`cv_data`,
//! `lm_data`) depuis le profil et l'offre, libellés, compilation ajustée au
//! nombre de pages, et trace de la génération.

pub mod build_cv_data;
pub mod build_lm_data;
pub mod document_fallback;
pub mod identity;
pub mod labels_cache;
pub mod page_fit;
pub mod recipient;
pub mod reference_template;
pub mod trace;

#[cfg(test)]
mod tests;
