//! Détection des polices introuvables dans les avertissements Typst.
//!
//! Typst ne fait pas échouer une compilation dont la famille demandée est
//! absente : il émet l'avertissement `unknown font family: <famille>` et se
//! replie sur sa police par défaut. Ce module est le seul endroit qui
//! reconnaît cet avertissement, pour que la compilation puisse le
//! transformer en erreur bloquante.

// Importation des crates externes
use typst_library::diag::SourceDiagnostic;

/// Préfixe du message émis par Typst pour une famille absente
/// (`typst-library`, `text::check_font_list`).
const UNKNOWN_FONT_PREFIX: &str = "unknown font family: ";

/// Relève les familles de polices introuvables signalées par Typst.
///
/// Typst écrit la famille en minuscules : sa graphie d'origine est reprise
/// d'une chaîne de la source qui lui est égale à la casse près, s'il y en a
/// une.
///
/// # Arguments
///
/// * `warnings` - Avertissements renvoyés par le compilateur.
/// * `main_source` - Source principale compilée.
///
/// # Returns
///
/// Familles introuvables, sans doublon, dans l'ordre des avertissements.
pub fn missing_font_families(warnings: &[SourceDiagnostic], main_source: &str) -> Vec<String> {
    let mut families: Vec<String> = Vec::new();
    for warning in warnings {
        let Some(family) = warning.message.strip_prefix(UNKNOWN_FONT_PREFIX) else {
            continue;
        };
        let family = original_spelling(family.trim(), main_source);
        if !families.contains(&family) {
            families.push(family);
        }
    }
    families
}

/// Message d'erreur montré à l'utilisateur pour des polices introuvables.
///
/// # Arguments
///
/// * `families` - Familles introuvables (au moins une).
///
/// # Returns
///
/// Phrase française qui nomme chaque famille et dit où l'importer.
pub fn missing_fonts_message(families: &[String]) -> String {
    debug_assert!(!families.is_empty(), "aucune police introuvable à signaler");
    let quoted = families.iter().map(|family| format!("\"{family}\"")).collect::<Vec<_>>().join(", ");
    if families.len() > 1 {
        format!("Polices introuvables : {quoted}. Importez-les dans Paramètres > Polices.")
    } else {
        format!("Police introuvable : {quoted}. Importez-la dans Paramètres > Polices.")
    }
}

/// Retrouve la graphie d'une famille dans les chaînes de la source.
///
/// Découpage naïf sur `"` (les guillemets échappés sont rares dans un nom
/// de police) : en cas d'échec, la graphie de Typst est conservée.
fn original_spelling(family: &str, main_source: &str) -> String {
    main_source
        .split('"')
        .skip(1)
        .step_by(2)
        .find(|literal| literal.to_lowercase() == family.to_lowercase())
        .unwrap_or(family)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst_syntax::Span;

    fn warning(message: &str) -> SourceDiagnostic {
        SourceDiagnostic::warning(Span::detached(), message)
    }

    #[test]
    fn releve_la_famille_avec_sa_graphie_d_origine() {
        let warnings = [warning("unknown font family: arimo")];
        let families = missing_font_families(&warnings, "#set text(font: \"Arimo\")");
        assert_eq!(families, vec!["Arimo".to_string()]);
    }

    #[test]
    fn garde_la_graphie_de_typst_sans_chaine_correspondante() {
        let warnings = [warning("unknown font family: arimo")];
        assert_eq!(missing_font_families(&warnings, "#set text(font: data.font)"), vec!["arimo".to_string()]);
    }

    #[test]
    fn ignore_les_autres_avertissements_et_dedoublonne() {
        let warnings = [
            warning("`show par: set block(spacing: ..)` has no effect anymore"),
            warning("unknown font family: arimo"),
            warning("unknown font family: arimo"),
            warning("unknown font family: inter"),
        ];
        let source = "#set text(font: (\"Arimo\", \"Inter\"))";
        assert_eq!(missing_font_families(&warnings, source), vec!["Arimo".to_string(), "Inter".to_string()]);
    }

    #[test]
    fn aucun_avertissement_de_police() {
        assert!(missing_font_families(&[warning("autre chose")], "").is_empty());
    }

    #[test]
    fn message_singulier_et_pluriel() {
        assert_eq!(
            missing_fonts_message(&["Arimo".to_string()]),
            "Police introuvable : \"Arimo\". Importez-la dans Paramètres > Polices."
        );
        assert_eq!(
            missing_fonts_message(&["Arimo".to_string(), "Inter".to_string()]),
            "Polices introuvables : \"Arimo\", \"Inter\". Importez-les dans Paramètres > Polices."
        );
    }
}
