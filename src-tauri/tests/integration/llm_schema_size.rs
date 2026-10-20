//! Tests : taille des schémas de structured output envoyés à l'API.
//!
//! L'API refuse un schéma dont la grammaire compilée est trop grande, sans
//! documenter la limite. Ces tests cassent dès qu'un schéma envoyé dépasse
//! la limite mesurée (`llm::schema_size`), au lieu de le découvrir à l'import
//! d'une référence (fragilité F1 de l'audit).

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::common;
use findmyjob_lib::llm::cv_writing::cv_content_schema;
use findmyjob_lib::llm::doc_data_schema::{cv_labels_schema, lm_labels_schema};
use findmyjob_lib::llm::lm_writing::lm_content_schema;
use findmyjob_lib::llm::matching::matching_schema;
use findmyjob_lib::llm::offer::offer_analysis_schema;
use findmyjob_lib::llm::profile_translation::translation_schema;
use findmyjob_lib::llm::schema_size::{schema_size, SchemaSize};
use findmyjob_lib::llm::scraping_extract::scraped_offer_schema;
use findmyjob_lib::llm::templatize_example::example_output_schema;

/// Nombre de projets du profil de test : la liste des identifiants de
/// projets fait partie du schéma du matching.
const PROJECT_COUNT: i64 = 30;

/// Schémas envoyés à l'API, nommés.
fn sent_schemas() -> Vec<(&'static str, Value)> {
    let mut profile = common::profile();
    profile.projets = (1..=PROJECT_COUNT).map(|id| common::projet(id, "Projet")).collect();
    vec![
        ("analyse d'offre", offer_analysis_schema()),
        ("matching", matching_schema(&profile, 3)),
        ("libellés CV", cv_labels_schema()),
        ("libellés lettre", lm_labels_schema()),
        ("traduction du profil", translation_schema()),
        ("rédaction CV", cv_content_schema()),
        ("rédaction lettre", lm_content_schema()),
        ("extraction d'offre", scraped_offer_schema()),
        ("templatisation, données CV", example_output_schema("cv").expect("type connu")),
        ("templatisation, données lettre", example_output_schema("lettre_motivation").expect("type connu")),
    ]
}

#[test]
fn limite_calibree_sur_les_mesures_de_l_api() {
    let refused = SchemaSize { bytes: 3_225, type_nodes: 70 };
    let accepted = SchemaSize { bytes: 3_032, type_nodes: 67 };
    assert!(!refused.fits_api_grammar());
    assert!(accepted.fits_api_grammar());
}

#[test]
fn schemas_envoyes_sous_la_limite() {
    let too_large: Vec<String> = sent_schemas()
        .into_iter()
        .map(|(name, schema)| (name, schema_size(&schema)))
        .filter(|(_, size)| !size.fits_api_grammar())
        .map(|(name, size)| format!("{name} : {size:?}"))
        .collect();
    assert!(too_large.is_empty(), "schémas au-dessus de la limite de l'API : {too_large:?}");
}
