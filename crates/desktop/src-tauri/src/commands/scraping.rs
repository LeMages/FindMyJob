//! Commande Tauri pour le scraping et l'extraction automatique d'une offre.

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use findmyjob_core::llm::scraping_extract::ScrapedOfferFields;

/// Scrape une offre d'emploi depuis son URL et en extrait les champs du
/// formulaire de candidature (poste, entreprise, lieu, type de contrat,
/// rémunération, description) via l'agent.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `url` - URL de la page d'offre à analyser.
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé. Erreur si l'URL est
/// vide, si le site bloque l'accès, ou si l'extraction LLM échoue.
#[tauri::command]
pub async fn scrape_offer(
    api: tauri::State<'_, ApiClient>,
    url: String,
) -> Result<ScrapedOfferFields, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::Validation("L'URL de l'offre est requise.".to_string()));
    }

    api.post("/api/scraping/url", &json!({ "url": url })).await
}

/// Extrait les champs du formulaire de candidature depuis un texte d'offre
/// collé manuellement — fallback quand le scraping (`scrape_offer`) échoue
/// (site protégé, timeout...).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `text` - Texte de l'offre collé par l'utilisateur.
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé. Erreur si le texte est
/// vide ou si l'extraction LLM échoue.
#[tauri::command]
pub async fn extract_offer_from_text(
    api: tauri::State<'_, ApiClient>,
    text: String,
) -> Result<ScrapedOfferFields, AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Validation("Le texte de l'offre est vide.".to_string()));
    }

    api.post("/api/scraping/text", &json!({ "text": text })).await
}
