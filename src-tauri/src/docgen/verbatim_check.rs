//! Contrôle de recopie du contexte détaillé dans la lettre : le contexte est
//! une matière de fond, jamais un texte à citer. Il y a recopie quand une
//! suite d'au moins `MIN_COPIED_WORDS` mots consécutifs d'un contexte se
//! retrouve dans un paragraphe, à la casse, à la ponctuation et aux espaces
//! près. Fonctions pures, sans LLM.

// Importation des modules standard
use std::collections::HashSet;

// Importation des modules internes
use crate::docgen::lint::Violation;

/// Nombre minimal de mots consécutifs repris pour parler de recopie.
pub const MIN_COPIED_WORDS: usize = 8;

/// Mots d'un texte sous forme comparable : minuscules, tout caractère qui
/// n'est ni une lettre ni un chiffre sert de séparateur.
fn normalized_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// Suites de `MIN_COPIED_WORDS` mots consécutifs d'une liste de mots.
fn windows(words: &[String]) -> impl Iterator<Item = String> + '_ {
    words.windows(MIN_COPIED_WORDS).map(|w| w.join(" "))
}

/// Premier passage d'un texte repris mot pour mot d'une des sources.
///
/// # Arguments
///
/// * `text` - Texte rédigé (un paragraphe).
/// * `sources` - Textes à ne pas recopier (contextes détaillés).
///
/// # Returns
///
/// Le passage repris (`MIN_COPIED_WORDS` mots normalisés), ou `None`.
pub fn copied_passage(text: &str, sources: &[String]) -> Option<String> {
    let source_windows: HashSet<String> =
        sources.iter().flat_map(|s| windows(&normalized_words(s)).collect::<Vec<_>>()).collect();
    if source_windows.is_empty() {
        return None;
    }
    let words = normalized_words(text);
    let mut found = windows(&words).filter(|w| source_windows.contains(w));
    found.next()
}

/// Violations de recopie des paragraphes d'une lettre : une par paragraphe
/// qui reprend un passage d'un contexte, stricte (elle justifie une relance).
///
/// # Arguments
///
/// * `paragraphs` - Paragraphes de la lettre, dans l'ordre.
/// * `sources` - Contextes détaillés transmis à la rédaction.
///
/// # Returns
///
/// Violations relevées, au chemin `paragraphs[i]`.
pub fn verbatim_violations(paragraphs: &[String], sources: &[String]) -> Vec<Violation> {
    paragraphs
        .iter()
        .enumerate()
        .filter_map(|(i, paragraph)| {
            copied_passage(paragraph, sources).map(|passage| {
                Violation::new(
                    format!("paragraphs[{i}]"),
                    format!(
                        "reprend mot pour mot un passage du contexte détaillé (« {passage} ») : reformule ce fait avec tes propres mots, dans le style du modèle"
                    ),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contexte de test (17 mots).
    fn contexte() -> Vec<String> {
        vec!["J'ai migré quarante rapports Excel vers dbt en trois mois, avec l'équipe finance de Lyon.".to_string()]
    }

    #[test]
    fn sept_mots_consecutifs_ne_sont_pas_une_recopie() {
        let text = "Pendant mon alternance, j'ai migré quarante rapports Excel vers Power BI.";
        assert_eq!(copied_passage(text, &contexte()), None);
    }

    #[test]
    fn huit_mots_consecutifs_sont_une_recopie() {
        let text = "Pendant mon alternance, j'ai migré quarante rapports Excel vers dbt en un trimestre.";
        assert_eq!(copied_passage(text, &contexte()).as_deref(), Some("j ai migré quarante rapports excel vers dbt"));
    }

    #[test]
    fn casse_ponctuation_et_espaces_ignores() {
        let text = "J’AI   MIGRÉ quarante rapports — Excel, vers DBT en trois mois !";
        assert!(copied_passage(text, &contexte()).is_some());
    }

    #[test]
    fn sources_vides_ou_courtes_ne_relevent_rien() {
        let text = "J'ai migré quarante rapports Excel vers dbt en trois mois.";
        assert_eq!(copied_passage(text, &[]), None);
        assert_eq!(copied_passage(text, &["J'ai migré quarante rapports.".to_string()]), None);
    }

    #[test]
    fn violation_stricte_au_chemin_du_paragraphe() {
        let paragraphs = vec![
            "Je souhaite rejoindre votre équipe.".to_string(),
            "J'ai migré quarante rapports Excel vers dbt en trois mois.".to_string(),
        ];
        let violations = verbatim_violations(&paragraphs, &contexte());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].field, "paragraphs[1]");
        assert!(!violations[0].advisory);
        assert!(violations[0].message.contains("contexte détaillé"));
    }
}
