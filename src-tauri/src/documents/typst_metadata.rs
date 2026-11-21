//! Lecture de la métadonnée `#metadata((...)) <findmyjob>` d'un document
//! Typst compilé, par l'introspecteur (équivalent de `typst query`, sans
//! CLI ni LLM).

// Importation des crates externes
use typst_layout::PagedDocument;
use typst_library::introspection::{Introspector, MetadataElem};

// Importation des modules internes
use crate::models::METADATA_LABEL;

/// Valeurs de toutes les métadonnées `<findmyjob>` du document, converties
/// en JSON, dans l'ordre du document. Une valeur que Typst ne sait pas
/// convertir (contenu, fonction…) donne une erreur décrite.
///
/// # Arguments
///
/// * `doc` - Document compilé.
///
/// # Returns
///
/// Une entrée par élément étiqueté `<findmyjob>` (vide si aucun).
pub fn findmyjob_metadata(doc: &PagedDocument) -> Vec<Result<serde_json::Value, String>> {
    doc.introspector()
        .query_labelled()
        .iter()
        .filter(|content| content.label().is_some_and(|label| label.resolve().as_str() == METADATA_LABEL))
        .map(|content| match content.to_packed::<MetadataElem>() {
            Some(metadata) => serde_json::to_value(&metadata.value).map_err(|e| e.to_string()),
            None => Err(format!("<{METADATA_LABEL}> is not attached to a metadata element")),
        })
        .collect()
}
