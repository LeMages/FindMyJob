//! Endpoints REST d'extraction IA d'une offre d'emploi, depuis une URL ou un
//! texte collé manuellement.

// Importation des crates externes
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::llm::client::DEFAULT_MODEL;
use findmyjob_core::llm::scraping_extract::{extract_offer_fields, ScrapedOfferFields};
use findmyjob_core::scraping::extract::{extract_text, truncate_chars, MAX_CHARS};
use findmyjob_core::scraping::fetch::fetch_html;

#[derive(serde::Deserialize)]
struct UrlInput {
    url: String,
}

#[derive(serde::Deserialize)]
struct TextInput {
    text: String,
}

/// Assemble les routes `/api/scraping/*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scraping/url", post(scrape_offer))
        .route("/scraping/text", post(extract_offer_from_text))
}

async fn scrape_offer(
    State(state): State<AppState>,
    Json(input): Json<UrlInput>,
) -> Result<Json<ScrapedOfferFields>, ApiError> {
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    let html = fetch_html(&input.url).await?;
    let text = extract_text(&html)?;
    Ok(Json(extract_offer_fields(&text, &model).await?))
}

async fn extract_offer_from_text(
    State(state): State<AppState>,
    Json(input): Json<TextInput>,
) -> Result<Json<ScrapedOfferFields>, ApiError> {
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    let text = truncate_chars(&input.text, MAX_CHARS);
    Ok(Json(extract_offer_fields(&text, &model).await?))
}
