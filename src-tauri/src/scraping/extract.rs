//! Extraction du texte exploitable d'une page HTML d'offre d'emploi, pour
//! l'étage LLM de la cascade (quand la page n'a pas de JSON-LD exploitable).

// Importation des modules standard
use std::collections::HashSet;

// Importation des crates externes
use scraper::{ElementRef, Html, Selector};

// Importation des modules internes
use crate::error::AppError;
use crate::scraping::content_check::is_insufficient_page_text;
use crate::scraping::failure::ScrapingFailure;
use crate::scraping::page_hints::{read_page_hints, PageHints};
use crate::text::html_walk::render_element;
use crate::text::whitespace_normalize::normalize_whitespace;

/// Sélecteurs CSS des conteneurs susceptibles de porter le contenu de
/// l'offre. Tous les éléments qui matchent sont candidats ; le plus riche en
/// texte l'emporte.
const CONTENT_SELECTORS: &[&str] = &[
    "article",
    "main",
    "section",
    "[class*=\"job\"]",
    "[class*=\"offer\"]",
    "[class*=\"description\"]",
    "[class*=\"posting\"]",
];

/// Balises dont le sous-arbre est ignoré dans une page complète (navigation,
/// en-têtes, formulaires — jamais du contenu d'offre exploitable). Les
/// scripts et styles sont déjà ignorés par le rendu.
const PAGE_SKIP_TAGS: &[&str] = &[
    "nav", "header", "footer", "aside", "noscript", "form", "button", "svg", "iframe", "select",
];

/// Troncature du texte transmis au LLM, en nombre de caractères — garde de
/// contexte appliquée après l'isolement du contenu de l'offre. Réutilisée
/// par la route d'extraction depuis un texte collé.
pub const MAX_CHARS: usize = 12_000;

/// Texte exploitable d'une page d'offre et indices de titre.
///
/// # Attributes
///
/// * `text` - Texte de l'offre, titres et listes conservés, tronqué à [`MAX_CHARS`].
/// * `hints` - Titre de la page (`<title>`, `og:title`), transmis au LLM.
#[derive(Debug, Clone)]
pub struct PageText {
    pub text: String,
    pub hints: PageHints,
}

/// Extrait le texte lisible d'une page HTML d'offre d'emploi.
///
/// Parmi les conteneurs candidats ([`CONTENT_SELECTORS`]), retient celui
/// dont le texte est le plus long (repli sur `<body>`), le rend en
/// conservant titres et listes, puis tronque. La navigation, les en-têtes,
/// pieds de page et formulaires sont exclus.
///
/// # Arguments
///
/// * `html` - HTML brut de la page.
///
/// # Returns
///
/// Le texte et les indices de titre, ou `AppError::Scraping(InsufficientContent)`
/// si la page n'a pas assez de texte (coquille JavaScript, page de
/// challenge, bandeau cookies seul).
pub fn extract_page_text(html: &str) -> Result<PageText, AppError> {
    let document = Html::parse_document(html);
    let hints = read_page_hints(&document);

    let text = find_content_root(&document)
        .map(|root| normalize_whitespace(&render_element(root, PAGE_SKIP_TAGS)))
        .unwrap_or_default();

    if is_insufficient_page_text(&text) {
        return Err(AppError::Scraping(ScrapingFailure::InsufficientContent));
    }
    Ok(PageText {
        text: truncate_chars(&text, MAX_CHARS),
        hints,
    })
}

/// Localise la racine d'extraction : le candidat au texte le plus long,
/// sinon `<body>`, sinon le document entier.
fn find_content_root(document: &Html) -> Option<ElementRef<'_>> {
    let candidates = outermost_candidates(document);
    let best = candidates
        .into_iter()
        .map(|element| (visible_text_len(element), element))
        .filter(|(len, _)| *len > 0)
        .max_by_key(|(len, _)| *len)
        .map(|(_, element)| element);
    if best.is_some() {
        return best;
    }

    let body_selector = Selector::parse("body").expect("sélecteur CSS constant valide");
    document
        .select(&body_selector)
        .next()
        .or_else(|| Some(document.root_element()))
}

/// Collecte les candidats qui ne sont contenus dans aucun autre candidat.
///
/// Le texte d'un ancêtre englobe celui de ses descendants : seuls les
/// candidats les plus externes peuvent être « le plus long », ce qui évite
/// de mesurer plusieurs fois les mêmes sous-arbres. Les candidats situés
/// dans une zone ignorée (ex. `<nav class="job-menu">`) sont écartés.
fn outermost_candidates(document: &Html) -> Vec<ElementRef<'_>> {
    let mut matched: Vec<ElementRef> = Vec::new();
    for raw_selector in CONTENT_SELECTORS {
        // Les sélecteurs sont des constantes CSS valides connues à la
        // compilation — l'échec de parsing n'est pas un cas à gérer ici.
        let selector = Selector::parse(raw_selector).expect("sélecteur CSS constant valide");
        matched.extend(document.select(&selector));
    }
    let ids: HashSet<_> = matched.iter().map(|element| element.id()).collect();

    let mut seen = HashSet::new();
    matched
        .into_iter()
        .filter(|element| seen.insert(element.id()))
        .filter(|element| !PAGE_SKIP_TAGS.contains(&element.value().name()))
        .filter(|element| {
            !element.ancestors().any(|ancestor| {
                ids.contains(&ancestor.id())
                    || ElementRef::wrap(ancestor).is_some_and(|el| PAGE_SKIP_TAGS.contains(&el.value().name()))
            })
        })
        .collect()
}

/// Longueur (en caractères non blancs) du texte visible d'un élément,
/// zones ignorées exclues.
fn visible_text_len(element: ElementRef) -> usize {
    render_element(element, PAGE_SKIP_TAGS)
        .chars()
        .filter(|c| !c.is_whitespace())
        .count()
}

/// Tronque une chaîne à `max_chars` caractères, en respectant les frontières
/// UTF-8 (le contenu est en français : ne pas découper au milieu d'un octet).
///
/// # Arguments
///
/// * `s` - Texte à tronquer.
/// * `max_chars` - Nombre maximal de caractères conservés.
///
/// # Returns
///
/// Le texte, tronqué si nécessaire.
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect()
}
