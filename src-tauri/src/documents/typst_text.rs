//! Extraction du texte rendu d'un document Typst compilé, dans l'ordre de
//! mise en page, pour comparer deux rendus mot à mot.

// Importation des crates externes
use typst_layout::PagedDocument;
use typst_library::layout::{Frame, FrameItem};

/// Profondeur maximale d'imbrication de groupes parcourue : bien au-delà de
/// ce que produit un CV ou une lettre, elle borne la récursion.
pub(crate) const MAX_GROUP_DEPTH: usize = 64;

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

/// Texte lisible d'un rendu, pour un prompt : les mots coupés en fin de
/// ligne (césure, rendue par un trait d'union conditionnel) sont recollés
/// et les espaces multiples réduites à une seule.
///
/// # Arguments
///
/// * `text` - Texte rendu (`extract_text`).
///
/// # Returns
///
/// Texte d'un seul tenant, mots séparés par une espace.
pub fn reading_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // Vrai juste après un trait d'union conditionnel : le mot suivant est
    // la suite du mot coupé.
    let mut glue = false;
    for word in text.split_whitespace() {
        let (stem, hyphenated) = match word.strip_suffix(SOFT_HYPHEN) {
            Some(stem) => (stem, true),
            None => (word, false),
        };
        if !stem.is_empty() && !out.is_empty() && !glue {
            out.push(' ');
        }
        out.push_str(stem);
        glue = hyphenated || (glue && stem.is_empty());
    }
    out
}

/// Trait d'union conditionnel, rendu par Typst à une césure.
const SOFT_HYPHEN: char = '\u{ad}';

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

#[cfg(test)]
mod tests {
    use super::reading_text;

    #[test]
    fn cesure_recollee_et_espaces_reduites() {
        assert_eq!(reading_text("intéresse parti \u{ad} culièrement. Fin"), "intéresse particulièrement. Fin");
        assert_eq!(reading_text("parti\u{ad} culièrement"), "particulièrement");
        assert_eq!(reading_text("tableaux de bord  utiles"), "tableaux de bord utiles");
    }
}
