//! Endpoints REST système : diagnostic de la base de données, health check,
//! export complet des données.

// Importation des crates externes
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;

/// Résultat du diagnostic de connexion PostgreSQL.
#[derive(serde::Serialize)]
struct DbStatusResult {
    db_path: String,
    tables: Vec<String>,
}

/// Assemble les routes `/api/system/db-status` et `/api/export`, protégées
/// par l'authentification (voir `health_router` pour `/api/system/health`,
/// volontairement non protégée).
pub fn router() -> Router<AppState> {
    Router::new().route("/system/db-status", get(db_status)).route("/export", post(export_data))
}

/// Assemble la route `/api/system/health`, exclue de l'authentification par
/// jeton : un check de santé doit rester accessible sans jeton (sondes de
/// supervision, load balancer) — voir `api::router`.
pub fn health_router() -> Router<AppState> {
    Router::new().route("/system/health", get(health))
}

async fn db_status(State(state): State<AppState>) -> Result<Json<DbStatusResult>, ApiError> {
    let (db_path, tables) = db::system::status(&state.pool).await?;
    Ok(Json(DbStatusResult { db_path, tables }))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn export_data(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let data = db::export::export_all_data(&state.pool).await?;
    Ok(Json(serde_json::Value::Object(data)))
}
