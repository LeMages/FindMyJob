//! Résolution de l'AppHandle (dossier de données, dossier de téléchargements)
//! et émission des événements de progression Tauri, pour déléguer la
//! logique de stockage des documents à `findmyjob_core::documents`.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use tauri::{Emitter, Manager};

// Importation des modules internes
use crate::db::init::{resolve_data_dir, DbState};
use crate::documents::fonts::resolve_fonts_dir;
use crate::error::AppError;
use crate::models::{CandidatureDetail, DocumentFileInfo, DocumentReferenceView, DocumentSaveProgress};
use findmyjob_core::documents as core_docs;

/// Nom de l'événement Tauri émis à chaque étape de `save_reference`, écouté
/// par le frontend pour afficher une barre de progression.
const SAVE_PROGRESS_EVENT: &str = "document-save-progress";

/// Émet la progression courante de `save_reference` vers le frontend.
///
/// Best-effort : un échec d'émission (aucune fenêtre active, par exemple)
/// n'a aucun impact sur la sauvegarde elle-même, purement cosmétique.
fn emit_save_progress(app: &tauri::AppHandle, session_id: i64, doc_type: &str, step: u32, label: &str) {
    let _ = app.emit(
        SAVE_PROGRESS_EVENT,
        DocumentSaveProgress {
            session_id,
            doc_type: doc_type.to_string(),
            step,
            total: core_docs::SAVE_STEPS_TOTAL,
            label: label.to_string(),
        },
    );
}

/// Récupère la référence de document existante.
pub async fn get_reference(app: &tauri::AppHandle, session_id: i64, doc_type: &str) -> Option<DocumentReferenceView> {
    let pool = app.state::<DbState>().pool.clone();
    core_docs::get_reference(&pool, &resolve_data_dir(app), session_id, doc_type).await
}

/// Analyse le style, sauvegarde le nouveau document de référence, et le compile en PDF.
pub async fn save_reference(
    app: tauri::AppHandle,
    session_id: i64,
    doc_type: &'static str,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    let pool = app.state::<DbState>().pool.clone();
    let data_dir = resolve_data_dir(&app);
    let fonts_dir = resolve_fonts_dir(&app);
    let app_for_progress = app.clone();
    core_docs::save_reference(
        &pool,
        &data_dir,
        &fonts_dir,
        session_id,
        doc_type,
        filename,
        content,
        move |step, label| emit_save_progress(&app_for_progress, session_id, doc_type, step, label),
    )
    .await
}

/// Compile la référence sauvegardée et retourne le PDF encodé en base64.
pub async fn compile_reference(
    app: tauri::AppHandle,
    session_id: i64,
    doc_type: &'static str,
) -> Result<String, AppError> {
    core_docs::compile_reference(&resolve_data_dir(&app), &resolve_fonts_dir(&app), session_id, doc_type).await
}

/// Inspecte l'état sur disque d'un document joint à une candidature.
pub fn document_info(path: &str) -> DocumentFileInfo {
    core_docs::document_info(path)
}

/// Lit un document PDF joint à une candidature et l'encode en base64.
pub fn read_document_base64(path: &str) -> Result<String, AppError> {
    core_docs::read_document_base64(path)
}

/// Finalise un document mis en scène en le déplaçant vers son emplacement définitif.
pub fn finalize_candidature_document(
    app: &tauri::AppHandle,
    session_id: i64,
    doc_type: &str,
    staged_path: Option<String>,
    candidature_id: i64,
) -> Result<Option<String>, AppError> {
    core_docs::finalize_candidature_document(&resolve_data_dir(app), session_id, doc_type, staged_path, candidature_id)
}

/// Sauvegarde un document généré par le pipeline IA à l'emplacement définitif d'une candidature.
pub fn save_generated_document(
    app: &tauri::AppHandle,
    session_id: i64,
    doc_type: &str,
    candidature_id: i64,
    typst: &str,
    pdf: &[u8],
) -> Result<String, AppError> {
    core_docs::save_generated_document(&resolve_data_dir(app), session_id, doc_type, candidature_id, typst, pdf)
}

/// Copie le CV et la lettre de motivation d'une candidature vers un
/// sous-dossier `<poste>_<entreprise>_<id>` du dossier de téléchargements de
/// l'utilisateur — logique commune aux trois façons de postuler.
fn export_documents(app: &tauri::AppHandle, detail: &CandidatureDetail) -> Result<PathBuf, AppError> {
    let download_dir = app
        .path()
        .download_dir()
        .map_err(|e| AppError::Validation(format!("Dossier de téléchargements introuvable : {}", e)))?;
    let dest_dir = download_dir.join(core_docs::build_export_dir_name(detail));
    core_docs::copy_candidature_documents(&dest_dir, detail)?;
    Ok(dest_dir)
}

/// Exporte le CV et la lettre de motivation d'une candidature vers le dossier
/// de téléchargements de l'utilisateur — utilisé par « Postuler sans message ».
pub fn export_for_apply(app: &tauri::AppHandle, detail: &CandidatureDetail) -> Result<(), AppError> {
    export_documents(app, detail)?;
    Ok(())
}

/// Exporte le CV, la lettre de motivation et un message de candidature rédigé
/// par l'IA vers le dossier de téléchargements de l'utilisateur — utilisé par
/// « Postuler par mail » et « Postuler par message ».
pub fn export_for_apply_with_message(
    app: &tauri::AppHandle,
    detail: &CandidatureDetail,
    message: &str,
    message_filename: &str,
) -> Result<(), AppError> {
    let dest_dir = export_documents(app, detail)?;
    core_docs::write_export_message(&dest_dir, message, message_filename)
}
