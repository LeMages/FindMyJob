//! Commandes Tauri pour la gestion des documents : CV et lettre de motivation
//! de référence (format Typst), et documents PDF joints à une candidature.
//! Chaque document de référence (et ses pièces jointes) appartient à une
//! session de candidature, identifiée par `session_id`.

// Importation des modules internes
use crate::documents;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView};

/// Récupère la référence de CV sauvegardée pour une session, si elle existe.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si un CV de référence a été sauvegardé.
#[tauri::command]
pub async fn get_cv_reference(
    app: tauri::AppHandle,
    session_id: i64,
) -> Result<Option<DocumentReferenceView>, AppError> {
    Ok(documents::get_reference(&app, session_id, "cv").await)
}

/// Récupère la référence de lettre de motivation sauvegardée pour une
/// session, si elle existe.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si une lettre de référence a été sauvegardée.
#[tauri::command]
pub async fn get_lettre_motivation_reference(
    app: tauri::AppHandle,
    session_id: i64,
) -> Result<Option<DocumentReferenceView>, AppError> {
    Ok(documents::get_reference(&app, session_id, "lettre_motivation").await)
}

/// Sauvegarde une nouvelle référence de CV (.typ) pour une session et la
/// compile en PDF.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `filename` - Nom du fichier fourni par l'utilisateur.
/// * `content` - Contenu Typst du CV.
///
/// # Returns
///
/// `DocumentReferenceView` du CV sauvegardé.
#[tauri::command]
pub async fn save_cv(
    app: tauri::AppHandle,
    session_id: i64,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    documents::save_reference(app, session_id, "cv", filename, content).await
}

/// Sauvegarde une nouvelle référence de lettre de motivation (.typ) pour une
/// session et la compile en PDF.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `filename` - Nom du fichier fourni par l'utilisateur.
/// * `content` - Contenu Typst de la lettre.
///
/// # Returns
///
/// `DocumentReferenceView` de la lettre sauvegardée.
#[tauri::command]
pub async fn save_lettre_motivation(
    app: tauri::AppHandle,
    session_id: i64,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    documents::save_reference(app, session_id, "lettre_motivation", filename, content).await
}

/// Compile la référence de CV sauvegardée d'une session en PDF encodé base64.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_cv(app: tauri::AppHandle, session_id: i64) -> Result<String, AppError> {
    documents::compile_reference(app, session_id, "cv").await
}

/// Compile la référence de lettre de motivation sauvegardée d'une session en
/// PDF encodé base64.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_lettre_motivation(
    app: tauri::AppHandle,
    session_id: i64,
) -> Result<String, AppError> {
    documents::compile_reference(app, session_id, "lettre_motivation").await
}

/// Résout le dossier de référence du CV d'une session (utilisé comme dossier
/// par défaut du sélecteur de fichiers).
/// Inspecte l'état sur disque d'un document joint à une candidature (existence, taille).
///
/// # Arguments
///
/// * `path` - Chemin absolu du document.
///
/// # Returns
///
/// `DocumentFileInfo` reflétant l'état réel du fichier sur disque.
#[tauri::command]
pub fn get_document_info(path: String) -> Result<DocumentFileInfo, AppError> {
    Ok(documents::document_info(&path))
}

/// Lit un document joint à une candidature (CV, lettre de motivation) et
/// retourne son contenu PDF encodé en base64, pour un aperçu inline dans une iframe.
///
/// # Arguments
///
/// * `path` - Chemin absolu du document PDF.
///
/// # Returns
///
/// Contenu du PDF encodé en base64.
#[tauri::command]
pub fn get_document_pdf(path: String) -> Result<String, AppError> {
    documents::read_document_base64(&path)
}
