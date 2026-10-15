//! Résultat de l'extraction d'une offre renvoyé par les commandes
//! d'import (lien, capture, texte collé) : champs du formulaire et étage de
//! la cascade utilisé.

// Importation des modules internes
use crate::llm::scraping_extract::ScrapedOfferFields;

/// Étage de la cascade d'extraction qui a produit les champs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionStage {
    /// Données structurées `JobPosting` de la page, sans LLM.
    JsonLd,
    /// Texte de la page (ou texte collé) analysé par le LLM.
    Llm,
}

/// Champs extraits d'une offre et étage qui les a produits.
///
/// # Attributes
///
/// * `fields` - Champs du formulaire, nettoyés.
/// * `stage` - Étage de la cascade utilisé.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OfferExtraction {
    pub fields: ScrapedOfferFields,
    pub stage: ExtractionStage,
}
