//! Lecture des indices de titre d'une page (`<title>`, `og:title`), transmis
//! au LLM pour l'aider à identifier le poste et l'entreprise.

// Importation des crates externes
use scraper::{Html, Selector};

/// Indices de titre lus dans le `<head>` d'une page.
///
/// # Attributes
///
/// * `title` - Contenu de `<title>`, espaces réduits.
/// * `og_title` - Contenu de `<meta property="og:title">`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageHints {
    pub title: Option<String>,
    pub og_title: Option<String>,
}

impl PageHints {
    /// Indique si aucun indice n'a été trouvé.
    ///
    /// # Returns
    ///
    /// `true` si ni `<title>` ni `og:title` ne sont renseignés.
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.og_title.is_none()
    }
}

/// Lit `<title>` et `og:title` dans un document parsé.
///
/// # Arguments
///
/// * `document` - Page HTML parsée.
///
/// # Returns
///
/// Les indices trouvés, chacun `None` s'il est absent ou vide.
pub fn read_page_hints(document: &Html) -> PageHints {
    let title_selector = Selector::parse("title").expect("sélecteur CSS constant valide");
    let og_selector = Selector::parse("meta[property=\"og:title\"]").expect("sélecteur CSS constant valide");

    let title = document
        .select(&title_selector)
        .next()
        .and_then(|el| non_empty(&el.text().collect::<String>()));
    let og_title = document
        .select(&og_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .and_then(non_empty);
    PageHints { title, og_title }
}

/// Réduit les espaces et renvoie `None` pour une chaîne vide.
fn non_empty(raw: &str) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    (!collapsed.is_empty()).then_some(collapsed)
}
