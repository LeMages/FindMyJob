//! Endpoints REST pour les formations du profil.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateFormationInput, FormationView};

/// Assemble les routes `/api/formations*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/formations", post(add_formation))
        .route("/formations/{id}", put(update_formation).delete(delete_formation))
}

async fn add_formation(
    State(state): State<AppState>,
    Json(input): Json<CreateFormationInput>,
) -> Result<Json<FormationView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::formations::insert(&state.pool, user_id, input).await?))
}

async fn update_formation(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateFormationInput>,
) -> Result<Json<FormationView>, ApiError> {
    Ok(Json(db::formations::update(&state.pool, id, input).await?))
}

async fn delete_formation(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::formations::delete(&state.pool, id).await?;
    Ok(())
}
