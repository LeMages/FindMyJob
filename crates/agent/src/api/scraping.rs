//! Endpoints REST d'extraction d'une offre d'emploi : depuis son URL (page
//! récupérée par l'agent), depuis un HTML déjà capturé par la fenêtre du
//! desktop, ou depuis un texte collé manuellement.

// Importation des crates externes
use axum::extract::{DefaultBodyLimit, State};
use axum::routing::post;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use crate::offer_extraction::{extract_from_text, run_cascade};
use findmyjob_core::llm::client::DEFAULT_MODEL;
use findmyjob_core::models::OfferExtraction;
use findmyjob_core::scraping::fetch::fetch_html;
use findmyjob_core::scraping::url_validation::validate_offer_url;

/// Taille maximale du corps de `POST /api/scraping/html` (6 Mo). La limite
/// Axum par défaut (2 Mo) est trop basse pour une page d'offre capturée,
/// qui embarque souvent l'état hydraté d'une SPA.
const HTML_BODY_LIMIT: usize = 6 * 1024 * 1024;

#[derive(serde::Deserialize)]
struct UrlInput {
    url: String,
}

#[derive(serde::Deserialize)]
struct HtmlInput {
    url: String,
    html: String,
}

#[derive(serde::Deserialize)]
struct TextInput {
    text: String,
}

/// Assemble les routes `/api/scraping/*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scraping/url", post(scrape_offer))
        .route(
            "/scraping/html",
            post(extract_offer_from_html).layer(DefaultBodyLimit::max(HTML_BODY_LIMIT)),
        )
        .route("/scraping/text", post(extract_offer_from_text))
}

/// Récupère la page à l'URL donnée puis lance la cascade d'extraction.
async fn scrape_offer(
    State(state): State<AppState>,
    Json(input): Json<UrlInput>,
) -> Result<Json<OfferExtraction>, ApiError> {
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    let html = fetch_html(&input.url).await?;
    Ok(Json(run_cascade(&html, &model).await?))
}

/// Lance la cascade d'extraction sur un HTML déjà capturé par le desktop.
/// L'URL est validée comme pour une récupération, mais n'est pas chargée.
async fn extract_offer_from_html(
    State(state): State<AppState>,
    Json(input): Json<HtmlInput>,
) -> Result<Json<OfferExtraction>, ApiError> {
    validate_offer_url(&input.url)?;
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    Ok(Json(run_cascade(&input.html, &model).await?))
}

/// Extrait les champs depuis un texte d'offre collé manuellement.
async fn extract_offer_from_text(
    State(state): State<AppState>,
    Json(input): Json<TextInput>,
) -> Result<Json<OfferExtraction>, ApiError> {
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    Ok(Json(extract_from_text(&input.text, &model).await?))
}
