//! Endpoints REST système : health check, export complet des données.

// Importation des crates externes
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;

/// Assemble la route `/api/export`, protégée par l'authentification (voir
/// `health_router` pour `/api/system/health`, volontairement non protégée).
pub fn router() -> Router<AppState> {
    Router::new().route("/export", post(export_data))
}

/// Assemble la route `/api/system/health`, exclue de l'authentification par
/// jeton : un check de santé doit rester accessible sans jeton (sondes de
/// supervision, load balancer) — voir `api::router`.
pub fn health_router() -> Router<AppState> {
    Router::new().route("/system/health", get(health))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn export_data(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let data = db::export::export_all_data(&state.pool).await?;
    Ok(Json(serde_json::Value::Object(data)))
}
