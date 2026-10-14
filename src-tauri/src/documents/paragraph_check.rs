//! Contrôle structurel des paragraphes d'une lettre templatisée : chaque
//! entrée de `paragraphs` doit être un paragraphe du corps de la référence,
//! et être rendue comme un paragraphe distinct.
//!
//! Tout est dérivé des rendus (`typst_lines`), jamais de la façon dont la
//! source sépare ses paragraphes. Entre deux lignes consécutives du corps,
//! la rupture est « dure » (nouveau paragraphe) si :
//! - un élément `par` commence (ligne vide, `parbreak()`, `#v`…) ;
//! - l'écart de baselines dépasse le pas de ligne du corps ;
//! - la ligne du dessus est courte alors que le premier mot de la suivante
//!   y aurait tenu (`\`, bloc).
//!
//! Sinon c'est un retour à la ligne automatique. Le corps de la référence
//! va du premier mot de `paragraphs[0]` au dernier de la dernière entrée :
//! son nombre de paragraphes ne dépend pas du découpage choisi par le LLM.

// Importation des modules internes
use crate::docgen::text_diff::{normalize_quotes, word_key};
use crate::documents::typst_lines::TextLine;

/// Nombre de mots qui situent le début ou la fin d'une entrée dans un rendu.
const ANCHOR_WORDS: usize = 3;
/// Écart de baselines au-delà du pas de ligne qui marque un paragraphe, en pt.
const GAP_TOLERANCE: f64 = 1.0;
/// Marge sur le bord droit du corps, en pt.
const EDGE_TOLERANCE: f64 = 1.0;
/// Largeur d'une espace, en fraction de la taille de police (borne haute).
const SPACE_EM: f64 = 0.35;
/// Écart d'espacement vertical toléré entre le rendu et la référence, en pt.
const SPACING_TOLERANCE: f64 = 1.0;
/// Traits d'union qu'une coupure de mot laisse en fin de ligne.
const HYPHENS: [char; 3] = ['-', '\u{2010}', '\u{00AD}'];

/// Mot d'un rendu, situé dans ses lignes.
struct Token {
    raw: String,
    key: String,
    line: usize,
    first_in_line: bool,
    last_in_line: bool,
}

/// Rendu préparé pour le contrôle : lignes et mots.
struct Layout<'a> {
    lines: &'a [TextLine],
    tokens: Vec<Token>,
}

/// Situation d'une entrée de `paragraphs` dans un rendu.
#[derive(Clone, Copy, PartialEq)]
enum Start {
    /// Premier mot trouvé, à cet index de mot.
    At(usize),
    /// Premier mot collé au dernier mot de l'entrée précédente, à cet index.
    Glued(usize),
    /// Introuvable (texte différent : l'écart de texte le signale déjà).
    Missing,
}

/// Contrôle la structure en paragraphes d'une lettre.
///
/// # Arguments
///
/// * `reference` - Lignes du rendu de la référence.
/// * `rendered` - Lignes du rendu du template avec ses données d'exemple.
/// * `paragraphs` - Entrées `paragraphs` des données d'exemple.
///
/// # Returns
///
/// Problèmes à renvoyer en feedback (vide si la structure est conforme, ou
/// si le corps est introuvable dans la référence).
pub fn paragraph_problems(reference: &[TextLine], rendered: &[TextLine], paragraphs: &[String]) -> Vec<String> {
    let paragraphs: Vec<Vec<String>> =
        paragraphs.iter().map(|p| p.split_whitespace().map(str::to_string).collect()).filter(|w: &Vec<String>| !w.is_empty()).collect();
    if paragraphs.is_empty() {
        return Vec::new();
    }
    let reference = Layout::new(reference);
    let rendered = Layout::new(rendered);
    let ref_starts = reference.locate(&paragraphs);
    let Some(ref_body) = reference.body(&ref_starts, &paragraphs) else {
        return Vec::new();
    };
    let ref_hard = reference.hard_breaks(ref_body);

    // ------------------------------------------------------------------
    // 1. Nombre d'entrées contre nombre de paragraphes de la référence
    // ------------------------------------------------------------------
    let mut problems = Vec::new();
    let expected = 1 + ref_hard.iter().filter(|&&hard| hard).count();
    if paragraphs.len() != expected {
        problems.push(count_problem(expected, paragraphs.len()));
    }

    // ------------------------------------------------------------------
    // 2. Chaque frontière : dans la référence, puis dans le rendu
    // ------------------------------------------------------------------
    let ref_check = BoundaryCheck { layout: &reference, starts: &ref_starts, body: ref_body, hard: &ref_hard };
    let rendered_starts = rendered.locate(&paragraphs);
    let rendered_body = rendered.body(&rendered_starts, &paragraphs);
    let rendered_hard = rendered_body.map(|body| rendered.hard_breaks(body)).unwrap_or_default();
    let rendered_check = rendered_body.map(|body| BoundaryCheck {
        layout: &rendered,
        starts: &rendered_starts,
        body,
        hard: &rendered_hard,
    });
    for i in 1..paragraphs.len() {
        problems.extend(ref_check.reference_problem(i));
        let reference_gap = ref_check.gap_before(i);
        problems.extend(rendered_problem(&rendered, &rendered_starts, rendered_check.as_ref(), i, reference_gap));
    }

    // ------------------------------------------------------------------
    // 3. Paragraphes en trop dans le rendu (une entrée rendue en plusieurs)
    // ------------------------------------------------------------------
    if problems.is_empty() {
        let rendered_count = 1 + rendered_hard.iter().filter(|&&hard| hard).count();
        if rendered_body.is_some() && rendered_count != expected {
            problems.push(format!(
                "Rendu avec example_data, le corps de la lettre compte {rendered_count} paragraphe(s), la référence \
                 {expected} : chaque entrée de paragraphs est rendue comme un seul paragraphe, sans saut de \
                 paragraphe à l'intérieur."
            ));
        }
    }
    problems
}

/// Problème d'une frontière dans le rendu avec les données d'exemple.
fn rendered_problem(
    rendered: &Layout<'_>,
    starts: &[Start],
    check: Option<&BoundaryCheck<'_>>,
    i: usize,
    reference_gap: Option<f64>,
) -> Option<String> {
    match starts[i] {
        Start::Glued(_) => Some(format!(
            "Les paragraphes paragraphs[{}] et paragraphs[{i}] sont rendus sans séparateur (« {} ») : chaque entrée \
             de paragraphs doit être un paragraphe distinct, séparé du précédent comme dans l'original.",
            i - 1,
            rendered.tokens[starts[i].index()?].raw
        )),
        Start::Missing => None,
        Start::At(index) => {
            let check = check?;
            if !check.starts_paragraph(index) {
                return Some(joined_problem(i, rendered.tokens[index].first_in_line));
            }
            spacing_problem(i, check.gap_before(i)?, reference_gap?)
        }
    }
}

/// Feedback d'une entrée rendue à la suite de la précédente.
fn joined_problem(i: usize, on_new_line: bool) -> String {
    let how = if on_new_line {
        "dans le même paragraphe (simple retour à la ligne)"
    } else {
        "sur la même ligne (paragraphes joints en un seul bloc)"
    };
    format!(
        "paragraphs[{i}] est rendu à la suite de paragraphs[{}] {how} : chaque entrée de paragraphs doit être un \
         paragraphe distinct. N'utilise ni [#p], ni {{ p }}, ni text(p) enchaînés, ni .join() : sépare les entrées \
         comme l'original sépare ses paragraphes.",
        i - 1
    )
}

/// Feedback d'un espacement vertical différent de la référence.
fn spacing_problem(i: usize, rendered_gap: f64, reference_gap: f64) -> Option<String> {
    ((rendered_gap - reference_gap).abs() > SPACING_TOLERANCE).then(|| {
        format!(
            "Entre paragraphs[{}] et paragraphs[{i}], l'espacement vertical rendu ({rendered_gap:.1} pt entre les \
             lignes de base) diffère de la référence ({reference_gap:.1} pt) : reprends le séparateur et \
             l'espacement de l'original entre deux paragraphes.",
            i - 1
        )
    })
}

/// Feedback d'un nombre d'entrées différent du nombre de paragraphes.
fn count_problem(expected: usize, found: usize) -> String {
    format!(
        "Le corps de la référence compte {expected} paragraphe(s), mais paragraphs a {found} entrée(s). Une entrée \
         de paragraphs = un paragraphe de l'original (séparé par une ligne vide, #v, parbreak(), \\ ou un bloc), \
         jamais une zone balisée : plusieurs zones d'un même paragraphe restent dans une seule entrée, leur \
         découpage va dans guidance.zones."
    )
}

impl Start {
    /// Index du premier mot, s'il a été trouvé.
    fn index(self) -> Option<usize> {
        match self {
            Start::At(index) | Start::Glued(index) => Some(index),
            Start::Missing => None,
        }
    }
}

/// Frontières d'un rendu : début des entrées, corps et ruptures dures.
///
/// # Attributes
///
/// * `body` - Lignes du corps, première et dernière incluses.
/// * `hard` - Pour chaque ligne du corps après la première, vrai si la
///   rupture avec la précédente ouvre un paragraphe.
struct BoundaryCheck<'a> {
    layout: &'a Layout<'a>,
    starts: &'a [Start],
    body: (usize, usize),
    hard: &'a [bool],
}

impl BoundaryCheck<'_> {
    /// Indique si le mot `index` ouvre un paragraphe du corps.
    fn starts_paragraph(&self, index: usize) -> bool {
        let token = &self.layout.tokens[index];
        let (first, last) = self.body;
        token.first_in_line && token.line > first && token.line <= last && self.hard[token.line - first - 1]
    }

    /// Problème d'une entrée qui commence au milieu d'un paragraphe de la référence.
    fn reference_problem(&self, i: usize) -> Option<String> {
        let Start::At(index) = self.starts[i] else { return None };
        (!self.starts_paragraph(index)).then(|| {
            format!(
                "paragraphs[{i}] commence au milieu d'un paragraphe de la référence (« {} ») : une entrée de \
                 paragraphs = un paragraphe entier de l'original, jamais une zone balisée. Les zones d'un même \
                 paragraphe restent dans une seule entrée, leur découpage va dans guidance.zones.",
                self.layout.tokens[index].raw
            )
        })
    }

    /// Écart de baselines entre la dernière ligne de l'entrée `i - 1` et la
    /// première de l'entrée `i`, si elles sont sur la même page et que `i`
    /// ouvre un paragraphe.
    fn gap_before(&self, i: usize) -> Option<f64> {
        let index = self.starts[i].index()?;
        if index == 0 || !self.starts_paragraph(index) {
            return None;
        }
        let lines = self.layout.lines;
        let (above, below) = (&lines[self.layout.tokens[index - 1].line], &lines[self.layout.tokens[index].line]);
        (above.page == below.page).then_some(below.baseline - above.baseline)
    }
}

impl<'a> Layout<'a> {
    /// Découpe les lignes en mots situés.
    fn new(lines: &'a [TextLine]) -> Self {
        let mut tokens = Vec::new();
        for (line_index, line) in lines.iter().enumerate() {
            let count = line.words.len();
            tokens.extend(line.words.iter().enumerate().map(|(i, word)| Token {
                raw: word.clone(),
                key: key(word),
                line: line_index,
                first_in_line: i == 0,
                last_in_line: i + 1 == count,
            }));
        }
        Self { lines, tokens }
    }

    /// Situe le début de chaque entrée, dans l'ordre du document.
    fn locate(&self, paragraphs: &[Vec<String>]) -> Vec<Start> {
        let mut starts = Vec::with_capacity(paragraphs.len());
        let mut cursor = 0;
        for (i, words) in paragraphs.iter().enumerate() {
            let previous_last = i.checked_sub(1).and_then(|p| paragraphs[p].last());
            let anchor = self.find_words(cursor, &words[..words.len().min(ANCHOR_WORDS)]);
            let glued = previous_last.and_then(|last| self.find_glued(cursor.saturating_sub(1), last, &words[0]));
            let start = match (anchor, glued) {
                (Some(a), Some(g)) if g < a => Start::Glued(g),
                (Some(a), _) => Start::At(a),
                (None, Some(g)) => Start::Glued(g),
                (None, None) => Start::Missing,
            };
            if let Some(index) = start.index() {
                cursor = index + 1;
            }
            starts.push(start);
        }
        starts
    }

    /// Lignes du corps : de la ligne du premier mot de la première entrée à
    /// celle du dernier mot de la dernière.
    fn body(&self, starts: &[Start], paragraphs: &[Vec<String>]) -> Option<(usize, usize)> {
        let first = starts.first()?.index()?;
        let last_start = starts.iter().rev().find_map(|s| s.index())?;
        let last_words = paragraphs.last()?;
        let tail = &last_words[last_words.len().saturating_sub(ANCHOR_WORDS)..];
        let end = self.find_words_end(last_start, tail)?;
        Some((self.tokens[first].line, self.tokens[end - 1].line))
    }

    /// Ruptures dures entre les lignes consécutives du corps.
    fn hard_breaks(&self, (first, last): (usize, usize)) -> Vec<bool> {
        let body = &self.lines[first..=last];
        let pitch = body
            .windows(2)
            .filter(|pair| pair[0].page == pair[1].page)
            .map(|pair| pair[1].baseline - pair[0].baseline)
            .fold(f64::INFINITY, f64::min);
        let right_edge = body.iter().map(|l| l.right).fold(f64::MIN, f64::max);
        body.windows(2).map(|pair| is_hard_break(&pair[0], &pair[1], pitch, right_edge)).collect()
    }

    /// Premier index ≥ `from` où commence la suite de mots `words`.
    fn find_words(&self, from: usize, words: &[String]) -> Option<usize> {
        (from..self.tokens.len()).find(|&start| self.match_words(start, words).is_some())
    }

    /// Index qui suit la première occurrence de `words` à partir de `from`.
    fn find_words_end(&self, from: usize, words: &[String]) -> Option<usize> {
        (from..self.tokens.len()).find_map(|start| self.match_words(start, words))
    }

    /// Mot égal à `last` suivi de `first` sans séparateur, à partir de `from`.
    fn find_glued(&self, from: usize, last: &str, first: &str) -> Option<usize> {
        let glued = normalize_quotes(&format!("{last}{first}"));
        (from..self.tokens.len()).find(|&i| normalize_quotes(&self.tokens[i].raw) == glued)
    }

    /// Fait correspondre `words` aux mots du rendu à partir de `start`, en
    /// tolérant un mot coupé par un trait d'union en fin de ligne.
    ///
    /// # Returns
    ///
    /// Index qui suit le dernier mot apparié, ou `None`.
    fn match_words(&self, start: usize, words: &[String]) -> Option<usize> {
        let mut index = start;
        for word in words {
            let expected = key(word);
            let token = self.tokens.get(index)?;
            if token.key == expected {
                index += 1;
                continue;
            }
            let next = self.tokens.get(index + 1).filter(|_| token.last_in_line && token.raw.ends_with(HYPHENS))?;
            let stem = token.raw.trim_end_matches(HYPHENS);
            if key(&format!("{stem}{}", next.raw)) != expected && key(&format!("{}{}", token.raw, next.raw)) != expected {
                return None;
            }
            index += 2;
        }
        Some(index)
    }
}

/// Indique si la rupture entre deux lignes consécutives ouvre un paragraphe.
fn is_hard_break(above: &TextLine, below: &TextLine, pitch: f64, right_edge: f64) -> bool {
    if below.starts_par {
        return true;
    }
    if above.page == below.page && below.baseline - above.baseline > pitch + GAP_TOLERANCE {
        return true;
    }
    let is_short = above.right < right_edge - EDGE_TOLERANCE;
    let word_fits = above.right + SPACE_EM * above.size + below.first_word_width <= right_edge + EDGE_TOLERANCE;
    is_short && word_fits
}

/// Clé de comparaison d'un mot (guillemets normalisés, ponctuation de bord retirée).
fn key(word: &str) -> String {
    word_key(&normalize_quotes(word)).to_string()
}

#[cfg(test)]
#[path = "paragraph_check_tests.rs"]
mod tests;
