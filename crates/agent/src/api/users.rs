//! Endpoints REST pour l'utilisateur unique de l'application.

// Importation des crates externes
use axum::extract::State;
use axum::routing::{get, patch, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::{CreateUserInput, UpdateUserInfoInput, UserView};

/// Assemble les routes `/api/users*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/exists", get(user_exists))
        .route("/users", post(create_user))
        .route("/users/me", get(get_user))
        .route("/users/description", patch(update_description))
        .route("/users/info", patch(update_user_info))
}

async fn user_exists(State(state): State<AppState>) -> Result<Json<bool>, ApiError> {
    Ok(Json(db::users::user_exists(&state.pool).await?))
}

async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<UserView>, ApiError> {
    Ok(Json(db::users::insert_user(&state.pool, input).await?))
}

async fn get_user(State(state): State<AppState>) -> Result<Json<UserView>, ApiError> {
    Ok(Json(db::users::fetch_first_user(&state.pool).await?))
}

async fn update_description(
    State(state): State<AppState>,
    Json(description): Json<Option<String>>,
) -> Result<Json<UserView>, ApiError> {
    Ok(Json(db::users::update_description(&state.pool, description).await?))
}

async fn update_user_info(
    State(state): State<AppState>,
    Json(input): Json<UpdateUserInfoInput>,
) -> Result<Json<UserView>, ApiError> {
    Ok(Json(db::users::update_user_info(&state.pool, input).await?))
}
