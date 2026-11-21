//! Mots d'un texte rendu (texte extrait d'un PDF compilé) : découpage,
//! clé de comparaison et séparateurs orphelins.
//!
//! Les mots sont comparés sans leur ponctuation de bord (`word_key`) : une
//! virgule collée à un mot (« Lyon, » contre « Lyon ») tient à la mise en
//! page, pas au contenu.

/// Ponctuation retirée en tête et en fin de mot avant comparaison
/// (guillemets typographiques déjà ramenés à leur forme droite).
const EDGE_PUNCTUATION: [char; 10] = [',', '.', ';', ':', '!', '?', '"', '\'', '«', '»'];

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
