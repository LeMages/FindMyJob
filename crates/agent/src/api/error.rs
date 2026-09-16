//! Conversion de `AppError` en réponse HTTP.
//!
//! `AppError` (défini dans `findmyjob-core`) et `axum::response::IntoResponse`
//! sont tous deux étrangers à ce crate : la règle de cohérence de Rust
//! interdit `impl IntoResponse for AppError` directement (même contrainte que
//! `db::common::map_db_err` vis-à-vis de `sqlx::Error`, voir ce fichier).
//! `ApiError` enveloppe `AppError` pour porter l'implémentation localement.

// Importation des crates externes
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Erreur applicative convertible en réponse HTTP, pour être utilisée comme
/// type d'erreur des handlers Axum (`Result<Json<T>, ApiError>`).
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::Database(_) | AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Llm(_) | AppError::Scraping(_) => StatusCode::BAD_GATEWAY,
        };
        (status, Json(self.0)).into_response()
    }
}
