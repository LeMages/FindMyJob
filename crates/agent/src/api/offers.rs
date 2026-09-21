//! Endpoints REST pour les offres scrapées et scorées par le pipeline.

// Importation des crates externes
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::OfferView;

/// Seuil de score appliqué quand le client n'en fournit pas.
const DEFAULT_MIN_SCORE: f32 = 0.5;

/// Paramètres de requête de `GET /sessions/{session_id}/offers`.
#[derive(Deserialize)]
struct SessionOffersQuery {
    min_score: Option<f32>,
}

/// Assemble les routes `/api/offers*` et `/api/sessions/{session_id}/offers`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}/offers", get(list_session_offers))
        .route("/offers/{id}", get(get_offer))
        .route("/offers/{id}/dismiss", post(dismiss_offer))
        .route("/offers/{id}/applied", post(mark_offer_applied))
}

async fn list_session_offers(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    Query(query): Query<SessionOffersQuery>,
) -> Result<Json<Vec<OfferView>>, ApiError> {
    let min_score = query.min_score.unwrap_or(DEFAULT_MIN_SCORE);
    Ok(Json(db::offers::list_for_session(&state.pool, session_id, min_score).await?))
}

async fn get_offer(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<OfferView>, ApiError> {
    Ok(Json(db::offers::fetch_one(&state.pool, id).await?))
}

async fn dismiss_offer(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::offers::dismiss(&state.pool, id).await?;
    Ok(())
}

async fn mark_offer_applied(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::offers::mark_applied(&state.pool, id).await?;
    Ok(())
}
