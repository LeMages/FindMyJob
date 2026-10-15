//! Extraction d'une offre d'emploi : depuis son lien (page récupérée par
//! l'application), depuis un HTML capturé par la fenêtre de capture, ou
//! depuis un texte collé.

// Importation des modules internes
use crate::offer_extraction::{extract_from_text, run_cascade};
use crate::services::claude_target::ClaudeTargetSource;
use crate::error::AppError;
use crate::models::OfferExtraction;
use crate::scraping::fetch::fetch_html;
use crate::scraping::url_validation::validate_offer_url;

/// Récupère la page à l'URL donnée puis lance la cascade d'extraction.
///
/// # Arguments
///
/// * `claude` - Source de la clé API et du modèle configuré.
/// * `url` - Lien de l'offre.
///
/// # Returns
///
/// Champs extraits, `AppError::Validation` si le lien est vide, ou l'échec
/// typé de récupération (`AppError::Scraping`).
pub async fn scrape_offer(claude: ClaudeTargetSource<'_>, url: &str) -> Result<OfferExtraction, AppError> {
    scrape_offer_with(claude, url, fetch_html).await
}

/// [`scrape_offer`] avec la fonction de récupération en paramètre.
/// L'application passe toujours [`fetch_html`] ; les tests y passent la
/// variante qui vise leur serveur local.
///
/// # Arguments
///
/// * `claude` - Source de la clé API et du modèle configuré.
/// * `url` - Lien de l'offre.
/// * `fetch` - Récupère le HTML d'une URL déjà non vide.
///
/// # Returns
///
/// Champs extraits, `AppError::Validation` si le lien est vide, ou l'échec
/// typé de récupération (`AppError::Scraping`).
pub async fn scrape_offer_with<F>(claude: ClaudeTargetSource<'_>, url: &str, fetch: F) -> Result<OfferExtraction, AppError>
where
    F: AsyncFnOnce(&str) -> Result<String, AppError>,
{
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::Validation("L'URL de l'offre est requise.".to_string()));
    }
    let html = fetch(url).await?;
    run_cascade(&html, claude).await
}

/// Lance la cascade d'extraction sur un HTML déjà capturé. L'URL est
/// validée comme pour une récupération, mais n'est pas chargée.
///
/// # Arguments
///
/// * `claude` - Source de la clé API et du modèle configuré.
/// * `url` - Lien de la page capturée.
/// * `html` - HTML rendu de la page.
///
/// # Returns
///
/// Champs extraits, ou l'erreur de validation / d'extraction.
pub async fn extract_from_html(claude: ClaudeTargetSource<'_>, url: &str, html: &str) -> Result<OfferExtraction, AppError> {
    validate_offer_url(url)?;
    run_cascade(html, claude).await
}

/// Extrait les champs depuis un texte d'offre collé manuellement.
///
/// # Arguments
///
/// * `claude` - Source de la clé API et du modèle configuré.
/// * `text` - Texte collé.
///
/// # Returns
///
/// Champs extraits, ou `AppError::Validation` si le texte est vide.
pub async fn extract_from_pasted_text(claude: ClaudeTargetSource<'_>, text: &str) -> Result<OfferExtraction, AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Validation("Le texte de l'offre est vide.".to_string()));
    }
    let target = claude.resolve().await?;
    extract_from_text(text, &target).await
}
