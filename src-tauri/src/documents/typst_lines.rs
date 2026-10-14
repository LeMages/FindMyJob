//! Lignes de texte d'un document Typst compilé, avec leur position sur la
//! page : ce qu'il faut pour savoir où commence un paragraphe rendu, sans
//! dépendre de la façon dont la source sépare ses paragraphes (ligne vide,
//! `#v`, `parbreak()`, `\`, blocs).
//!
//! Les frames Typst ne gardent pas toujours la notion de paragraphe : un
//! texte seul dans un bloc ou des lignes séparées par `\` ne forment pas
//! d'élément `par`. Le modèle est donc visuel (lignes, baselines, bords),
//! complété par les débuts d'éléments `par` taggés, quand il y en a.

// Importation des crates externes
use typst_layout::PagedDocument;
use typst_library::introspection::Tag;
use typst_library::layout::{Frame, FrameItem, Point, Transform};
use typst_library::model::ParElem;
use typst_library::text::TextItem;

/// Profondeur maximale d'imbrication de groupes parcourue (voir `typst_text`).
const MAX_GROUP_DEPTH: usize = 64;
/// Écart de baseline maximal entre deux fragments d'une même ligne, en
/// fraction de la taille de police.
const SAME_LINE_TOLERANCE: f64 = 0.3;

/// Ligne de texte rendue.
///
/// # Attributes
///
/// * `page` - Index de la page (à partir de 0).
/// * `baseline` - Ordonnée de la baseline, en pt depuis le haut de la page.
/// * `left` - Bord gauche du premier fragment, en pt.
/// * `right` - Bord droit du dernier fragment, en pt.
/// * `size` - Plus grande taille de police de la ligne, en pt.
/// * `words` - Mots de la ligne, dans l'ordre (fragments découpés aux espaces).
/// * `first_word_width` - Largeur du premier mot, en pt.
/// * `starts_par` - Vrai si un élément `par` commence juste avant la ligne.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub page: usize,
    pub baseline: f64,
    pub left: f64,
    pub right: f64,
    pub size: f64,
    pub words: Vec<String>,
    pub first_word_width: f64,
    pub starts_par: bool,
}

/// Fragment de texte positionné, avant regroupement en lignes.
struct Fragment {
    page: usize,
    baseline: f64,
    left: f64,
    right: f64,
    size: f64,
    text: String,
    first_word_width: f64,
}

/// Ce que le parcours des frames relève : fragments et débuts de `par`
/// (page, ordonnée).
#[derive(Default)]
struct Collected {
    fragments: Vec<Fragment>,
    par_starts: Vec<(usize, f64)>,
}

/// Extrait les lignes de texte de toutes les pages d'un document compilé,
/// triées par page puis de haut en bas.
///
/// # Arguments
///
/// * `doc` - Document compilé.
///
/// # Returns
///
/// Lignes du document.
pub fn extract_lines(doc: &PagedDocument) -> Vec<TextLine> {
    let mut collected = Collected::default();
    for (page, content) in doc.pages().iter().enumerate() {
        collect_frame(&content.frame, page, Transform::identity(), 0, &mut collected);
    }
    let mut lines = group_lines(collected.fragments);
    mark_par_starts(&mut lines, &collected.par_starts);
    lines
}

/// Parcourt un cadre et ses sous-groupes, en cumulant leurs transformations.
fn collect_frame(frame: &Frame, page: usize, ts: Transform, depth: usize, out: &mut Collected) {
    if depth > MAX_GROUP_DEPTH {
        return;
    }
    for (pos, item) in frame.items() {
        let at = ts.pre_concat(Transform::translate(pos.x, pos.y));
        match item {
            FrameItem::Group(group) => collect_frame(&group.frame, page, at.pre_concat(group.transform), depth + 1, out),
            FrameItem::Text(text) => out.fragments.extend(fragment(text, page, at)),
            FrameItem::Tag(Tag::Start(elem, _)) if elem.is::<ParElem>() => {
                out.par_starts.push((page, Point::zero().transform(at).y.to_pt()));
            }
            _ => {}
        }
    }
}

/// Fragment positionné d'un `TextItem` (`None` s'il ne contient que des espaces).
fn fragment(text: &TextItem, page: usize, at: Transform) -> Option<Fragment> {
    if text.text.trim().is_empty() {
        return None;
    }
    let origin = Point::zero().transform(at);
    let end = Point::with_x(text.width()).transform(at);
    Some(Fragment {
        page,
        baseline: origin.y.to_pt(),
        left: origin.x.to_pt(),
        right: end.x.to_pt(),
        size: text.size.to_pt(),
        text: text.text.to_string(),
        first_word_width: first_word_width(text),
    })
}

/// Largeur du premier mot d'un fragment : avances des glyphes jusqu'au
/// premier espace qui suit un caractère visible.
fn first_word_width(text: &TextItem) -> f64 {
    let mut width = 0.0;
    let mut started = false;
    for glyph in &text.glyphs {
        let is_space = text.text.get(glyph.range()).is_some_and(|s| s.chars().all(char::is_whitespace));
        if is_space && started {
            break;
        }
        started |= !is_space;
        if started {
            width += glyph.x_advance.at(text.size).to_pt();
        }
    }
    width
}

/// Regroupe les fragments en lignes : même page et baselines voisines.
fn group_lines(mut fragments: Vec<Fragment>) -> Vec<TextLine> {
    fragments.sort_by(|a, b| (a.page, a.baseline).partial_cmp(&(b.page, b.baseline)).unwrap_or(std::cmp::Ordering::Equal));
    let mut groups: Vec<Vec<Fragment>> = Vec::new();
    for fragment in fragments {
        match groups.last_mut() {
            Some(group) if same_line(&group[0], &fragment) => group.push(fragment),
            _ => groups.push(vec![fragment]),
        }
    }
    groups.into_iter().map(into_line).collect()
}

/// Indique si un fragment est sur la ligne ouverte par `first`.
fn same_line(first: &Fragment, other: &Fragment) -> bool {
    let tolerance = SAME_LINE_TOLERANCE * first.size.max(other.size);
    first.page == other.page && (other.baseline - first.baseline).abs() <= tolerance
}

/// Construit une ligne à partir de ses fragments, triés de gauche à droite.
fn into_line(mut group: Vec<Fragment>) -> TextLine {
    group.sort_by(|a, b| a.left.partial_cmp(&b.left).unwrap_or(std::cmp::Ordering::Equal));
    let first = &group[0];
    TextLine {
        page: first.page,
        baseline: first.baseline,
        left: first.left,
        right: group.iter().map(|f| f.right).fold(f64::MIN, f64::max),
        size: group.iter().map(|f| f.size).fold(0.0, f64::max),
        words: group.iter().flat_map(|f| f.text.split_whitespace().map(str::to_string)).collect(),
        first_word_width: first.first_word_width,
        starts_par: false,
    }
}

/// Marque la première ligne sous chaque début de `par` de la même page.
fn mark_par_starts(lines: &mut [TextLine], par_starts: &[(usize, f64)]) {
    for &(page, y) in par_starts {
        if let Some(line) = lines.iter_mut().find(|l| l.page == page && l.baseline > y) {
            line.starts_par = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::typst::compile_document;

    /// Compile une source et retourne ses lignes.
    fn lines_of(source: &str) -> Vec<TextLine> {
        let (root, fonts) = (std::env::temp_dir(), std::env::temp_dir().join("findmyjob-no-fonts"));
        compile_document(source.to_string(), root, fonts).expect("compilation").lines
    }

    #[test]
    fn fragments_de_styles_differents_sur_une_seule_ligne() {
        let lines = lines_of("Un *mot* en gras et _un autre_.");
        assert_eq!(lines.len(), 1, "{lines:?}");
        let words = lines[0].words.join(" ");
        assert!(words.starts_with("Un mot en gras et un autre"), "{words}");
        assert!(lines[0].right > lines[0].left);
    }

    #[test]
    fn debut_de_paragraphe_marque() {
        let lines = lines_of("Premier paragraphe.\n\nSecond paragraphe. \\\nMême paragraphe.");
        let starts: Vec<bool> = lines.iter().map(|l| l.starts_par).collect();
        assert_eq!(starts, [true, true, false], "{lines:?}");
        assert!(lines[1].baseline > lines[0].baseline);
    }
}
