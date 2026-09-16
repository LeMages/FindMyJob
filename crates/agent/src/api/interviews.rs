//! Endpoints REST pour les entretiens.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{InterviewView, PlanningInterviewView};

/// Assemble les routes `/api/candidatures/{id}/interviews/pending` et
/// `/api/sessions/{id}/interviews`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/candidatures/{id}/interviews/pending", get(list_pending_interviews))
        .route("/sessions/{id}/interviews", get(list_interviews_for_session))
}

async fn list_pending_interviews(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<InterviewView>>, ApiError> {
    Ok(Json(db::interviews::list_pending_for_candidature(&state.pool, id).await?))
}

async fn list_interviews_for_session(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<Vec<PlanningInterviewView>>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::interviews::list_for_session(&state.pool, user_id, session_id).await?))
}
