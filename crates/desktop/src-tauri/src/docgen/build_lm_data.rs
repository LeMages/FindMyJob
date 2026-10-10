//! Assemblage déterministe des données d'une lettre (`LmData`) : identité
//! du profil, destinataire, date formatée, contenu rédigé et libellés.
//! Aucun appel LLM.

// Importation des crates externes
use chrono::NaiveDate;

// Importation des modules internes
use crate::docgen::identity::build_identity;
use findmyjob_core::docgen::date_format::format_long_date;
use findmyjob_core::models::{LmContent, LmData, LmLabels, ProfileDto, Recipient};

/// Assemble les données de la lettre.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `recipient` - Destinataire (champs disponibles en base uniquement).
/// * `content` - Contenu rédigé (objet, paragraphes).
/// * `labels` - Libellés dans la langue de la lettre.
/// * `date` - Date de la lettre.
/// * `lang` - Langue de la lettre.
///
/// # Returns
///
/// `LmData` prêt à être sérialisé en `data.json`.
pub fn build_lm_data(
    profile: &ProfileDto,
    recipient: &Recipient,
    content: &LmContent,
    labels: &LmLabels,
    date: NaiveDate,
    lang: &str,
) -> LmData {
    LmData {
        identity: build_identity(&profile.user),
        lang: lang.to_string(),
        recipient: recipient.clone(),
        date: format_long_date(date, lang),
        subject: content.subject.clone(),
        paragraphs: content.paragraphs.clone(),
        labels: labels.clone(),
    }
}
