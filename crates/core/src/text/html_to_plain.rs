//! Point d'entrée de la conversion d'un champ d'offre (HTML ou Markdown)
//! en texte brut : rendu HTML ou décodage des entités, nettoyage Markdown,
//! suppression des emojis puis normalisation des espaces.

// Importation des modules internes
use super::emoji_strip::strip_emojis;
use super::html_walk::{decode_entities, render_html};
use super::markdown_cleanup::clean_markdown;
use super::whitespace_normalize::normalize_whitespace;

/// Convertit un texte d'offre HTML ou Markdown en texte brut lisible.
///
/// Le HTML est détecté par la présence d'une balise ; sinon l'entrée est
/// traitée comme du texte ou du Markdown, dont les sauts de ligne sont
/// significatifs. Un HTML échappé (`&lt;p&gt;…`, fréquent dans le JSON-LD des
/// pages d'offre) est décodé puis rendu comme du HTML. Le nettoyage Markdown
/// s'applique dans tous les cas, car certains ATS glissent des `**` dans
/// leur HTML.
///
/// # Arguments
///
/// * `input` - Texte brut du champ (HTML, Markdown, ou valeur vide/`"null"`).
///
/// # Returns
///
/// Le texte brut normalisé, ou `None` si l'entrée est vide, ne contient que
/// des espaces, ou vaut `"null"` (y compris `"<p>null</p>"`).
pub fn html_to_plain(input: &str) -> Option<String> {
    let rendered = if looks_like_html(input) {
        render_html(input)
    } else {
        let decoded = decode_entities(input);
        if looks_like_html(&decoded) {
            render_html(&decoded)
        } else {
            decoded
        }
    };

    let cleaned = clean_markdown(&rendered);
    let without_emojis = strip_emojis(&cleaned);
    let normalized = normalize_whitespace(&without_emojis);

    // Certaines sources sérialisent l'absence de valeur en chaîne "null" :
    // on la traite comme un champ vide, après rendu pour couvrir aussi la
    // variante "<p>null</p>".
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("null") {
        None
    } else {
        Some(normalized)
    }
}

/// Indique si le texte contient au moins une balise HTML (`<` suivi d'une
/// lettre, de `/` ou de `!`). Un `<` isolé (« < 3 ans ») ne compte pas.
fn looks_like_html(input: &str) -> bool {
    input
        .as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'<' && (pair[1].is_ascii_alphabetic() || pair[1] == b'/' || pair[1] == b'!'))
}
