//! Endpoint REST pour le profil complet de l'utilisateur.

// Importation des crates externes
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use findmyjob_core::models::ProfileDto;

/// Assemble la route `/api/profile`.
pub fn router() -> Router<AppState> {
    Router::new().route("/profile", get(get_profile))
}

async fn get_profile(State(state): State<AppState>) -> Result<Json<ProfileDto>, ApiError> {
    Ok(Json(db::profile::build_profile(&state.pool).await?))
}
