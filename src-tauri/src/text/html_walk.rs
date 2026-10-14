//! Rendu d'un fragment HTML en texte structuré par lignes : blocs séparés
//! par une ligne vide, listes en puces `- ` indentées, entités décodées.

// Importation des crates externes
use scraper::{ElementRef, Html, Node};

/// Balises rendues comme des blocs séparés par une ligne vide.
const BLOCK_TAGS: &[&str] = &[
    "p", "div", "section", "article", "header", "footer", "blockquote", "h1", "h2", "h3", "h4", "h5", "h6",
];

/// Balises dont le sous-arbre est ignoré (jamais du contenu d'offre).
const SKIP_TAGS: &[&str] = &["script", "style", "head", "template"];

/// Profondeur d'imbrication maximale parcourue : garde-fou contre un HTML
/// pathologique, largement au-delà de ce que produisent les ATS.
const MAX_DEPTH: usize = 128;

/// Indentation ajoutée par niveau de liste imbriquée.
const LIST_INDENT: &str = "  ";

/// Accumulateur du rendu, avec le niveau de liste courant.
///
/// Attributes:
///     out: Texte produit jusqu'ici.
///     list_depth: Nombre de `<ul>`/`<ol>` ouverts au point courant.
///     extra_skip: Balises ignorées en plus de [`SKIP_TAGS`] (ex. `nav` pour
///         une page complète).
struct HtmlRenderer<'a> {
    out: String,
    list_depth: usize,
    extra_skip: &'a [&'a str],
}

impl HtmlRenderer<'_> {
    /// Parcourt les enfants d'un élément et rend chacun d'eux.
    fn walk(&mut self, element: ElementRef, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        for child in element.children() {
            match child.value() {
                Node::Text(text) => self.push_text(text),
                Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        self.visit_element(child_element, depth + 1);
                    }
                }
                _ => {}
            }
        }
    }

    /// Rend un élément selon sa balise (bloc, liste, saut de ligne, inline).
    fn visit_element(&mut self, element: ElementRef, depth: usize) {
        let name = element.value().name();
        match name {
            "br" => self.out.push('\n'),
            "li" => self.visit_list_item(element, depth),
            "ul" | "ol" => self.visit_list(element, depth),
            _ if SKIP_TAGS.contains(&name) || self.extra_skip.contains(&name) => {}
            _ if BLOCK_TAGS.contains(&name) => self.visit_block(element, depth),
            _ => self.walk(element, depth),
        }
    }

    /// Rend un bloc : ligne vide avant/après hors liste ; dans une puce, un
    /// simple retour à la ligne (`<li><p>…</p></li>` ne doit pas aérer la liste).
    fn visit_block(&mut self, element: ElementRef, depth: usize) {
        if self.list_depth > 0 {
            if self.current_line_has_content() {
                self.out.push('\n');
            }
            self.walk(element, depth);
            return;
        }
        self.out.push_str("\n\n");
        self.walk(element, depth);
        self.out.push_str("\n\n");
    }

    /// Rend une liste : isolée par des lignes vides au premier niveau,
    /// collée à la puce parente quand elle est imbriquée.
    fn visit_list(&mut self, element: ElementRef, depth: usize) {
        let top_level = self.list_depth == 0;
        if top_level {
            self.out.push_str("\n\n");
        }
        self.list_depth += 1;
        self.walk(element, depth);
        self.list_depth -= 1;
        if top_level {
            self.out.push_str("\n\n");
        }
    }

    /// Rend une puce `- `, indentée selon le niveau d'imbrication.
    fn visit_list_item(&mut self, element: ElementRef, depth: usize) {
        self.out.push('\n');
        self.out.push_str(&LIST_INDENT.repeat(self.list_depth.saturating_sub(1)));
        self.out.push_str("- ");
        self.walk(element, depth);
    }

    /// Ajoute un nœud texte en réduisant ses espaces (y compris insécables)
    /// à un seul, tout en gardant un espace aux bords : il sépare le texte
    /// des balises inline voisines (`<strong>école</strong>&#xa0;ou`).
    fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if text.starts_with(char::is_whitespace) {
            self.out.push(' ');
        }
        self.out.push_str(&text.split_whitespace().collect::<Vec<_>>().join(" "));
        if text.ends_with(char::is_whitespace) {
            self.out.push(' ');
        }
    }

    /// Indique si la ligne en cours contient du texte au-delà de son
    /// éventuel préfixe de puce.
    fn current_line_has_content(&self) -> bool {
        let line = self.out.rsplit('\n').next().unwrap_or("");
        let trimmed = line.trim();
        !trimmed.is_empty() && trimmed != "-"
    }
}

/// Rend un fragment HTML en texte structuré par lignes.
///
/// # Arguments
///
/// * `html` - Fragment HTML (champ d'offre).
///
/// # Returns
///
/// Le texte rendu, pas encore normalisé (lignes vides multiples, espaces
/// en bord de ligne possibles).
pub(crate) fn render_html(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    render_element(fragment.root_element(), &[])
}

/// Rend le contenu d'un élément d'un document déjà parsé en texte structuré
/// par lignes (titres en blocs, listes en puces).
///
/// # Arguments
///
/// * `element` - Racine du rendu (ex. `<main>` d'une page d'offre).
/// * `extra_skip` - Balises dont le sous-arbre est ignoré en plus des
///   scripts et styles (ex. `nav`, `footer`).
///
/// # Returns
///
/// Le texte rendu, pas encore normalisé.
pub(crate) fn render_element(element: ElementRef, extra_skip: &[&str]) -> String {
    let mut renderer = HtmlRenderer {
        out: String::new(),
        list_depth: 0,
        extra_skip,
    };
    renderer.walk(element, 0);
    renderer.out
}

/// Décode les entités HTML d'un texte sans balise, en conservant ses sauts
/// de ligne (le Markdown en dépend).
///
/// # Arguments
///
/// * `text` - Texte brut pouvant contenir des entités (`&amp;`, `&#xa0;`...).
///
/// # Returns
///
/// Le texte avec les entités décodées.
pub(crate) fn decode_entities(text: &str) -> String {
    Html::parse_fragment(text).root_element().text().collect()
}
