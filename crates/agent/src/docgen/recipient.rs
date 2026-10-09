//! Destinataire de la lettre, construit uniquement depuis les champs
//! disponibles en base : la candidature, son offre source et ses contacts.

// Importation des modules internes
use crate::docgen::identity::text;
use findmyjob_core::models::{CandidatureDetail, ContactView, OfferView, Recipient};

/// Construit le destinataire de la lettre.
///
/// L'entreprise et le lieu viennent de la candidature, sinon de l'offre
/// dont elle est issue. Le contact est le premier contact nommé de la
/// candidature, s'il y en a un.
///
/// # Arguments
///
/// * `candidature` - Candidature en cours de génération.
/// * `offer` - Offre source, si la candidature vient du Marché.
/// * `contacts` - Contacts de la candidature.
///
/// # Returns
///
/// `Recipient`, champs absents en chaîne vide.
pub fn build_recipient(candidature: &CandidatureDetail, offer: Option<&OfferView>, contacts: &[ContactView]) -> Recipient {
    let or_offer = |own: String, from_offer: Option<&String>| if own.is_empty() { text(from_offer) } else { own };
    let contact = contacts.iter().find(|c| !text(c.nom.as_ref()).is_empty());

    Recipient {
        company: or_offer(text(candidature.entreprise.as_ref()), offer.map(|o| &o.company)),
        location: or_offer(text(candidature.localisation.as_ref()), offer.map(|o| &o.location)),
        contact_name: contact.map(|c| text(c.nom.as_ref())).unwrap_or_default(),
        contact_role: contact.map(|c| text(c.role.as_ref())).unwrap_or_default(),
    }
}
