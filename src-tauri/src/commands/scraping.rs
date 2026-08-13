//! Commande Tauri pour le scraping et l'extraction automatique d'une offre.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::scraping_extract::{extract_offer_fields, ScrapedOfferFields};
use crate::scraping::extract::{extract_text, truncate_chars, MAX_CHARS};
use crate::scraping::fetch::fetch_html;

/// Scrape une offre d'emploi depuis son URL et en extrait les champs du
/// formulaire de candidature (poste, entreprise, lieu, type de contrat,
/// rémunération, description) via l'API Claude.
///
/// # Arguments
///
/// * `url` - URL de la page d'offre à analyser.
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé. Erreur si l'URL est
/// vide, si le site bloque l'accès, ou si l'extraction LLM échoue.
#[tauri::command]
pub async fn scrape_offer(url: String) -> Result<ScrapedOfferFields, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::Validation(
            "L'URL de l'offre est requise.".to_string(),
        ));
    }

    let html = fetch_html(url).await?;
    let text = extract_text(&html)?;
    extract_offer_fields(&text).await
}

/// Extrait les champs du formulaire de candidature depuis un texte d'offre
/// collé manuellement — fallback quand le scraping (`scrape_offer`) échoue
/// (site protégé, timeout...).
///
/// # Arguments
///
/// * `text` - Texte de l'offre collé par l'utilisateur.
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé. Erreur si le texte est
/// vide ou si l'extraction LLM échoue.
#[tauri::command]
pub async fn extract_offer_from_text(text: String) -> Result<ScrapedOfferFields, AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Validation(
            "Le texte de l'offre est vide.".to_string(),
        ));
    }

    let truncated = truncate_chars(text, MAX_CHARS);
    extract_offer_fields(&truncated).await
}
