//! Commandes Tauri pour la gestion des documents : CV et lettre de motivation
//! de référence (format Typst), et documents PDF joints à une candidature.
//! Chaque document de référence (et ses pièces jointes) appartient à une
//! session de candidature, identifiée par `session_id`.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use serde_json::json;
use tauri::Manager;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView, DocumentsStatus};

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

/// Récupère l'état de préparation des documents de référence (CV, lettre de
/// motivation) d'une session — conditionne la création d'une candidature.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `DocumentsStatus` reflétant l'état des deux documents de référence.
#[tauri::command]
pub async fn get_documents_status(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<DocumentsStatus, AppError> {
    api.get(&format!("/api/sessions/{session_id}/documents/status")).await
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

/// Télécharge un document joint à une candidature depuis l'agent et l'écrit
/// sur le disque local, en écrasant un éventuel fichier existant. Le dossier
/// parent est créé s'il n'existe pas.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `doc_path` - Chemin du document côté agent.
/// * `save_path` - Chemin complet de destination (dossier + nom de fichier).
///
/// # Returns
///
/// Chemin du fichier sauvegardé.
#[tauri::command]
pub async fn download_document(
    api: tauri::State<'_, ApiClient>,
    doc_path: String,
    save_path: String,
) -> Result<String, AppError> {
    // ------------------------------------------------------------------
    // 1. Récupération du PDF brut
    // ------------------------------------------------------------------
    let bytes = api
        .get_bytes_query("/api/documents/download", &[("path", doc_path.as_str())])
        .await?;

    // ------------------------------------------------------------------
    // 2. Écriture locale
    // ------------------------------------------------------------------
    write_local_file(&save_path, &bytes)?;
    Ok(save_path)
}

/// Écrit un fichier texte sur le disque local (ex. mail ou message de
/// candidature rédigé par l'IA, exporté à côté des PDF), en écrasant un
/// éventuel fichier existant. Le dossier parent est créé s'il n'existe pas.
///
/// # Arguments
///
/// * `save_path` - Chemin complet de destination.
/// * `content` - Contenu texte à écrire.
///
/// # Returns
///
/// Chemin du fichier sauvegardé.
#[tauri::command]
pub fn save_text_file(save_path: String, content: String) -> Result<String, AppError> {
    write_local_file(&save_path, content.as_bytes())?;
    Ok(save_path)
}

/// Écrit `bytes` dans `save_path` (écrasement), après création du dossier parent.
fn write_local_file(save_path: &str, bytes: &[u8]) -> Result<(), AppError> {
    let target = Path::new(save_path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Validation(format!("Impossible de créer le dossier {} : {e}", parent.display()))
        })?;
    }
    std::fs::write(target, bytes)
        .map_err(|e| AppError::Validation(format!("Impossible d'écrire le fichier {save_path} : {e}")))
}

/// Retourne le dossier de téléchargements de l'utilisateur.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri (accès aux chemins système).
///
/// # Returns
///
/// Chemin absolu du dossier de téléchargements.
#[tauri::command]
pub fn get_downloads_dir(app: tauri::AppHandle) -> Result<String, AppError> {
    let dir = app
        .path()
        .download_dir()
        .map_err(|e| AppError::Config(format!("Dossier de téléchargements introuvable : {e}")))?;
    Ok(dir.to_string_lossy().to_string())
}
