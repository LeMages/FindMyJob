//! Comparaison mot à mot de deux textes rendus (texte extrait d'un PDF
//! compilé), utilisée pour vérifier qu'un template rendu avec ses données
//! d'exemple reproduit le texte de la référence, et qu'un template rendu
//! avec des données vides ne contient aucun contenu en dur.

/// Nombre maximal de mots par texte comparé : borne la table de la plus
/// longue sous-séquence commune (quadratique) à quelques millions de cases.
const MAX_WORDS: usize = 4000;

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

    /// Mots manquants qui ne sont pas couverts par une liste de mots
    /// autorisés à manquer (chaque mot autorisé ne couvre qu'une occurrence).
    ///
    /// # Arguments
    ///
    /// * `allowed` - Texte dont les mots peuvent légitimement manquer
    ///   (texte retiré volontairement du template), quotes normalisées ou non.
    ///
    /// # Returns
    ///
    /// Mots manquants restants, dans l'ordre.
    pub fn uncovered_missing(&self, allowed: &[String]) -> Vec<String> {
        let mut pool: Vec<String> =
            allowed.iter().flat_map(|t| words(&normalize_quotes(t)).into_iter().map(str::to_string).collect::<Vec<_>>()).collect();
        let mut uncovered = Vec::new();
        for word in &self.missing {
            match pool.iter().position(|w| w == word) {
                Some(index) => {
                    pool.swap_remove(index);
                }
                None => uncovered.push(word.clone()),
            }
        }
        uncovered
    }
}

/// Découpe un texte en mots (séparateurs : espaces de tout type).
pub fn words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

/// Ramène les apostrophes et guillemets typographiques à leur forme droite.
///
/// Typst convertit les `'` et `"` écrits en markup en guillemets
/// typographiques (smartquotes), mais pas ceux d'une chaîne insérée depuis
/// les données : sans cette normalisation, un template fidèle serait
/// signalé en écart sur chaque apostrophe.
fn normalize_quotes(text: &str) -> String {
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
/// lignes, aux espaces et à la forme des apostrophes et guillemets.
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
    let (expected, actual) = (normalize_quotes(expected), normalize_quotes(actual));
    let a = words(&expected);
    let b = words(&actual);
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
            lcs[i * width + j] = if a[i] == b[j] {
                lcs[(i + 1) * width + j + 1] + 1
            } else {
                lcs[(i + 1) * width + j].max(lcs[i * width + j + 1])
            };
        }
    }

    // ------------------------------------------------------------------
    // 2. Parcours de l'alignement : tout mot non apparié est un écart
    // ------------------------------------------------------------------
    let (mut i, mut j) = (0, 0);
    let mut missing = Vec::new();
    let mut extra = Vec::new();
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            i += 1;
            j += 1;
        } else if lcs[(i + 1) * width + j] >= lcs[i * width + j + 1] {
            missing.push(a[i].to_string());
            i += 1;
        } else {
            extra.push(b[j].to_string());
            j += 1;
        }
    }
    missing.extend(a[i..].iter().map(|w| w.to_string()));
    extra.extend(b[j..].iter().map(|w| w.to_string()));

    Ok(TextDiff { expected_words: a.len(), missing, extra })
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
