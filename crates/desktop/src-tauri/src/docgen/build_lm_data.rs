//! Assemblage déterministe des données d'une lettre (`LmData`) : identité
//! du profil (ou du document de référence quand le profil est vide sur un
//! champ), destinataire, date formatée, contenu rédigé et libellés. Aucun
//! appel LLM.

// Importation des crates externes
use chrono::NaiveDate;

// Importation des modules internes
use crate::docgen::identity::build_identity;
use findmyjob_core::docgen::date_format::format_long_date;
use findmyjob_core::models::{LmContent, LmData, LmLabels, ProfileDto, Recipient};

/// Ce que l'assemblage de la lettre reprend de la référence et du CV.
///
/// # Attributes
///
/// * `document` - Données d'exemple de la référence, reprises quand le
///   profil est vide sur une information.
/// * `labels` - Libellés dans la langue de la lettre.
/// * `title` - Sous-titre professionnel, repris du CV final : les deux
///   documents affichent le même.
/// * `date` - Date de la lettre.
/// * `lang` - Langue de la lettre.
pub struct LmLayout<'a> {
    pub document: &'a LmData,
    pub labels: &'a LmLabels,
    pub title: &'a str,
    pub date: NaiveDate,
    pub lang: &'a str,
}

/// Assemble les données de la lettre.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `recipient` - Destinataire (champs disponibles en base uniquement).
/// * `content` - Contenu rédigé (objet, paragraphes).
/// * `layout` - Document de référence, libellés, sous-titre, date et langue.
///
/// # Returns
///
/// `LmData` prêt à être sérialisé en `data.json`.
pub fn build_lm_data(profile: &ProfileDto, recipient: &Recipient, content: &LmContent, layout: &LmLayout<'_>) -> LmData {
    LmData {
        identity: build_identity(&profile.user, layout.title, &layout.document.identity),
        lang: layout.lang.to_string(),
        recipient: recipient.clone(),
        date: format_long_date(layout.date, layout.lang),
        subject: content.subject.clone(),
        paragraphs: content.paragraphs.clone(),
        labels: layout.labels.clone(),
    }
}
