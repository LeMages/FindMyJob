//! Endpoints REST système : health check.

// Importation des crates externes
use axum::routing::get;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::AppState;

/// Assemble la route `/api/system/health`, exclue de l'authentification par
/// jeton : un check de santé doit rester accessible sans jeton (sondes de
/// supervision, load balancer) — voir `api::router`.
pub fn health_router() -> Router<AppState> {
    Router::new().route("/system/health", get(health))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
