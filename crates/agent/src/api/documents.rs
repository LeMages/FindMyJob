//! Endpoints REST pour les documents de référence (CV, lettre de motivation)
//! et les documents joints à une candidature.

// Importation des crates externes
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::{db, documents};
use findmyjob_core::models::{DocumentFileInfo, DocumentReferenceView, DocumentsStatus};

#[derive(serde::Deserialize)]
struct SaveDocumentInput {
    filename: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct DocumentPathQuery {
    path: String,
}

/// Assemble les routes `/api/sessions/{id}/documents/*` et `/api/documents/*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/{id}/documents/cv", get(get_cv_reference).post(save_cv))
        .route("/sessions/{id}/documents/lm", get(get_lm_reference).post(save_lettre_motivation))
        .route("/sessions/{id}/documents/cv/compile", post(compile_cv))
        .route("/sessions/{id}/documents/lm/compile", post(compile_lettre_motivation))
        .route("/sessions/{id}/documents/status", get(get_documents_status))
        .route("/documents/info", get(get_document_info))
        .route("/documents/pdf", get(get_document_pdf))
}

async fn get_cv_reference(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Json<Option<DocumentReferenceView>> {
    Json(documents::get_reference(&state.pool, &state.data_dir, session_id, "cv").await)
}

async fn get_lm_reference(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Json<Option<DocumentReferenceView>> {
    Json(documents::get_reference(&state.pool, &state.data_dir, session_id, "lettre_motivation").await)
}

async fn save_cv(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    Json(input): Json<SaveDocumentInput>,
) -> Result<Json<DocumentReferenceView>, ApiError> {
    let view = documents::save_reference(
        &state.pool,
        &state.data_dir,
        &state.fonts_dir,
        session_id,
        "cv",
        input.filename,
        input.content,
        |_, _| {},
    )
    .await?;
    Ok(Json(view))
}

async fn save_lettre_motivation(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    Json(input): Json<SaveDocumentInput>,
) -> Result<Json<DocumentReferenceView>, ApiError> {
    let view = documents::save_reference(
        &state.pool,
        &state.data_dir,
        &state.fonts_dir,
        session_id,
        "lettre_motivation",
        input.filename,
        input.content,
        |_, _| {},
    )
    .await?;
    Ok(Json(view))
}

async fn compile_cv(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<String>, ApiError> {
    Ok(Json(documents::compile_reference(&state.data_dir, &state.fonts_dir, session_id, "cv").await?))
}

async fn compile_lettre_motivation(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<String>, ApiError> {
    Ok(Json(
        documents::compile_reference(&state.data_dir, &state.fonts_dir, session_id, "lettre_motivation").await?,
    ))
}

async fn get_documents_status(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<DocumentsStatus>, ApiError> {
    Ok(Json(db::documents::status_for_session(&state.pool, session_id).await?))
}

async fn get_document_info(Query(query): Query<DocumentPathQuery>) -> Json<DocumentFileInfo> {
    Json(documents::document_info(&query.path))
}

async fn get_document_pdf(Query(query): Query<DocumentPathQuery>) -> Result<Json<String>, ApiError> {
    Ok(Json(documents::read_document_base64(&query.path)?))
}
