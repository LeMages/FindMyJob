//! Titres de section (`heading`) d'un document Typst compilé, et présence
//! d'un contenu après chacun : ce qu'il faut pour repérer un titre resté
//! affiché alors que sa section est vide.
//!
//! Le parcours suit l'ordre des éléments dans les frames (ordre de mise en
//! page : une grille ou des colonnes rendent leurs cellules l'une après
//! l'autre). Un texte placé hors du flux (`place`, en-tête ou pied de page)
//! peut donc compter comme contenu de la section qui le précède dans ce
//! parcours.

// Importation des crates externes
use typst_layout::PagedDocument;
use typst_library::introspection::{Location, Tag};
use typst_library::layout::{Frame, FrameItem};
use typst_library::model::HeadingElem;

// Importation des modules internes
use crate::documents::typst_text::MAX_GROUP_DEPTH;

/// Titre de section rendu.
///
/// # Attributes
///
/// * `title` - Texte du titre.
/// * `has_content` - Vrai si un texte est rendu entre la fin du titre et le
///   titre suivant (ou la fin du document).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingSection {
    pub title: String,
    pub has_content: bool,
}

/// État du parcours : titres trouvés, titre en cours de rendu.
#[derive(Default)]
struct Walk {
    sections: Vec<HeadingSection>,
    inside: Option<Location>,
}

/// Relève les titres de section d'un document compilé, dans l'ordre.
///
/// # Arguments
///
/// * `doc` - Document compilé.
///
/// # Returns
///
/// Un `HeadingSection` par `heading` rendu.
pub fn extract_headings(doc: &PagedDocument) -> Vec<HeadingSection> {
    let mut walk = Walk::default();
    for page in doc.pages() {
        collect(&page.frame, 0, &mut walk);
    }
    walk.sections
}

/// Parcourt un cadre et ses sous-groupes.
fn collect(frame: &Frame, depth: usize, walk: &mut Walk) {
    if depth > MAX_GROUP_DEPTH {
        return;
    }
    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => collect(&group.frame, depth + 1, walk),
            FrameItem::Tag(Tag::Start(elem, _)) => {
                if let Some(heading) = elem.to_packed::<HeadingElem>() {
                    // Le corps seul : `plain_text` de l'élément y ajoute son supplément (« Section »).
                    let title = heading.body.plain_text().trim().to_string();
                    walk.sections.push(HeadingSection { title, has_content: false });
                    walk.inside = elem.location();
                }
            }
            FrameItem::Tag(Tag::End(location, ..)) if walk.inside == Some(*location) => walk.inside = None,
            FrameItem::Text(text) if walk.inside.is_none() && !text.text.trim().is_empty() => {
                if let Some(last) = walk.sections.last_mut() {
                    last.has_content = true;
                }
            }
            _ => {}
        }
    }
}
