//! Endpoints REST pour les expériences professionnelles du profil.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateExperienceInput, ExperienceView};

/// Assemble les routes `/api/experiences*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/experiences", post(add_experience))
        .route("/experiences/{id}", put(update_experience).delete(delete_experience))
}

async fn add_experience(
    State(state): State<AppState>,
    Json(input): Json<CreateExperienceInput>,
) -> Result<Json<ExperienceView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::experiences::insert(&state.pool, user_id, input).await?))
}

async fn update_experience(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateExperienceInput>,
) -> Result<Json<ExperienceView>, ApiError> {
    Ok(Json(db::experiences::update(&state.pool, id, input).await?))
}

async fn delete_experience(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::experiences::delete(&state.pool, id).await?;
    Ok(())
}
