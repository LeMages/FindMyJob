//! Commandes Tauri pour l'historique des documents de référence (CV, lettre
//! de motivation) d'une session : liste des versions, aperçu PDF d'une
//! version, restauration d'une version comme référence courante.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::documents;
use crate::error::AppError;
use crate::models::{DocumentReferenceView, DocumentVersionView};

/// Liste les versions d'une référence.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Versions, de la plus récente à la plus ancienne.
#[tauri::command]
pub async fn get_document_versions(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
    doc_type: String,
) -> Result<Vec<DocumentVersionView>, AppError> {
    let doc_type = documents::parse_doc_type(&doc_type)?;
    db::document_versions::list_versions(&ctx.state()?.pool, session_id, doc_type).await
}

/// Lit le PDF d'une version pour aperçu.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro de la version.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn get_document_version_pdf(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
    doc_type: String,
    version: i32,
) -> Result<String, AppError> {
    let doc_type = documents::parse_doc_type(&doc_type)?;
    let state = ctx.state()?;
    documents::read_version_pdf(&state.pool, &state.data_dir, session_id, doc_type, version).await
}

/// Restaure une version comme référence courante.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro de la version.
///
/// # Returns
///
/// `DocumentReferenceView` de la nouvelle référence courante.
#[tauri::command]
pub async fn restore_document_version(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
    doc_type: String,
    version: i32,
) -> Result<DocumentReferenceView, AppError> {
    let doc_type = documents::parse_doc_type(&doc_type)?;
    let state = ctx.state()?;
    documents::restore_version(&state.pool, &state.data_dir, session_id, doc_type, version).await
}
