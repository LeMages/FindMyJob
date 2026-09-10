//! Extraction du texte exploitable d'une page HTML d'offre d'emploi.

// Importation des crates externes
use scraper::{ElementRef, Html, Node, Selector};

// Importation des modules internes
use crate::error::AppError;

/// Sélecteurs CSS essayés dans l'ordre pour localiser le contenu de l'offre.
/// Le premier qui matche est utilisé comme racine d'extraction.
const CONTENT_SELECTORS: &[&str] = &[
    "article",
    "main",
    "section",
    "[class*=\"job\"]",
    "[class*=\"offer\"]",
    "[class*=\"description\"]",
    "[class*=\"posting\"]",
];

/// Balises dont le sous-arbre est ignoré lors de l'extraction (navigation,
/// scripts, styles — jamais du contenu d'offre exploitable).
const SKIP_TAGS: &[&str] = &["script", "style", "nav", "header", "footer", "aside"];

/// Troncature du texte extrait, en nombre de caractères — garde de contexte
/// avant l'appel LLM. Réutilisée telle quelle par
/// `commands::scraping::extract_offer_from_text` (crate `findmyjob`, desktop)
/// pour le texte collé manuellement (même garde que pour le texte scrapé).
pub const MAX_CHARS: usize = 12_000;

/// Extrait le texte lisible d'une page HTML d'offre d'emploi.
///
/// Cherche d'abord un conteneur sémantique probable (`<article>`, `<main>`,
/// classes contenant "job"/"offer"/"description"/"posting"...), avec repli
/// sur `<body>` entier. Les balises `<script>`, `<style>`, `<nav>`,
/// `<header>`, `<footer>` et `<aside>` sont exclues du texte extrait.
///
/// # Arguments
///
/// * `html` - HTML brut de la page.
///
/// # Returns
///
/// Le texte nettoyé (espaces collapsés, tronqué à [`MAX_CHARS`] caractères).
pub fn extract_text(html: &str) -> Result<String, AppError> {
    let document = Html::parse_document(html);

    let root = find_content_root(&document);

    let mut raw_text = String::new();
    if let Some(root) = root {
        collect_text(root, &mut raw_text);
    }

    let cleaned = raw_text.trim();
    if cleaned.is_empty() {
        return Err(AppError::Scraping(
            "Aucun contenu exploitable trouvé sur cette page. Essaie la saisie manuelle."
                .to_string(),
        ));
    }

    Ok(truncate_chars(cleaned, MAX_CHARS))
}

/// Localise l'élément racine à partir duquel extraire le texte : premier
/// sélecteur sémantique qui matche, sinon `<body>`, sinon le document entier.
fn find_content_root(document: &Html) -> Option<ElementRef<'_>> {
    for raw_selector in CONTENT_SELECTORS {
        // Les sélecteurs sont des constantes CSS valides connues à la
        // compilation — l'échec de parsing n'est pas un cas à gérer ici.
        let selector = Selector::parse(raw_selector).expect("sélecteur CSS constant valide");
        if let Some(element) = document.select(&selector).next() {
            return Some(element);
        }
    }

    let body_selector = Selector::parse("body").expect("sélecteur CSS constant valide");
    document
        .select(&body_selector)
        .next()
        .or_else(|| Some(document.root_element()))
}

/// Parcourt récursivement les descendants d'un élément et accumule le texte
/// des noeuds `Text`, une entrée par ligne, en ignorant les sous-arbres des
/// balises listées dans [`SKIP_TAGS`].
fn collect_text(element: ElementRef, out: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if !collapsed.is_empty() {
                    out.push_str(&collapsed);
                    out.push('\n');
                }
            }
            Node::Element(el) => {
                if SKIP_TAGS.contains(&el.name()) {
                    continue;
                }
                if let Some(child_element) = ElementRef::wrap(child) {
                    collect_text(child_element, out);
                }
            }
            _ => {}
        }
    }
}

/// Tronque une chaîne à `max_chars` caractères, en respectant les frontières
/// UTF-8 (le contenu est en français : ne pas découper au milieu d'un octet).
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect()
}
