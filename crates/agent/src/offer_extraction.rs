//! Cascade d'extraction des champs d'une offre à partir de son HTML brut,
//! partagée par la récupération par lien (`POST /api/scraping/url`) et la
//! fenêtre de capture du desktop (`POST /api/scraping/html`).

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::llm::scraping_extract::{extract_offer_fields, ScrapedOfferFields};
use findmyjob_core::scraping::extract::extract_text;

/// Extrait les champs du formulaire de candidature depuis le HTML d'une page
/// d'offre.
///
/// # Arguments
///
/// * `html` - HTML brut de la page (récupéré par l'agent ou capturé).
/// * `model` - Modèle Claude à utiliser pour l'extraction.
///
/// # Returns
///
/// Les champs extraits, ou l'erreur typée de l'étape qui a échoué.
pub async fn run_cascade(html: &str, model: &str) -> Result<ScrapedOfferFields, AppError> {
    let text = extract_text(html)?;
    extract_offer_fields(&text, model).await
}
