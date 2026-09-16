//! Endpoints REST pour les langues du profil.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateLangInput, LangView};

/// Assemble les routes `/api/langues*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/langues", post(add_lang))
        .route("/langues/{id}", put(update_lang).delete(delete_lang))
}

async fn add_lang(
    State(state): State<AppState>,
    Json(input): Json<CreateLangInput>,
) -> Result<Json<LangView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::langues::insert(&state.pool, user_id, input).await?))
}

async fn update_lang(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateLangInput>,
) -> Result<Json<LangView>, ApiError> {
    Ok(Json(db::langues::update(&state.pool, id, input).await?))
}

async fn delete_lang(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::langues::delete(&state.pool, id).await?;
    Ok(())
}
