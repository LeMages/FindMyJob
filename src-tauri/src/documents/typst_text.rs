//! Extraction du texte rendu d'un document Typst compilé, dans l'ordre de
//! mise en page, pour comparer deux rendus mot à mot.

// Importation des crates externes
use typst_layout::PagedDocument;
use typst_library::layout::{Frame, FrameItem};

/// Profondeur maximale d'imbrication de groupes parcourue : bien au-delà de
/// ce que produit un CV ou une lettre, elle borne la récursion.
const MAX_GROUP_DEPTH: usize = 64;

/// Extrait le texte rendu de toutes les pages d'un document compilé.
///
/// Chaque fragment de texte (`TextItem`) est séparé du suivant par une
/// espace : la comparaison se fait ensuite mot à mot, insensible aux espaces.
///
/// # Arguments
///
/// * `doc` - Document compilé.
///
/// # Returns
///
/// Texte rendu, fragments séparés par des espaces.
pub fn extract_text(doc: &PagedDocument) -> String {
    let mut fragments = Vec::new();
    for page in doc.pages() {
        collect_frame_text(&page.frame, 0, &mut fragments);
    }
    fragments.join(" ")
}

/// Ajoute à `fragments` le texte d'un cadre et de ses sous-groupes.
fn collect_frame_text(frame: &Frame, depth: usize, fragments: &mut Vec<String>) {
    if depth > MAX_GROUP_DEPTH {
        return;
    }
    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => collect_frame_text(&group.frame, depth + 1, fragments),
            FrameItem::Text(text) => fragments.push(text.text.to_string()),
            _ => {}
        }
    }
}
