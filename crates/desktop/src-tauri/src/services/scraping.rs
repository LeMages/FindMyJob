//! Extraction d'une offre d'emploi : depuis son lien (page récupérée par
//! l'application), depuis un HTML capturé par la fenêtre de capture, ou
//! depuis un texte collé.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::offer_extraction::{extract_from_text, run_cascade};
use findmyjob_core::error::AppError;
use findmyjob_core::llm::client::DEFAULT_MODEL;
use findmyjob_core::models::OfferExtraction;
use findmyjob_core::scraping::fetch::fetch_html;
use findmyjob_core::scraping::url_validation::validate_offer_url;

/// Récupère la page à l'URL donnée puis lance la cascade d'extraction.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite (modèle configuré).
/// * `url` - Lien de l'offre.
///
/// # Returns
///
/// Champs extraits, `AppError::Validation` si le lien est vide, ou l'échec
/// typé de récupération (`AppError::Scraping`).
pub async fn scrape_offer(pool: &SqlitePool, url: &str) -> Result<OfferExtraction, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::Validation("L'URL de l'offre est requise.".to_string()));
    }
    let model = db::settings::resolve_llm_model(pool, DEFAULT_MODEL).await?;
    let html = fetch_html(url).await?;
    run_cascade(&html, &model).await
}

/// Lance la cascade d'extraction sur un HTML déjà capturé. L'URL est
/// validée comme pour une récupération, mais n'est pas chargée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite (modèle configuré).
/// * `url` - Lien de la page capturée.
/// * `html` - HTML rendu de la page.
///
/// # Returns
///
/// Champs extraits, ou l'erreur de validation / d'extraction.
pub async fn extract_from_html(pool: &SqlitePool, url: &str, html: &str) -> Result<OfferExtraction, AppError> {
    validate_offer_url(url)?;
    let model = db::settings::resolve_llm_model(pool, DEFAULT_MODEL).await?;
    run_cascade(html, &model).await
}

/// Extrait les champs depuis un texte d'offre collé manuellement.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite (modèle configuré).
/// * `text` - Texte collé.
///
/// # Returns
///
/// Champs extraits, ou `AppError::Validation` si le texte est vide.
pub async fn extract_from_pasted_text(pool: &SqlitePool, text: &str) -> Result<OfferExtraction, AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Validation("Le texte de l'offre est vide.".to_string()));
    }
    let model = db::settings::resolve_llm_model(pool, DEFAULT_MODEL).await?;
    extract_from_text(text, &model).await
}
