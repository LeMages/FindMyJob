//! Destinataire de la lettre, construit uniquement depuis la candidature :
//! la lettre ne mentionne jamais de contact.

// Importation des modules internes
use crate::docgen::identity::text;
use findmyjob_core::models::{CandidatureDetail, Recipient};

/// Construit le destinataire de la lettre : entreprise et lieu de la
/// candidature.
///
/// # Arguments
///
/// * `candidature` - Candidature en cours de génération.
///
/// # Returns
///
/// `Recipient`, champs absents en chaîne vide (`contact_*` toujours vides).
pub fn build_recipient(candidature: &CandidatureDetail) -> Recipient {
    Recipient {
        company: text(candidature.entreprise.as_ref()),
        location: text(candidature.localisation.as_ref()),
        ..Default::default()
    }
}
