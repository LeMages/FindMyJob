//! Contrôles de prose communs à la rédaction du CV et de la lettre :
//! marques et formules interdites (hors texte recopié d'une zone `[FIXE]`),
//! respect des zones fixes, et nettoyage mécanique de dernier recours qui
//! laisse les zones fixes intactes.

// Importation des modules internes
use crate::docgen::fixed_zones::{fixed_zone_violations, transform_outside_fixed, without_fixed_text};
use crate::docgen::lint::{lint_prose, strip_dashes, Violation};
use crate::models::{ProseField, ProseGuidance};

/// Ce que la revue de la prose doit savoir de la référence.
///
/// # Attributes
///
/// * `example_prose` - Prose de la référence : une formule qui y figure
///   est autorisée (jamais une marque).
/// * `guidance` - Consignes de la référence, champs aux chemins du contenu
///   rédigé.
/// * `information` - Informations des données d'exemple, qu'une zone fixe
///   peut mettre à jour (voir `docgen::example_facts`).
pub struct ProseChecks {
    pub example_prose: String,
    pub guidance: ProseGuidance,
    pub information: Vec<String>,
}

impl ProseChecks {
    /// Violations de la prose rédigée : linter hors zones fixes, puis zones
    /// fixes non recopiées.
    ///
    /// # Arguments
    ///
    /// * `fields` - Champs de prose du contenu rédigé.
    ///
    /// # Returns
    ///
    /// Violations relevées.
    pub fn violations(&self, fields: &[ProseField]) -> Vec<Violation> {
        let mut violations = lint_prose(&without_fixed_text(&self.guidance, fields), &self.example_prose);
        violations.extend(fixed_zone_violations(&self.guidance, fields, &self.information));
        violations
    }

    /// Nettoyage mécanique d'un champ (marques interdites), hors texte de
    /// ses zones fixes.
    ///
    /// # Arguments
    ///
    /// * `path` - Chemin du champ.
    /// * `text` - Texte du champ.
    ///
    /// # Returns
    ///
    /// Texte nettoyé.
    pub fn clean(&self, path: &str, text: &str) -> String {
        let fixed: Vec<&str> = self.guidance.fixed_texts(path).collect();
        transform_outside_fixed(text, &fixed, strip_dashes)
    }
}
