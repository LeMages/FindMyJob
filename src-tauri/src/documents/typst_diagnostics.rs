//! Mise en forme lisible des erreurs de compilation Typst : chaque
//! diagnostic est situé (numéro et texte de la ligne fautive dans la source
//! principale), pour qu'un message d'erreur soit exploitable par un humain
//! ou renvoyé tel quel en feedback à un LLM.

// Importation des crates externes
use typst_as_lib::TypstAsLibError;
use typst_library::diag::SourceDiagnostic;
use typst_syntax::{DiagSpanKind, Source};

/// Longueur maximale citée d'une ligne fautive.
const MAX_LINE_CHARS: usize = 160;

/// Décrit une erreur de compilation Typst, diagnostics situés.
///
/// # Arguments
///
/// * `error` - Erreur renvoyée par `typst_as_lib`.
/// * `main_source` - Source principale compilée, pour situer les diagnostics.
///
/// # Returns
///
/// Une ligne par diagnostic : `ligne N : message (« texte de la ligne »)`.
pub fn describe_compile_error(error: &TypstAsLibError, main_source: &str) -> String {
    match error {
        TypstAsLibError::TypstSource(diagnostics) => diagnostics
            .iter()
            .map(|diagnostic| describe_diagnostic(diagnostic, main_source))
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

/// Décrit un diagnostic, avec sa ligne s'il pointe dans la source principale.
fn describe_diagnostic(diagnostic: &SourceDiagnostic, main_source: &str) -> String {
    let hints = diagnostic.hints.iter().map(|hint| format!(" Indice : {}", hint.v)).collect::<String>();
    match locate(diagnostic, main_source) {
        Some((line_number, line)) => {
            format!("ligne {line_number} : {} (« {line} »).{hints}", diagnostic.message)
        }
        None => format!("{}.{hints}", diagnostic.message),
    }
}

/// Retrouve la ligne (numéro 1-based, texte tronqué) visée par un
/// diagnostic, s'il pointe dans la source principale.
///
/// Les numéros de span ne dépendent que de l'arbre syntaxique : reparser la
/// même source sous le même identifiant de fichier retrouve le même nœud.
/// Un diagnostic qui vise un autre fichier (ex. `data.json`) n'est pas situé.
fn locate(diagnostic: &SourceDiagnostic, main_source: &str) -> Option<(usize, String)> {
    let range = match diagnostic.span.get() {
        DiagSpanKind::Number { id, num, sub_range } => {
            if id.vpath().get_without_slash().ends_with(".json") {
                return None;
            }
            Source::new(id, main_source.to_string()).range(num, sub_range)?
        }
        DiagSpanKind::Range { .. } | DiagSpanKind::Detached => return None,
    };
    let start = range.start.min(main_source.len());
    let line_number = main_source[..start].matches('\n').count() + 1;
    let line = main_source.lines().nth(line_number - 1)?.trim();
    Some((line_number, line.chars().take(MAX_LINE_CHARS).collect()))
}
