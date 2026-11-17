//! Assemblage déterministe des données d'une lettre (`LmData`) : identité
//! du profil (sans repli sur le document de référence), destinataire, date formatée, contenu rédigé et libellés. Aucun
//! appel LLM.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des crates externes
use chrono::NaiveDate;

// Importation des modules internes
use crate::docgen::identity::{build_identity, SenderLocation};
use crate::docgen::date_format::format_long_date;
use crate::models::{LmContent, LmData, LmLabels, ProfileDto, Recipient};

/// Ce que l'assemblage de la lettre reprend de la session, de la référence et du CV.
///
/// # Attributes
///
/// * `labels` - Libellés dans la langue de la lettre.
/// * `title` - Sous-titre professionnel, repris du CV final : les deux
///   documents affichent le même.
/// * `date` - Date de la lettre.
/// * `lang` - Langue de la lettre.
/// * `texts` - Textes en dur de la référence, dans la langue de la lettre.
/// * `location` - Localisation de l'expéditeur (session), sans repli sur la référence.
pub struct LmLayout<'a> {
    pub location: &'a SenderLocation,
    pub labels: &'a LmLabels,
    pub title: &'a str,
    pub date: NaiveDate,
    pub lang: &'a str,
    pub texts: &'a BTreeMap<String, String>,
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
        identity: build_identity(&profile.user, layout.location, layout.title),
        lang: layout.lang.to_string(),
        recipient: recipient.clone(),
        date: format_long_date(layout.date, layout.lang),
        subject: content.subject.clone(),
        paragraphs: content.paragraphs.clone(),
        labels: layout.labels.clone(),
        texts: layout.texts.clone(),
    }
}
