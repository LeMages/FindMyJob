//! Types du balisage d'une référence : consignes générales, zones balisées,
//! textes visibles et erreurs de balisage (avec leur ligne).
//!
//! Le format est décrit dans `docs/reference-markup.md`.

// Importation des modules standard
use std::fmt;

// Importation des modules internes
use crate::models::ZoneMode;

/// Zone balisée telle qu'écrite dans la référence, avant son rattachement à
/// un champ de prose.
///
/// # Attributes
///
/// * `line` - Ligne (1-based) de la balise dans la référence.
/// * `mode` - Rôle de la zone.
/// * `instruction` - Consigne écrite après la balise (chaîne vide si aucune).
/// * `text` - Texte visible couvert par la zone, espaces normalisés.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkupZone {
    pub line: usize,
    pub mode: ZoneMode,
    pub instruction: String,
    pub text: String,
}

/// Texte visible de la référence, à l'échelle d'un littéral ou d'une suite
/// de texte en markup : candidat « texte en dur » s'il n'est couvert par
/// aucune donnée.
///
/// # Attributes
///
/// * `line` - Ligne (1-based) où il commence.
/// * `text` - Texte, espaces normalisés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleText {
    pub line: usize,
    pub text: String,
}

/// Balisage lu dans une référence.
///
/// # Attributes
///
/// * `general` - Consignes générales (bloc `[CONSIGNES]`), chaîne vide si aucune.
/// * `zones` - Zones balisées, dans l'ordre du source.
/// * `visible_texts` - Textes visibles, dans l'ordre du source.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReferenceMarkup {
    pub general: String,
    pub zones: Vec<MarkupZone>,
    pub visible_texts: Vec<VisibleText>,
}

/// Erreur de balisage, à corriger par l'utilisateur dans sa référence.
///
/// # Attributes
///
/// * `line` - Ligne (1-based) concernée.
/// * `message` - Explication en français.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkupError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for MarkupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {} : {}", self.line, self.message)
    }
}

/// Formule une liste d'erreurs de balisage pour l'utilisateur.
///
/// # Arguments
///
/// * `errors` - Erreurs relevées.
///
/// # Returns
///
/// Un message lisible : quoi, où, et quoi faire.
pub fn describe_markup_errors(errors: &[MarkupError]) -> String {
    let lines: Vec<String> = errors.iter().map(|e| format!("- {e}")).collect();
    format!(
        "Le balisage de la référence contient des erreurs. Corrigez-les dans le fichier .typ puis réimportez-le \
         (format décrit dans l'aide « Balisage des consignes ») :\n{}",
        lines.join("\n")
    )
}
