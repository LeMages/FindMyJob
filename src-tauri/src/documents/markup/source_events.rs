//! Lecture d'une source Typst en une suite d'événements ordonnés :
//! commentaires, textes visibles, coupures de paragraphe et code.
//!
//! Le texte visible est le texte du markup (avec ses espaces, échappements,
//! raccourcis et guillemets) et les chaînes passées en argument positionnel
//! d'une fonction d'affichage. Ne sont pas visibles : les arguments nommés
//! (`font: "Arimo"`), les règles `set` / `show`, les imports et les
//! arguments des fonctions qui ne rendent pas leur texte (`rgb`, `json`,
//! `image`, `display`…).

// Importation des crates externes
use typst_syntax::ast::{AstNode, Escape, Shorthand, SmartQuote, Str};
use typst_syntax::{SyntaxKind, SyntaxNode};

/// Fonctions dont les chaînes en argument ne sont jamais rendues telles
/// quelles (couleurs, fichiers, formats de date, compteurs…).
const NON_VISUAL_CALLS: [&str; 17] = [
    "rgb", "luma", "cmyk", "oklab", "oklch", "json", "read", "image", "yaml", "toml", "csv", "xml", "display",
    "state", "counter", "label", "datetime",
];

/// Profondeur maximale de l'arbre parcouru : bien au-delà de toute
/// référence réelle, elle borne la récursion sur un fichier pathologique.
pub const MAX_DEPTH: usize = 256;

/// Événement de la source, dans l'ordre du fichier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceEvent {
    /// Commentaire de ligne : texte après `//`, ligne (1-based).
    LineComment { line: usize, body: String },
    /// Commentaire de bloc : texte entre `/*` et `*/`, ligne de début.
    BlockComment { line: usize, body: String },
    /// Morceau de texte visible. `unit_start` est vrai quand il ne prolonge
    /// pas le texte visible précédent (nouveau littéral, ou texte du markup
    /// après du code). `scope` identifie son conteneur : chemin des blocs de
    /// markup englobants, plus le littéral lui-même pour une chaîne. Un
    /// texte est dans le conteneur d'un autre si son `scope` le prolonge.
    Visible { line: usize, text: String, unit_start: bool, scope: Vec<usize> },
    /// Coupure de paragraphe (ligne vide).
    Break,
    /// Élément de code ou de structure sans texte visible.
    Code,
    /// Espace sans texte visible ni coupure (entre deux commentaires…).
    Blank,
}

/// Contexte hérité pendant le parcours.
#[derive(Clone, Copy)]
struct Context {
    /// Le nœud parent est du markup (le texte y est visible).
    in_markup: bool,
    /// Les chaînes rencontrées ici seraient rendues.
    strings_visible: bool,
}

/// Lit une source en événements.
///
/// # Arguments
///
/// * `source` - Source Typst.
///
/// # Returns
///
/// Les événements dans l'ordre du fichier, ou un message si l'arbre
/// dépasse `MAX_DEPTH`.
pub fn source_events(source: &str) -> Result<Vec<SourceEvent>, String> {
    let root = typst_syntax::parse(source);
    let mut walker = Walker {
        source,
        line_starts: line_starts(source),
        offset: 0,
        events: Vec::new(),
        markup_path: Vec::new(),
        next_id: 0,
    };
    walker.walk(&root, Context { in_markup: true, strings_visible: true }, 0)?;
    Ok(walker.events)
}

/// Parcours de l'arbre, avec la position courante dans la source.
struct Walker<'s> {
    source: &'s str,
    line_starts: Vec<usize>,
    offset: usize,
    events: Vec<SourceEvent>,
    /// Identifiants des nœuds de markup englobant la position courante.
    markup_path: Vec<usize>,
    /// Prochain identifiant de conteneur.
    next_id: usize,
}

impl Walker<'_> {
    /// Parcourt un nœud et ses enfants.
    fn walk(&mut self, node: &SyntaxNode, context: Context, depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err(format!("Référence trop imbriquée (plus de {MAX_DEPTH} niveaux)."));
        }
        if node.children().len() == 0 {
            self.leaf(node, context);
            self.offset += node.len();
            return Ok(());
        }
        let child_context = self.child_context(node, context);
        let is_markup = node.kind() == SyntaxKind::Markup;
        if is_markup {
            let id = self.fresh_id();
            self.markup_path.push(id);
        }
        for child in node.children() {
            self.walk(child, child_context, depth + 1)?;
        }
        if is_markup {
            self.markup_path.pop();
        }
        Ok(())
    }

    /// Contexte des enfants d'un nœud interne.
    fn child_context(&self, node: &SyntaxNode, context: Context) -> Context {
        let in_markup = node.kind() == SyntaxKind::Markup;
        let strings_visible = context.strings_visible
            && match node.kind() {
                SyntaxKind::Named
                | SyntaxKind::SetRule
                | SyntaxKind::ShowRule
                | SyntaxKind::ModuleImport
                | SyntaxKind::ModuleInclude => false,
                SyntaxKind::FuncCall => !is_non_visual_call(node),
                // Le contenu d'un bloc [...] est toujours affiché, même
                // passé à une fonction dont les chaînes ne le sont pas.
                _ => true,
            };
        // Un bloc de contenu redevient visible sous une règle `set` / `show`
        // ou un argument nommé (`body: [...]`).
        let strings_visible = strings_visible || node.kind() == SyntaxKind::ContentBlock;
        Context { in_markup, strings_visible }
    }

    /// Traduit une feuille en événement.
    fn leaf(&mut self, node: &SyntaxNode, context: Context) {
        let line = self.line_of(self.offset);
        let text = node.leaf_text();
        let event = match node.kind() {
            SyntaxKind::LineComment => {
                SourceEvent::LineComment { line, body: text.trim_start_matches("//").trim().to_string() }
            }
            SyntaxKind::BlockComment => SourceEvent::BlockComment {
                line,
                body: text.trim_start_matches("/*").trim_end_matches("*/").trim().to_string(),
            },
            SyntaxKind::Parbreak => SourceEvent::Break,
            SyntaxKind::Space if context.in_markup => self.visible(line, " ".to_string(), false),
            SyntaxKind::Space if text.matches('\n').count() >= 2 => SourceEvent::Break,
            SyntaxKind::Space => SourceEvent::Blank,
            SyntaxKind::Text if context.in_markup => self.visible(line, text.to_string(), false),
            SyntaxKind::Linebreak if context.in_markup => self.visible(line, " ".to_string(), false),
            SyntaxKind::Escape => Escape::from_untyped(node)
                .map_or(SourceEvent::Code, |e| self.visible(line, e.get().to_string(), false)),
            SyntaxKind::Shorthand if context.in_markup => Shorthand::from_untyped(node)
                .map_or(SourceEvent::Code, |s| self.visible(line, s.get().to_string(), false)),
            SyntaxKind::SmartQuote => SmartQuote::from_untyped(node)
                .map_or(SourceEvent::Code, |q| self.visible(line, quote_of(q).to_string(), false)),
            SyntaxKind::Str if context.strings_visible => {
                Str::from_untyped(node).map_or(SourceEvent::Code, |s| self.visible(line, s.get().to_string(), true))
            }
            _ => SourceEvent::Code,
        };
        self.events.push(event);
    }

    /// Événement de texte visible ; il prolonge le texte précédent sauf
    /// s'il est un littéral (`own_unit`) ou suit autre chose qu'un texte
    /// visible. Un littéral forme à lui seul son conteneur.
    fn visible(&mut self, line: usize, text: String, own_unit: bool) -> SourceEvent {
        let continues = matches!(self.events.last(), Some(SourceEvent::Visible { .. }));
        let mut scope = self.markup_path.clone();
        if own_unit {
            scope.push(self.fresh_id());
        }
        SourceEvent::Visible { line, text, unit_start: own_unit || !continues, scope }
    }

    /// Nouvel identifiant de conteneur.
    fn fresh_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    /// Ligne (1-based) d'un décalage en octets.
    fn line_of(&self, offset: usize) -> usize {
        debug_assert!(offset <= self.source.len());
        self.line_starts.partition_point(|&start| start <= offset)
    }
}

/// Vrai si l'appel est une fonction dont les chaînes ne sont pas rendues.
fn is_non_visual_call(call: &SyntaxNode) -> bool {
    let Some(callee) = call.children().next() else {
        return false;
    };
    let name = last_ident(callee);
    NON_VISUAL_CALLS.contains(&name.as_str())
}

/// Dernier identifiant d'une expression appelée (`rgb`, `x.display`…).
fn last_ident(node: &SyntaxNode) -> String {
    if node.kind() == SyntaxKind::Ident {
        return node.leaf_text().to_string();
    }
    node.children()
        .rfind(|child| child.kind() == SyntaxKind::Ident)
        .map(|ident| ident.leaf_text().to_string())
        .unwrap_or_default()
}

/// Caractère rendu par un guillemet intelligent.
fn quote_of(quote: SmartQuote<'_>) -> char {
    if quote.double() {
        '"'
    } else {
        '\''
    }
}

/// Décalages de début de chaque ligne.
fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1)).collect()
}
