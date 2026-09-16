//! Commandes Tauri pour la gestion des documents : CV et lettre de motivation
//! de référence (format Typst), et documents PDF joints à une candidature.
//! Chaque document de référence (et ses pièces jointes) appartient à une
//! session de candidature, identifiée par `session_id`.

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView};

/// Récupère la référence de CV sauvegardée pour une session, si elle existe.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si un CV de référence a été sauvegardé.
#[tauri::command]
pub async fn get_cv_reference(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<Option<DocumentReferenceView>, AppError> {
    api.get(&format!("/api/sessions/{session_id}/documents/cv")).await
}

/// Récupère la référence de lettre de motivation sauvegardée pour une
/// session, si elle existe.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si une lettre de référence a été sauvegardée.
#[tauri::command]
pub async fn get_lettre_motivation_reference(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<Option<DocumentReferenceView>, AppError> {
    api.get(&format!("/api/sessions/{session_id}/documents/lm")).await
}

/// Sauvegarde une nouvelle référence de CV (.typ) pour une session et la
/// compile en PDF.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `filename` - Nom du fichier fourni par l'utilisateur.
/// * `content` - Contenu Typst du CV.
///
/// # Returns
///
/// `DocumentReferenceView` du CV sauvegardé.
#[tauri::command]
pub async fn save_cv(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    api.post(
        &format!("/api/sessions/{session_id}/documents/cv"),
        &json!({ "filename": filename, "content": content }),
    )
    .await
}

/// Sauvegarde une nouvelle référence de lettre de motivation (.typ) pour une
/// session et la compile en PDF.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `filename` - Nom du fichier fourni par l'utilisateur.
/// * `content` - Contenu Typst de la lettre.
///
/// # Returns
///
/// `DocumentReferenceView` de la lettre sauvegardée.
#[tauri::command]
pub async fn save_lettre_motivation(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    api.post(
        &format!("/api/sessions/{session_id}/documents/lm"),
        &json!({ "filename": filename, "content": content }),
    )
    .await
}

/// Compile la référence de CV sauvegardée d'une session en PDF encodé base64.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_cv(api: tauri::State<'_, ApiClient>, session_id: i64) -> Result<String, AppError> {
    api.post(&format!("/api/sessions/{session_id}/documents/cv/compile"), &())
        .await
}

/// Compile la référence de lettre de motivation sauvegardée d'une session en
/// PDF encodé base64.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_lettre_motivation(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<String, AppError> {
    api.post(&format!("/api/sessions/{session_id}/documents/lm/compile"), &())
        .await
}

/// Inspecte l'état sur disque d'un document joint à une candidature (existence, taille).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `path` - Chemin absolu du document.
///
/// # Returns
///
/// `DocumentFileInfo` reflétant l'état réel du fichier sur disque.
#[tauri::command]
pub async fn get_document_info(
    api: tauri::State<'_, ApiClient>,
    path: String,
) -> Result<DocumentFileInfo, AppError> {
    api.get_query("/api/documents/info", &[("path", path.as_str())]).await
}

/// Lit un document joint à une candidature (CV, lettre de motivation) et
/// retourne son contenu PDF encodé en base64, pour un aperçu inline dans une iframe.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `path` - Chemin absolu du document PDF.
///
/// # Returns
///
/// Contenu du PDF encodé en base64.
#[tauri::command]
pub async fn get_document_pdf(api: tauri::State<'_, ApiClient>, path: String) -> Result<String, AppError> {
    api.get_query("/api/documents/pdf", &[("path", path.as_str())]).await
}
