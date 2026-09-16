//! Endpoints REST pour les sessions de candidature.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateSessionInput, SessionView, UpdateSessionInput};

/// Assemble les routes `/api/sessions*` (hors sous-ressources candidatures/entretiens).
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/{id}", put(update_session).delete(delete_session))
}

async fn list_sessions(State(state): State<AppState>) -> Result<Json<Vec<SessionView>>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::sessions::list_for_user(&state.pool, user_id).await?))
}

async fn create_session(
    State(state): State<AppState>,
    Json(input): Json<CreateSessionInput>,
) -> Result<Json<SessionView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::sessions::insert(&state.pool, user_id, &input.nom).await?))
}

async fn update_session(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateSessionInput>,
) -> Result<Json<SessionView>, ApiError> {
    Ok(Json(db::sessions::update(&state.pool, id, &input).await?))
}

async fn delete_session(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::sessions::delete(&state.pool, id).await?;
    Ok(())
}
