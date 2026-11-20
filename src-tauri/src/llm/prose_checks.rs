//! Contrôles de prose communs à la rédaction du CV et de la lettre :
//! marques et formules interdites, et nettoyage mécanique de dernier
//! recours. Les parties fixes d'une référence sont du texte du template :
//! elles ne passent jamais par la rédaction, donc jamais par ces contrôles.

// Importation des modules internes
use crate::docgen::lint::{lint_prose, strip_dashes, Violation};
use crate::models::ProseField;

/// Ce que la revue de la prose doit savoir de la référence.
///
/// # Attributes
///
/// * `example_prose` - Prose de la référence : une formule qui y figure
///   est autorisée (jamais une marque).
pub struct ProseChecks {
    pub example_prose: String,
}

impl ProseChecks {
    /// Violations de la prose rédigée (linter).
    ///
    /// # Arguments
    ///
    /// * `fields` - Champs de prose du contenu rédigé.
    ///
    /// # Returns
    ///
    /// Violations relevées.
    pub fn violations(&self, fields: &[ProseField]) -> Vec<Violation> {
        lint_prose(fields, &self.example_prose)
    }

    /// Nettoyage mécanique d'un champ (marques interdites).
    ///
    /// # Arguments
    ///
    /// * `text` - Texte du champ.
    ///
    /// # Returns
    ///
    /// Texte nettoyé.
    pub fn clean(&self, text: &str) -> String {
        strip_dashes(text)
    }
}
