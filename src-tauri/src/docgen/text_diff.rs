//! Comparaison mot à mot de deux textes rendus (texte extrait d'un PDF
//! compilé), utilisée pour vérifier qu'un template rendu avec ses données
//! d'exemple reproduit le texte de la référence, que les zones des
//! consignes redonnent la prose, et qu'une zone fixe est recopiée.
//!
//! Les mots sont comparés sans leur ponctuation de bord (`word_key`) : une
//! virgule collée à un mot (« Lyon, » contre « Lyon ») tient à la mise en
//! page, pas au contenu. Les écarts citent toujours les mots d'origine.

/// Nombre maximal de mots par texte comparé : borne la table de la plus
/// longue sous-séquence commune (quadratique) à quelques millions de cases.
const MAX_WORDS: usize = 4000;
/// Ponctuation retirée en tête et en fin de mot avant comparaison
/// (guillemets typographiques déjà ramenés à leur forme droite).
const EDGE_PUNCTUATION: [char; 10] = [',', '.', ';', ':', '!', '?', '"', '\'', '«', '»'];

/// Écart mot à mot entre un texte attendu et un texte obtenu.
///
/// # Attributes
///
/// * `expected_words` - Nombre de mots du texte attendu.
/// * `missing` - Mots attendus absents du texte obtenu, dans l'ordre.
/// * `extra` - Mots du texte obtenu absents du texte attendu, dans l'ordre.
#[derive(Debug, Clone, PartialEq)]
pub struct TextDiff {
    pub expected_words: usize,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}

impl TextDiff {
    /// Nombre total de mots en écart (manquants + en trop).
    pub fn distance(&self) -> usize {
        self.missing.len() + self.extra.len()
    }
}

/// Découpe un texte en mots (séparateurs : espaces de tout type).
pub fn words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

/// Clé de comparaison d'un mot : le mot sans sa ponctuation de bord
/// (`EDGE_PUNCTUATION`). Un mot fait uniquement de ponctuation (« : »,
/// « ! ») garde sa forme : il reste comparé tel quel.
///
/// # Arguments
///
/// * `word` - Mot, guillemets déjà normalisés.
///
/// # Returns
///
/// Clé de comparaison, empruntée au mot.
pub fn word_key(word: &str) -> &str {
    let trimmed = word.trim_matches(|c: char| EDGE_PUNCTUATION.contains(&c));
    if trimmed.is_empty() {
        word
    } else {
        trimmed
    }
}

/// Ramène les apostrophes et guillemets typographiques à leur forme droite.
///
/// Typst convertit les `'` et `"` écrits en markup en guillemets
/// typographiques (smartquotes), mais pas ceux d'une chaîne insérée depuis
/// les données : sans cette normalisation, un template fidèle serait
/// signalé en écart sur chaque apostrophe.
pub fn normalize_quotes(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            other => other,
        })
        .collect()
}

/// Calcule l'écart mot à mot entre deux textes, par plus longue
/// sous-séquence commune : un mot n'est compté en écart que s'il ne peut
/// pas être aligné, ce qui rend la comparaison insensible aux coupures de
/// lignes, aux espaces, à la forme des apostrophes et guillemets et à la
/// ponctuation de bord des mots (`word_key`).
///
/// # Arguments
///
/// * `expected` - Texte de référence.
/// * `actual` - Texte obtenu.
///
/// # Returns
///
/// `TextDiff`, ou une erreur si un texte dépasse `MAX_WORDS` mots.
pub fn diff_words(expected: &str, actual: &str) -> Result<TextDiff, String> {
    let hunks = diff_hunks(expected, actual)?;
    Ok(TextDiff {
        expected_words: words(&normalize_quotes(expected)).len(),
        missing: hunks.iter().flat_map(|h| h.missing.iter().cloned()).collect(),
        extra: hunks.iter().flat_map(|h| h.extra.iter().cloned()).collect(),
    })
}

/// Écart localisé : mots consécutifs non alignés, à une position du texte
/// attendu.
///
/// # Attributes
///
/// * `expected_start` - Indice (0-based) du premier mot attendu concerné
///   (ou du mot attendu qui suit des mots en trop).
/// * `missing` - Mots attendus absents, dans l'ordre.
/// * `extra` - Mots obtenus en trop, dans l'ordre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub expected_start: usize,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}

/// Calcule les écarts localisés entre deux textes (même alignement que
/// [`diff_words`]).
///
/// # Arguments
///
/// * `expected` - Texte de référence.
/// * `actual` - Texte obtenu.
///
/// # Returns
///
/// Les écarts dans l'ordre du texte attendu, ou une erreur si un texte
/// dépasse `MAX_WORDS` mots.
pub fn diff_hunks(expected: &str, actual: &str) -> Result<Vec<DiffHunk>, String> {
    let (expected, actual) = (normalize_quotes(expected), normalize_quotes(actual));
    let a = words(&expected);
    let b = words(&actual);
    let key_a: Vec<&str> = a.iter().map(|w| word_key(w)).collect();
    let key_b: Vec<&str> = b.iter().map(|w| word_key(w)).collect();
    if a.len() > MAX_WORDS || b.len() > MAX_WORDS {
        return Err(format!(
            "Texte trop long pour la comparaison ({} et {} mots, maximum {MAX_WORDS}).",
            a.len(),
            b.len()
        ));
    }

    // ------------------------------------------------------------------
    // 1. Table des longueurs de plus longue sous-séquence commune, calculée
    //    depuis la fin pour pouvoir reconstruire l'alignement vers l'avant
    // ------------------------------------------------------------------
    let width = b.len() + 1;
    let mut lcs = vec![0u16; (a.len() + 1) * width];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i * width + j] = if key_a[i] == key_b[j] {
                lcs[(i + 1) * width + j + 1] + 1
            } else {
                lcs[(i + 1) * width + j].max(lcs[i * width + j + 1])
            };
        }
    }

    // ------------------------------------------------------------------
    // 2. Parcours de l'alignement : les mots non appariés consécutifs
    //    forment un écart
    // ------------------------------------------------------------------
    let (mut i, mut j) = (0, 0);
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut open: Option<DiffHunk> = None;
    while i < a.len() || j < b.len() {
        let aligned = i < a.len() && j < b.len() && key_a[i] == key_b[j];
        if aligned {
            hunks.extend(open.take());
            i += 1;
            j += 1;
            continue;
        }
        let hunk = open.get_or_insert_with(|| DiffHunk { expected_start: i, missing: Vec::new(), extra: Vec::new() });
        let take_missing = j >= b.len() || (i < a.len() && lcs[(i + 1) * width + j] >= lcs[i * width + j + 1]);
        if take_missing {
            hunk.missing.push(a[i].to_string());
            i += 1;
        } else {
            hunk.extra.push(b[j].to_string());
            j += 1;
        }
    }
    hunks.extend(open);
    Ok(hunks)
}

/// Repère les séparateurs orphelins d'un texte rendu : deux séparateurs
/// graphiques consécutifs (« | | », « • • », « : | »), signe qu'une valeur
/// vide a été affichée entre eux au lieu de faire disparaître sa ligne.
///
/// # Arguments
///
/// * `text` - Texte rendu.
///
/// # Returns
///
/// Paires de séparateurs consécutifs, citées telles quelles.
pub fn orphan_separators(text: &str) -> Vec<String> {
    let is_separator = |w: &str| !w.is_empty() && w.chars().all(|c| SEPARATOR_CHARS.contains(&c));
    words(text)
        .windows(2)
        .filter(|pair| is_separator(pair[0]) && is_separator(pair[1]))
        .map(|pair| format!("{} {}", pair[0], pair[1]))
        .collect()
}

/// Caractères considérés comme séparateurs graphiques.
const SEPARATOR_CHARS: [char; 9] = ['|', '•', '·', '–', '—', '-', ':', ',', '/'];

/// Liste les mots qui contiennent au moins une lettre ou un chiffre.
///
/// Un template rendu avec des données entièrement vides ne doit produire
/// que des séparateurs graphiques (puces, barres, ponctuation) : tout mot
/// renvoyé ici est un contenu écrit en dur dans le template.
///
/// # Arguments
///
/// * `text` - Texte rendu.
///
/// # Returns
///
/// Mots alphanumériques, dans l'ordre d'apparition.
pub fn alphanumeric_words(text: &str) -> Vec<String> {
    words(text)
        .into_iter()
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .map(str::to_string)
        .collect()
}
