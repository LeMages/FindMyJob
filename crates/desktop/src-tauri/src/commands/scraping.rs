//! Commande Tauri pour le scraping et l'extraction automatique d'une offre.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::settings;
use crate::error::AppError;
use crate::llm::client::DEFAULT_MODEL;
use crate::llm::scraping_extract::{extract_offer_fields, ScrapedOfferFields};
use crate::scraping::extract::{extract_text, truncate_chars, MAX_CHARS};
use crate::scraping::fetch::fetch_html;

/// Scrape une offre d'emploi depuis son URL et en extrait les champs du
/// formulaire de candidature (poste, entreprise, lieu, type de contrat,
/// rémunération, description) via l'API Claude.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions, pour le modèle configuré).
/// * `url` - URL de la page d'offre à analyser.
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé. Erreur si l'URL est
/// vide, si le site bloque l'accès, ou si l'extraction LLM échoue.
#[tauri::command]
pub async fn scrape_offer(
    state: tauri::State<'_, DbState>,
    url: String,
) -> Result<ScrapedOfferFields, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::Validation(
            "L'URL de l'offre est requise.".to_string(),
        ));
    }

    let html = fetch_html(url).await?;
    let text = extract_text(&html)?;
    let model = settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    extract_offer_fields(&text, &model).await
}

/// Extrait les champs du formulaire de candidature depuis un texte d'offre
/// collé manuellement — fallback quand le scraping (`scrape_offer`) échoue
/// (site protégé, timeout...).
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions, pour le modèle configuré).
/// * `text` - Texte de l'offre collé par l'utilisateur.
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé. Erreur si le texte est
/// vide ou si l'extraction LLM échoue.
#[tauri::command]
pub async fn extract_offer_from_text(
    state: tauri::State<'_, DbState>,
    text: String,
) -> Result<ScrapedOfferFields, AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Validation(
            "Le texte de l'offre est vide.".to_string(),
        ));
    }

    let truncated = truncate_chars(text, MAX_CHARS);
    let model = settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    extract_offer_fields(&truncated, &model).await
}
