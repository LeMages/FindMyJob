//! Commandes Tauri pour la gestion des documents : CV et lettre de motivation
//! de référence (format Typst), et documents PDF joints à une candidature.
//!
//! Chaque document de référence (et ses pièces jointes) appartient à une
//! session de candidature, identifiée par `session_id`. Les chemins de
//! documents reçus de l'interface sont relatifs au dossier de données et
//! résolus sous celui-ci (voir `storage_paths::resolve_untrusted`).

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use tauri::Manager;

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::documents;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView, DocumentsStatus};

/// Récupère le CV de référence courant d'une session.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si un CV est défini et présent sur disque.
#[tauri::command]
pub async fn get_cv_reference(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
) -> Result<Option<DocumentReferenceView>, AppError> {
    let state = ctx.state()?;
    documents::get_reference(&state.pool, &state.data_dir, session_id, "cv").await
}

/// Récupère la lettre de motivation de référence courante d'une session.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si une lettre est définie et présente sur disque.
#[tauri::command]
pub async fn get_lettre_motivation_reference(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
) -> Result<Option<DocumentReferenceView>, AppError> {
    let state = ctx.state()?;
    documents::get_reference(&state.pool, &state.data_dir, session_id, "lettre_motivation").await
}

/// Importe un CV de référence `.typ` : compilation, templatisation par
/// l'IA, puis enregistrement comme nouvelle version courante.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
/// * `filename` - Nom du fichier importé.
/// * `content` - Source Typst.
///
/// # Returns
///
/// `DocumentReferenceView` de la nouvelle référence.
#[tauri::command]
pub async fn save_cv(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    let state = ctx.state()?;
    documents::save_reference(&state.pool, &state.data_dir, &state.fonts_dir, session_id, "cv", filename, content)
        .await
}

/// Importe une lettre de motivation de référence `.typ` (voir `save_cv`).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
/// * `filename` - Nom du fichier importé.
/// * `content` - Source Typst.
///
/// # Returns
///
/// `DocumentReferenceView` de la nouvelle référence.
#[tauri::command]
pub async fn save_lettre_motivation(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    let state = ctx.state()?;
    documents::save_reference(
        &state.pool,
        &state.data_dir,
        &state.fonts_dir,
        session_id,
        "lettre_motivation",
        filename,
        content,
    )
    .await
}

/// Recompile le CV de référence courant pour aperçu.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_cv(ctx: tauri::State<'_, AppContext>, session_id: i64) -> Result<String, AppError> {
    let state = ctx.state()?;
    documents::compile_reference(&state.pool, &state.data_dir, &state.fonts_dir, session_id, "cv").await
}

/// Recompile la lettre de motivation de référence courante pour aperçu.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn compile_lettre_motivation(ctx: tauri::State<'_, AppContext>, session_id: i64) -> Result<String, AppError> {
    let state = ctx.state()?;
    documents::compile_reference(&state.pool, &state.data_dir, &state.fonts_dir, session_id, "lettre_motivation")
        .await
}

/// Indique si les références d'une session sont prêtes (templatisées).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session concernée.
///
/// # Returns
///
/// `DocumentsStatus`.
#[tauri::command]
pub async fn get_documents_status(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
) -> Result<DocumentsStatus, AppError> {
    db::documents::status_for_session(&ctx.state()?.pool, session_id).await
}

/// Indique si un document joint existe sur disque, et sa taille.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `path` - Chemin relatif du document (`cv_path`, `lettre_motivation_path`).
///
/// # Returns
///
/// `DocumentFileInfo`.
#[tauri::command]
pub async fn get_document_info(ctx: tauri::State<'_, AppContext>, path: String) -> Result<DocumentFileInfo, AppError> {
    documents::document_info(&ctx.state()?.data_dir, &path)
}

/// Lit un document joint pour aperçu.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `path` - Chemin relatif du document.
///
/// # Returns
///
/// PDF encodé en base64.
#[tauri::command]
pub async fn get_document_pdf(ctx: tauri::State<'_, AppContext>, path: String) -> Result<String, AppError> {
    documents::read_document_base64(&ctx.state()?.data_dir, &path)
}

/// Exporte un document joint vers un fichier choisi par l'utilisateur.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `doc_path` - Chemin relatif du document.
/// * `save_path` - Fichier de destination.
///
/// # Returns
///
/// Le chemin du fichier écrit.
#[tauri::command]
pub async fn download_document(
    ctx: tauri::State<'_, AppContext>,
    doc_path: String,
    save_path: String,
) -> Result<String, AppError> {
    let bytes = documents::read_document_bytes(&ctx.state()?.data_dir, &doc_path)?;
    write_local_file(&save_path, &bytes)?;
    Ok(save_path)
}

/// Écrit un texte (message de candidature) dans un fichier choisi par l'utilisateur.
///
/// # Arguments
///
/// * `save_path` - Fichier de destination.
/// * `content` - Texte à écrire.
///
/// # Returns
///
/// Le chemin du fichier écrit.
#[tauri::command]
pub fn save_text_file(save_path: String, content: String) -> Result<String, AppError> {
    write_local_file(&save_path, content.as_bytes())?;
    Ok(save_path)
}

/// Écrit des octets dans un fichier local, en créant ses dossiers parents.
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

/// Renvoie le dossier de téléchargements de l'utilisateur.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Chemin du dossier de téléchargements.
#[tauri::command]
pub fn get_downloads_dir(app: tauri::AppHandle) -> Result<String, AppError> {
    let dir = app
        .path()
        .download_dir()
        .map_err(|e| AppError::Config(format!("Dossier de téléchargements introuvable : {e}")))?;
    Ok(dir.to_string_lossy().to_string())
}
