//! Endpoints REST pour les contacts recruteur d'une candidature.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::ContactView;

/// Assemble la route `/api/candidatures/{id}/contacts`.
pub fn router() -> Router<AppState> {
    Router::new().route("/candidatures/{id}/contacts", get(list_contacts))
}

async fn list_contacts(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<ContactView>>, ApiError> {
    Ok(Json(db::contacts::list_for_candidature(&state.pool, id).await?))
}
