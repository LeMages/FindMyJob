//! Destinataire de la lettre, construit uniquement depuis la candidature :
//! la lettre ne mentionne jamais de contact.

// Importation des modules internes
use crate::docgen::identity::text;
use crate::models::{CandidatureDetail, Recipient};

/// Découpe la localisation d'une candidature en lignes d'adresse : la
/// dernière partie (séparée par des virgules) est la ligne du lieu, les
/// précédentes (rue, service…) les lignes supplémentaires, dans l'ordre.
///
/// # Arguments
///
/// * `localisation` - Localisation telle qu'enregistrée (chaîne libre).
///
/// # Returns
///
/// `(extra_lines, location)` ; parties vides ignorées, `location` vide si
/// la localisation l'est.
pub fn split_location(localisation: &str) -> (Vec<String>, String) {
    let mut parts: Vec<String> =
        localisation.split(',').map(str::trim).filter(|part| !part.is_empty()).map(str::to_string).collect();
    let location = parts.pop().unwrap_or_default();
    (parts, location)
}

/// Construit le destinataire de la lettre : entreprise de la candidature,
/// localisation découpée en lignes d'adresse (`split_location`). La
/// localisation enregistrée n'est pas modifiée.
///
/// # Arguments
///
/// * `candidature` - Candidature en cours de génération.
///
/// # Returns
///
/// `Recipient`, champs absents en chaîne vide (`contact_*` toujours vides).
pub fn build_recipient(candidature: &CandidatureDetail) -> Recipient {
    let (extra_lines, location) = split_location(&text(candidature.localisation.as_ref()));
    Recipient { company: text(candidature.entreprise.as_ref()), location, extra_lines, ..Default::default() }
}
