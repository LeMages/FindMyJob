//! Extraction déterministe du balisage d'une référence : consignes
//! générales (`// [CONSIGNES]`), zones `// [FIXE]`, `// [VARIABLE]`,
//! `// [ADAPTABLE]` et textes visibles.
//!
//! Règles (détaillées dans `docs/reference-markup.md`) :
//! - une balise ouvre un commentaire de ligne `//`, en majuscules ou non ;
//!   le texte qui la suit, et les lignes `//` qui suivent sans ligne vide,
//!   forment sa consigne ;
//! - une zone couvre le texte visible qui suit sa balise, jusqu'à la balise
//!   suivante ou la fin du paragraphe (ligne vide) ;
//! - `[CONSIGNES]` ouvre le bloc des consignes générales : les lignes `//`
//!   qui le suivent, jusqu'à la première ligne qui n'est pas un commentaire ;
//! - une balise ailleurs qu'en début de commentaire, dans un commentaire
//!   `/* */`, un second bloc `[CONSIGNES]` ou une zone sans texte sont des
//!   erreurs, signalées avec leur ligne.

// Importation des modules internes
use crate::documents::markup::model::{MarkupError, MarkupZone, ReferenceMarkup, VisibleText};
use crate::documents::markup::source_events::{source_events, SourceEvent};
use crate::models::ZoneMode;

/// Balise lue en tête d'un commentaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    General,
    Zone(ZoneMode),
}

/// Balises reconnues, telles qu'écrites (sans les crochets).
const TAGS: [(&str, Tag); 4] = [
    ("CONSIGNES", Tag::General),
    ("FIXE", Tag::Zone(ZoneMode::Fixe)),
    ("VARIABLE", Tag::Zone(ZoneMode::Variable)),
    ("ADAPTABLE", Tag::Zone(ZoneMode::Adaptable)),
];

/// Ce que les commentaires suivants prolongent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Continuation {
    General,
    Instruction,
}

/// État de la lecture.
#[derive(Default)]
struct State {
    markup: ReferenceMarkup,
    errors: Vec<MarkupError>,
    general_line: Option<usize>,
    general_lines: Vec<String>,
    open_zone: Option<MarkupZone>,
    /// Conteneur du premier texte de la zone ouverte.
    zone_scope: Option<Vec<usize>>,
    instruction_lines: Vec<String>,
    continuation: Option<Continuation>,
    unit: Option<VisibleText>,
}

/// Lit le balisage d'une référence.
///
/// # Arguments
///
/// * `source` - Source Typst de la référence.
///
/// # Returns
///
/// `ReferenceMarkup`, ou toutes les erreurs de balisage relevées.
pub fn parse_markup(source: &str) -> Result<ReferenceMarkup, Vec<MarkupError>> {
    let events = source_events(source).map_err(|message| vec![MarkupError { line: 1, message }])?;
    let mut state = State::default();
    for event in events {
        state.handle(event);
    }
    state.finish()
}

impl State {
    /// Traite un événement de la source.
    fn handle(&mut self, event: SourceEvent) {
        match event {
            SourceEvent::LineComment { line, body } => self.line_comment(line, &body),
            SourceEvent::BlockComment { line, body } => {
                self.close_unit();
                if find_tag_anywhere(&body).is_some() {
                    self.error(line, "une balise doit être écrite dans un commentaire de ligne `//`, pas `/* */`.");
                }
            }
            SourceEvent::Visible { line, text, unit_start, scope } => self.visible(line, &text, unit_start, scope),
            SourceEvent::Break => {
                self.close_unit();
                self.continuation = None;
                if self.open_zone.as_ref().is_some_and(|zone| !zone.text.trim().is_empty()) {
                    self.close_zone();
                }
            }
            SourceEvent::Code => {
                self.close_unit();
                self.continuation = None;
            }
            SourceEvent::Blank => {}
        }
    }

    /// Commentaire de ligne : balise, suite d'une consigne, ou commentaire libre.
    fn line_comment(&mut self, line: usize, body: &str) {
        self.close_unit();
        match leading_tag(body) {
            Some((Tag::General, rest)) => self.open_general(line, rest),
            Some((Tag::Zone(mode), rest)) => self.open_zone(line, mode, rest),
            None => {
                if let Some(tag) = find_tag_anywhere(body) {
                    self.error(line, &format!("la balise [{tag}] doit ouvrir le commentaire (`// [{tag}] consigne…`)."));
                }
                match self.continuation {
                    Some(Continuation::General) => self.general_lines.push(body.to_string()),
                    Some(Continuation::Instruction) => self.instruction_lines.push(body.to_string()),
                    None => {}
                }
            }
        }
    }

    /// Ouvre le bloc des consignes générales.
    fn open_general(&mut self, line: usize, rest: &str) {
        if let Some(first) = self.general_line {
            self.error(line, &format!("un seul bloc [CONSIGNES] par référence (déjà ouvert ligne {first})."));
            return;
        }
        self.general_line = Some(line);
        if !rest.is_empty() {
            self.general_lines.push(rest.to_string());
        }
        self.continuation = Some(Continuation::General);
    }

    /// Ouvre une zone, en fermant la précédente.
    fn open_zone(&mut self, line: usize, mode: ZoneMode, rest: &str) {
        self.close_zone();
        self.instruction_lines = if rest.is_empty() { Vec::new() } else { vec![rest.to_string()] };
        self.open_zone = Some(MarkupZone { line, mode, instruction: String::new(), text: String::new() });
        self.continuation = Some(Continuation::Instruction);
    }

    /// Texte visible : alimente la zone ouverte et l'unité de texte courante.
    /// Une zone ne couvre que le conteneur de son premier texte (le
    /// paragraphe, le bloc `[...]` ou le littéral qui suit la balise).
    fn visible(&mut self, line: usize, text: &str, unit_start: bool, scope: Vec<usize>) {
        // Les espaces entre deux commentaires ne coupent pas une consigne.
        if text.trim().is_empty() {
            if self.zone_scope.as_ref().is_some_and(|zone_scope| scope.starts_with(zone_scope)) {
                self.append_visible(line, text, unit_start);
            } else {
                self.append_unit(line, text, unit_start);
            }
            return;
        }
        self.continuation = None;
        match &self.zone_scope {
            Some(zone_scope) if !scope.starts_with(zone_scope) => self.close_zone(),
            None if self.open_zone.is_some() => self.zone_scope = Some(scope),
            _ => {}
        }
        if let Some(zone) = &mut self.open_zone {
            if zone.instruction.is_empty() && zone.text.is_empty() {
                zone.instruction = join_lines(&self.instruction_lines);
            }
        }
        self.append_visible(line, text, unit_start);
    }

    /// Ajoute un texte visible à la zone ouverte et à l'unité courante.
    fn append_visible(&mut self, line: usize, text: &str, unit_start: bool) {
        if let Some(zone) = &mut self.open_zone {
            zone.text.push_str(text);
        }
        self.append_unit(line, text, unit_start);
    }

    /// Ajoute un texte visible à l'unité courante.
    fn append_unit(&mut self, line: usize, text: &str, unit_start: bool) {
        if unit_start {
            self.close_unit();
        }
        match &mut self.unit {
            Some(unit) => unit.text.push_str(text),
            None if !text.trim().is_empty() => self.unit = Some(VisibleText { line, text: text.to_string() }),
            None => {}
        }
    }

    /// Termine l'unité de texte visible courante.
    fn close_unit(&mut self) {
        if let Some(mut unit) = self.unit.take() {
            unit.text = normalize_spaces(&unit.text);
            if !unit.text.is_empty() {
                self.markup.visible_texts.push(unit);
            }
        }
    }

    /// Termine la zone ouverte ; une zone sans texte est une erreur.
    fn close_zone(&mut self) {
        self.zone_scope = None;
        let Some(mut zone) = self.open_zone.take() else {
            return;
        };
        if zone.instruction.is_empty() {
            zone.instruction = join_lines(&self.instruction_lines);
        }
        zone.text = normalize_spaces(&zone.text);
        if zone.text.is_empty() {
            let tag = zone.mode.tag();
            self.error(zone.line, &format!("la balise {tag} n'est suivie d'aucun texte avant la ligne vide suivante."));
            return;
        }
        self.markup.zones.push(zone);
    }

    /// Enregistre une erreur.
    fn error(&mut self, line: usize, message: &str) {
        self.errors.push(MarkupError { line, message: message.to_string() });
    }

    /// Termine la lecture.
    fn finish(mut self) -> Result<ReferenceMarkup, Vec<MarkupError>> {
        self.close_unit();
        self.close_zone();
        self.markup.general = join_lines(&self.general_lines);
        if self.errors.is_empty() {
            Ok(self.markup)
        } else {
            Err(self.errors)
        }
    }
}

/// Balise qui ouvre un commentaire, et le texte qui la suit.
fn leading_tag(body: &str) -> Option<(Tag, &str)> {
    let rest = body.strip_prefix('[')?;
    let (name, after) = rest.split_once(']')?;
    let tag = TAGS.iter().find(|(written, _)| written.eq_ignore_ascii_case(name.trim()))?.1;
    Some((tag, after.trim()))
}

/// Première balise reconnue n'importe où dans un commentaire, en majuscules.
fn find_tag_anywhere(body: &str) -> Option<&'static str> {
    let upper = body.to_uppercase();
    TAGS.iter().map(|(written, _)| *written).find(|written| upper.contains(&format!("[{written}]")))
}

/// Joint des lignes de consigne en un texte, lignes vides retirées.
fn join_lines(lines: &[String]) -> String {
    lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

/// Remplace toute suite d'espaces par un seul espace, sans espace au bord.
fn normalize_spaces(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
