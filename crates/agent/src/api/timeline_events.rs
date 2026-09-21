//! Endpoints REST pour la timeline de suivi d'une candidature.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{AvailableEvent, NewTimelineEvent};

/// Assemble les routes `/api/candidatures/{id}/events`.
pub fn router() -> Router<AppState> {
    Router::new().route("/candidatures/{id}/events", get(get_available_events).post(add_timeline_event))
}

async fn get_available_events(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<AvailableEvent>>, ApiError> {
    let current_status = db::candidatures::fetch_status(&state.pool, id).await?;
    let has_pending = db::interviews::count_pending(&state.pool, id).await? > 0;
    Ok(Json(db::timeline_events::available_events(&current_status, has_pending)))
}

async fn add_timeline_event(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(event): Json<NewTimelineEvent>,
) -> Result<(), ApiError> {
    db::timeline_events::add_event(&state.pool, id, event).await?;
    Ok(())
}
