//! Endpoints REST pour les offres recommandées par le scraper.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use crate::db::offers::OfferView;

/// Assemble les routes `/api/offers*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/offers", get(list_offers))
        .route("/offers/{id}", get(get_offer))
        .route("/offers/{id}/dismiss", post(dismiss_offer))
        .route("/offers/{id}/applied", post(mark_offer_applied))
}

async fn list_offers(State(state): State<AppState>) -> Result<Json<Vec<OfferView>>, ApiError> {
    Ok(Json(db::offers::list_new(&state.pool).await?))
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
