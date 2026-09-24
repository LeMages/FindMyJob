//! Endpoints REST pour l'historique des documents de référence (CV, lettre
//! de motivation) d'une session : liste des versions, PDF d'une version,
//! restauration d'une version comme référence courante.
//!
//! Le segment `{kind}` suit la convention des routes de référence existantes
//! (`/sessions/{id}/documents/cv|lm`, voir `api::documents`).

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::{db, documents};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{DocumentReferenceView, DocumentVersionView};

/// Assemble les routes `/api/sessions/{id}/documents/{kind}/versions*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/{id}/documents/{kind}/versions", get(list_versions))
        .route("/sessions/{id}/documents/{kind}/versions/latest", get(get_latest_version_pdf))
        .route("/sessions/{id}/documents/{kind}/versions/{version}", get(get_version_pdf))
        .route("/sessions/{id}/documents/{kind}/versions/{version}/restore", post(restore_version))
}

/// Convertit le segment d'URL `{kind}` en `doc_type` stocké en base.
///
/// # Arguments
///
/// * `kind` - `"cv"` ou `"lm"`.
///
/// # Returns
///
/// `"cv"` ou `"lettre_motivation"`, `AppError::Validation` pour toute autre valeur.
fn doc_type_from_kind(kind: &str) -> Result<&'static str, AppError> {
    match kind {
        "cv" => Ok("cv"),
        "lm" => Ok("lettre_motivation"),
        other => Err(AppError::Validation(format!(
            "Type de document inconnu : « {other} » (attendu : cv ou lm)."
        ))),
    }
}

async fn list_versions(
    State(state): State<AppState>,
    Path((session_id, kind)): Path<(i64, String)>,
) -> Result<Json<Vec<DocumentVersionView>>, ApiError> {
    let doc_type = doc_type_from_kind(&kind)?;
    Ok(Json(db::document_versions::list_versions(&state.pool, session_id, doc_type).await?))
}

async fn get_latest_version_pdf(
    State(state): State<AppState>,
    Path((session_id, kind)): Path<(i64, String)>,
) -> Result<Json<String>, ApiError> {
    let doc_type = doc_type_from_kind(&kind)?;
    Ok(Json(documents::read_latest_version_pdf(&state.pool, session_id, doc_type).await?))
}

async fn get_version_pdf(
    State(state): State<AppState>,
    Path((session_id, kind, version)): Path<(i64, String, i32)>,
) -> Result<Json<String>, ApiError> {
    let doc_type = doc_type_from_kind(&kind)?;
    Ok(Json(documents::read_version_pdf(&state.pool, session_id, doc_type, version).await?))
}

async fn restore_version(
    State(state): State<AppState>,
    Path((session_id, kind, version)): Path<(i64, String, i32)>,
) -> Result<Json<DocumentReferenceView>, ApiError> {
    let doc_type = doc_type_from_kind(&kind)?;
    Ok(Json(documents::restore_version(&state.pool, session_id, doc_type, version).await?))
}
