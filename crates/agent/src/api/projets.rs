//! Endpoints REST pour les projets du profil.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateProjetInput, ProjetView};

/// Assemble les routes `/api/projets*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projets", post(add_projet))
        .route("/projets/{id}", put(update_projet).delete(delete_projet))
}

async fn add_projet(
    State(state): State<AppState>,
    Json(input): Json<CreateProjetInput>,
) -> Result<Json<ProjetView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::projets::insert(&state.pool, user_id, input).await?))
}

async fn update_projet(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateProjetInput>,
) -> Result<Json<ProjetView>, ApiError> {
    Ok(Json(db::projets::update(&state.pool, id, input).await?))
}

async fn delete_projet(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::projets::delete(&state.pool, id).await?;
    Ok(())
}
