//! Destinataire de la lettre, construit uniquement depuis les champs
//! disponibles en base : la candidature et ses contacts.

// Importation des modules internes
use crate::docgen::identity::text;
use findmyjob_core::models::{CandidatureDetail, ContactView, Recipient};

/// Construit le destinataire de la lettre.
///
/// L'entreprise et le lieu viennent de la candidature. Le contact est le
/// premier contact nommé de la candidature, s'il y en a un.
///
/// # Arguments
///
/// * `candidature` - Candidature en cours de génération.
/// * `contacts` - Contacts de la candidature.
///
/// # Returns
///
/// `Recipient`, champs absents en chaîne vide.
pub fn build_recipient(candidature: &CandidatureDetail, contacts: &[ContactView]) -> Recipient {
    let contact = contacts.iter().find(|c| !text(c.nom.as_ref()).is_empty());

    Recipient {
        company: text(candidature.entreprise.as_ref()),
        location: text(candidature.localisation.as_ref()),
        contact_name: contact.map(|c| text(c.nom.as_ref())).unwrap_or_default(),
        contact_role: contact.map(|c| text(c.role.as_ref())).unwrap_or_default(),
    }
}
