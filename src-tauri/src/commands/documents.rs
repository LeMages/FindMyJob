//! Commandes Tauri pour la gestion des documents : CV et lettre de motivation
//! de référence (format Typst), et documents PDF joints à une candidature.

// Importation des modules internes
use crate::documents;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView};

/// Récupère la référence de CV sauvegardée, si elle existe.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si un CV de référence a été sauvegardé.
#[tauri::command]
pub async fn get_cv_reference(app: tauri::AppHandle) -> Result<Option<DocumentReferenceView>, AppError> {
    Ok(documents::get_reference(&app, "cv"))
}

/// Récupère la référence de lettre de motivation sauvegardée, si elle existe.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si une lettre de référence a été sauvegardée.
#[tauri::command]
pub async fn get_lettre_motivation_reference(
    app: tauri::AppHandle,
) -> Result<Option<DocumentReferenceView>, AppError> {
    Ok(documents::get_reference(&app, "lettre_motivation"))
}

/// Sauvegarde une nouvelle référence de CV (.typ) et la compile en PDF.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `filename` - Nom du fichier fourni par l'utilisateur.
/// * `content` - Contenu Typst du CV.
///
/// # Returns
///
/// `DocumentReferenceView` du CV sauvegardé.
#[tauri::command]
pub async fn save_cv(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    documents::save_reference(app, "cv", filename, content).await
}

/// Sauvegarde une nouvelle référence de lettre de motivation (.typ) et la compile en PDF.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `filename` - Nom du fichier fourni par l'utilisateur.
/// * `content` - Contenu Typst de la lettre.
///
/// # Returns
///
/// `DocumentReferenceView` de la lettre sauvegardée.
#[tauri::command]
pub async fn save_lettre_motivation(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    documents::save_reference(app, "lettre_motivation", filename, content).await
}

/// Compile la référence de CV sauvegardée en PDF encodé base64.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_cv(app: tauri::AppHandle) -> Result<String, AppError> {
    documents::compile_reference(app, "cv").await
}

/// Compile la référence de lettre de motivation sauvegardée en PDF encodé base64.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_lettre_motivation(app: tauri::AppHandle) -> Result<String, AppError> {
    documents::compile_reference(app, "lettre_motivation").await
}

/// Résout le dossier de référence du CV (utilisé comme dossier par défaut du sélecteur de fichiers).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Chemin absolu du dossier.
#[tauri::command]
pub async fn get_cv_reference_dir(app: tauri::AppHandle) -> Result<String, AppError> {
    documents::resolve_reference_dir(&app, "cv")
}

/// Résout le dossier de référence de la lettre de motivation.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Chemin absolu du dossier.
#[tauri::command]
pub async fn get_lettre_motivation_reference_dir(app: tauri::AppHandle) -> Result<String, AppError> {
    documents::resolve_reference_dir(&app, "lettre_motivation")
}

/// Met en scène un CV envoyé en base64 (glisser-déposer) avant rattachement à une candidature.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `filename` - Nom du fichier d'origine.
/// * `content` - Contenu du fichier encodé en base64.
///
/// # Returns
///
/// Chemin temporaire du fichier mis en scène.
#[tauri::command]
pub async fn upload_candidature_cv(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<String, AppError> {
    documents::upload_candidature_document(app, "cv", filename, content).await
}

/// Met en scène une lettre de motivation envoyée en base64 avant rattachement à une candidature.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `filename` - Nom du fichier d'origine.
/// * `content` - Contenu du fichier encodé en base64.
///
/// # Returns
///
/// Chemin temporaire du fichier mis en scène.
#[tauri::command]
pub async fn upload_candidature_lettre_motivation(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<String, AppError> {
    documents::upload_candidature_document(app, "lettre_motivation", filename, content).await
}

/// Met en scène un CV sélectionné via l'explorateur de fichiers natif.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `path` - Chemin absolu du fichier sélectionné.
///
/// # Returns
///
/// Chemin temporaire du fichier mis en scène.
#[tauri::command]
pub async fn select_candidature_cv(app: tauri::AppHandle, path: String) -> Result<String, AppError> {
    documents::select_candidature_document(app, "cv", path).await
}

/// Met en scène une lettre de motivation sélectionnée via l'explorateur de fichiers natif.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `path` - Chemin absolu du fichier sélectionné.
///
/// # Returns
///
/// Chemin temporaire du fichier mis en scène.
#[tauri::command]
pub async fn select_candidature_lettre_motivation(
    app: tauri::AppHandle,
    path: String,
) -> Result<String, AppError> {
    documents::select_candidature_document(app, "lettre_motivation", path).await
}

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

/// Ouvre un document joint à une candidature avec l'application par défaut du système.
///
/// # Arguments
///
/// * `path` - Chemin absolu du document à ouvrir.
///
/// # Returns
///
/// `Ok(())` si l'application associée a pu être lancée.
#[tauri::command]
pub fn open_document(path: String) -> Result<(), AppError> {
    open::that(path).map_err(|e| AppError::Validation(e.to_string()))
}
