//! Endpoints REST pour les compétences du profil.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateSkillInput, SkillView};

/// Assemble les routes `/api/skills*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/skills", post(add_skill))
        .route("/skills/{id}", put(update_skill).delete(delete_skill))
}

async fn add_skill(
    State(state): State<AppState>,
    Json(input): Json<CreateSkillInput>,
) -> Result<Json<SkillView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::skills::insert(&state.pool, user_id, input).await?))
}

async fn update_skill(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateSkillInput>,
) -> Result<Json<SkillView>, ApiError> {
    Ok(Json(db::skills::update(&state.pool, id, input).await?))
}

async fn delete_skill(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::skills::delete(&state.pool, id).await?;
    Ok(())
}
