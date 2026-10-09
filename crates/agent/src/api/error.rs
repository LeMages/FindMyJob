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
use findmyjob_core::scraping::failure::ScrapingFailure;

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
            AppError::Llm(_) => StatusCode::BAD_GATEWAY,
            AppError::Scraping(failure) => scraping_status(*failure),
        };
        (status, Json(self.0)).into_response()
    }
}

/// Statut HTTP d'un échec de récupération/extraction. Indicatif seulement :
/// le desktop lit la raison dans le corps (`{"kind":"Scraping","message":…}`).
fn scraping_status(failure: ScrapingFailure) -> StatusCode {
    match failure {
        ScrapingFailure::InvalidUrl => StatusCode::BAD_REQUEST,
        ScrapingFailure::OfferUnavailable => StatusCode::NOT_FOUND,
        ScrapingFailure::Timeout => StatusCode::GATEWAY_TIMEOUT,
        ScrapingFailure::AccessDenied | ScrapingFailure::Network => StatusCode::BAD_GATEWAY,
        ScrapingFailure::NotHtml
        | ScrapingFailure::EmptyPage
        | ScrapingFailure::TooLarge
        | ScrapingFailure::InsufficientContent => StatusCode::UNPROCESSABLE_ENTITY,
    }
}
