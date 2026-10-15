//! Commandes Tauri d'extraction d'une offre : depuis son lien ou depuis un
//! texte collé (la capture de page est dans `commands::capture`).

// Importation des modules internes
use crate::app_state::AppContext;
use crate::error::AppError;
use crate::services;
use crate::models::OfferExtraction;

/// Récupère la page d'une offre et en extrait les champs.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `url` - Lien de l'offre.
///
/// # Returns
///
/// Champs extraits, ou l'échec typé de récupération.
#[tauri::command]
pub async fn scrape_offer(ctx: tauri::State<'_, AppContext>, url: String) -> Result<OfferExtraction, AppError> {
    services::scraping::scrape_offer(ctx.state()?.claude(), &url).await
}

/// Extrait les champs d'une offre depuis un texte collé.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `text` - Texte de l'offre.
///
/// # Returns
///
/// Champs extraits.
#[tauri::command]
pub async fn extract_offer_from_text(
    ctx: tauri::State<'_, AppContext>,
    text: String,
) -> Result<OfferExtraction, AppError> {
    services::scraping::extract_from_pasted_text(ctx.state()?.claude(), &text).await
}
