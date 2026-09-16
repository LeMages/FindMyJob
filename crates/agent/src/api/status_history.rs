//! Endpoint REST pour l'historique de statut d'une candidature.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::StatusHistoryView;

/// Assemble la route `/api/candidatures/{id}/history`.
pub fn router() -> Router<AppState> {
    Router::new().route("/candidatures/{id}/history", get(get_status_history))
}

async fn get_status_history(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<StatusHistoryView>>, ApiError> {
    Ok(Json(db::status_history::list_for_candidature(&state.pool, id).await?))
}
