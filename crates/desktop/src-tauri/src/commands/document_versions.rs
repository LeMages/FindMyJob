//! Commandes Tauri pour l'historique des documents de référence (CV, lettre
//! de motivation) d'une session : liste des versions, aperçu PDF d'une
//! version, restauration d'une version comme référence courante.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{DocumentReferenceView, DocumentVersionView};

/// Convertit un `doc_type` (`"cv"` / `"lettre_motivation"`) en segment
/// d'URL de l'agent (`cv` / `lm`, voir `api::document_versions`).
///
/// # Arguments
///
/// * `doc_type` - Type de document reçu du frontend.
///
/// # Returns
///
/// Segment d'URL, ou `AppError::Validation` pour un type inconnu.
fn url_kind(doc_type: &str) -> Result<&'static str, AppError> {
    match doc_type {
        "cv" => Ok("cv"),
        "lettre_motivation" => Ok("lm"),
        other => Err(AppError::Validation(format!("Type de document inconnu : « {other} »."))),
    }
}

/// Liste les versions d'un document de référence d'une session, de la plus
/// récente à la plus ancienne.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Versions du document (vide pour une référence antérieure au versioning).
#[tauri::command]
pub async fn get_document_versions(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    doc_type: String,
) -> Result<Vec<DocumentVersionView>, AppError> {
    let kind = url_kind(&doc_type)?;
    api.get(&format!("/api/sessions/{session_id}/documents/{kind}/versions")).await
}

/// Récupère le PDF d'une version précise d'un document de référence.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro de la version.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn get_document_version_pdf(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    doc_type: String,
    version: i32,
) -> Result<String, AppError> {
    let kind = url_kind(&doc_type)?;
    api.get(&format!("/api/sessions/{session_id}/documents/{kind}/versions/{version}")).await
}

/// Restaure une version d'un document de référence comme référence courante
/// de la session (utilisée pour les prochaines générations).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro de la version à restaurer.
///
/// # Returns
///
/// `DocumentReferenceView` de la nouvelle référence courante.
#[tauri::command]
pub async fn restore_document_version(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    doc_type: String,
    version: i32,
) -> Result<DocumentReferenceView, AppError> {
    let kind = url_kind(&doc_type)?;
    api.post(&format!("/api/sessions/{session_id}/documents/{kind}/versions/{version}/restore"), &())
        .await
}
