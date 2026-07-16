//! Stockage sur disque des documents de référence (CV, lettre de motivation)
//! et des documents joints à une candidature : résolution de chemins,
//! sauvegarde avec historique, compilation Typst → PDF, mise en scène des
//! fichiers avant rattachement définitif à une candidature.
//!
//! Chaque session de candidature a son propre CV et sa propre lettre de
//! motivation : tout est stocké sous `documents/sessions/<session_id>/`.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use crate::db::resolve_data_dir;
use crate::documents::typst::compile_to_pdf;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView};

/// Résout le dossier d'un type de document (`cv` ou `lettre_motivation`)
/// propre à une session de candidature.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire des documents.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"` — le nom du dossier correspond exactement.
///
/// # Returns
///
/// Chemin absolu vers `documents/sessions/<session_id>/<doc_type>/`.
fn resolve_doc_dir(app: &tauri::AppHandle, session_id: i64, doc_type: &str) -> PathBuf {
    resolve_data_dir(app)
        .join("documents")
        .join("sessions")
        .join(session_id.to_string())
        .join(doc_type)
}

/// Nom de base utilisé pour les fichiers de sauvegarde (`cv.typ`/`lettre.typ`).
fn doc_backup_basename(doc_type: &str) -> &'static str {
    if doc_type == "lettre_motivation" { "lettre" } else { "cv" }
}

/// Formate la date de dernière modification d'un fichier en millisecondes epoch.
///
/// # Arguments
///
/// * `path` - Chemin du fichier à inspecter.
///
/// # Returns
///
/// Timestamp en millisecondes sous forme de chaîne, ou chaîne vide si indisponible.
fn updated_at_millis(path: &std::path::Path) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string()
        })
        .unwrap_or_default()
}

/// Récupère la référence de document existante (si un fichier `reference.typ` a déjà été sauvegardé).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si un fichier de référence existe, `None` sinon.
pub fn get_reference(app: &tauri::AppHandle, session_id: i64, doc_type: &str) -> Option<DocumentReferenceView> {
    let doc_dir = resolve_doc_dir(app, session_id, doc_type);
    let typ_path = doc_dir.join("reference.typ");
    if !typ_path.exists() {
        return None;
    }
    let pdf_path = doc_dir.join("reference.pdf");
    Some(DocumentReferenceView {
        file_path: typ_path.to_string_lossy().to_string(),
        pdf_path: pdf_path.to_string_lossy().to_string(),
        updated_at: updated_at_millis(&typ_path),
    })
}

/// Sauvegarde un nouveau document de référence (.typ), en archivant l'ancien s'il existe,
/// puis le compile immédiatement en PDF.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `filename` - Nom du fichier fourni par l'utilisateur (doit finir par `.typ`).
/// * `content` - Contenu Typst du document.
///
/// # Returns
///
/// `DocumentReferenceView` pointant vers les fichiers `.typ` et `.pdf` sauvegardés.
pub async fn save_reference(
    app: tauri::AppHandle,
    session_id: i64,
    doc_type: &'static str,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation
    // ------------------------------------------------------------------
    if !filename.to_lowercase().ends_with(".typ") {
        return Err(AppError::Validation(
            "Seuls les fichiers .typ sont acceptés.".to_string(),
        ));
    }

    let doc_dir = resolve_doc_dir(&app, session_id, doc_type);
    std::fs::create_dir_all(&doc_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let typ_path = doc_dir.join("reference.typ");
    let pdf_path = doc_dir.join("reference.pdf");

    // ------------------------------------------------------------------
    // 2. Sauvegarde de l'ancienne référence (si elle existe) dans backup/<timestamp>/
    // ------------------------------------------------------------------
    if typ_path.exists() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_dir = doc_dir.join("backup").join(ts.to_string());
        std::fs::create_dir_all(&backup_dir).map_err(|e| {
            AppError::Validation(format!("Impossible de créer le dossier de sauvegarde : {}", e))
        })?;
        let basename = doc_backup_basename(doc_type);
        std::fs::copy(&typ_path, backup_dir.join(format!("{}.typ", basename))).map_err(|e| {
            AppError::Validation(format!("Sauvegarde de l'ancien fichier échouée : {}", e))
        })?;
        if pdf_path.exists() {
            std::fs::copy(&pdf_path, backup_dir.join(format!("{}.pdf", basename))).ok();
        }
    }

    // ------------------------------------------------------------------
    // 3. Écriture du nouveau fichier .typ
    // ------------------------------------------------------------------
    std::fs::write(&typ_path, content.as_bytes())
        .map_err(|e| AppError::Validation(format!("Écriture du fichier impossible : {}", e)))?;

    // ------------------------------------------------------------------
    // 4. Compilation en PDF
    // ------------------------------------------------------------------
    let root = doc_dir.clone();
    let pdf_bytes = tauri::async_runtime::spawn_blocking(move || compile_to_pdf(content, root))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    std::fs::write(&pdf_path, &pdf_bytes)
        .map_err(|e| AppError::Validation(format!("Écriture du PDF impossible : {}", e)))?;

    Ok(DocumentReferenceView {
        file_path: typ_path.to_string_lossy().to_string(),
        pdf_path: pdf_path.to_string_lossy().to_string(),
        updated_at: updated_at_millis(&typ_path),
    })
}

/// Compile la référence sauvegardée d'un type de document et retourne le PDF encodé en base64.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// PDF encodé en base64, prêt à être affiché côté frontend.
pub async fn compile_reference(
    app: tauri::AppHandle,
    session_id: i64,
    doc_type: &'static str,
) -> Result<String, AppError> {
    let doc_dir = resolve_doc_dir(&app, session_id, doc_type);
    let typ_path = doc_dir.join("reference.typ");
    if !typ_path.exists() {
        return Err(AppError::NotFound("Aucun fichier de référence trouvé.".to_string()));
    }

    let content = std::fs::read_to_string(&typ_path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le fichier : {}", e)))?;
    let root = doc_dir;

    let pdf_bytes = tauri::async_runtime::spawn_blocking(move || compile_to_pdf(content, root))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

/// Résout (et crée si besoin) le dossier de référence d'un type de document,
/// utilisé comme dossier par défaut dans le sélecteur de fichiers du frontend.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Chemin absolu du dossier, sous forme de chaîne.
pub fn resolve_reference_dir(
    app: &tauri::AppHandle,
    session_id: i64,
    doc_type: &str,
) -> Result<String, AppError> {
    let dir = resolve_doc_dir(app, session_id, doc_type);
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;
    Ok(dir.to_string_lossy().to_string())
}

/// Construit un chemin de mise en scène unique pour un document en attente
/// de rattachement à une candidature (`artefact/pending_<timestamp>_<filename>`).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `filename` - Nom de fichier d'origine (utilisé comme suffixe).
///
/// # Returns
///
/// Chemin de destination temporaire.
fn next_staged_path(
    app: &tauri::AppHandle,
    session_id: i64,
    doc_type: &str,
    filename: &str,
) -> Result<PathBuf, AppError> {
    let artefact_dir = resolve_doc_dir(app, session_id, doc_type).join("artefact");
    std::fs::create_dir_all(&artefact_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    Ok(artefact_dir.join(format!("pending_{}_{}", ts, filename)))
}

/// Met en scène un document envoyé en base64 (glisser-déposer ou sélection navigateur)
/// avant rattachement définitif à une candidature.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `filename` - Nom du fichier d'origine (doit finir par `.pdf`).
/// * `content` - Contenu du fichier encodé en base64.
///
/// # Returns
///
/// Chemin temporaire du fichier mis en scène.
pub async fn upload_candidature_document(
    app: tauri::AppHandle,
    session_id: i64,
    doc_type: &'static str,
    filename: String,
    content: String,
) -> Result<String, AppError> {
    if !filename.to_lowercase().ends_with(".pdf") {
        return Err(AppError::Validation(
            "Seuls les fichiers PDF sont acceptés.".to_string(),
        ));
    }

    use base64::Engine as _;
    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(&content)
        .map_err(|e| AppError::Validation(format!("Contenu invalide : {}", e)))?;

    let dest = next_staged_path(&app, session_id, doc_type, &filename)?;
    std::fs::write(&dest, &pdf_bytes)
        .map_err(|e| AppError::Validation(format!("Écriture impossible : {}", e)))?;
    Ok(dest.to_string_lossy().to_string())
}

/// Met en scène un document sélectionné via l'explorateur de fichiers natif
/// (copie vers l'espace de mise en scène de l'application).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `path` - Chemin absolu du fichier sélectionné (doit finir par `.pdf`).
///
/// # Returns
///
/// Chemin temporaire du fichier mis en scène.
pub async fn select_candidature_document(
    app: tauri::AppHandle,
    session_id: i64,
    doc_type: &'static str,
    path: String,
) -> Result<String, AppError> {
    if !path.to_lowercase().ends_with(".pdf") {
        return Err(AppError::Validation(
            "Seuls les fichiers PDF sont acceptés.".to_string(),
        ));
    }

    let filename = std::path::Path::new(&path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "document.pdf".to_string());

    let dest = next_staged_path(&app, session_id, doc_type, &filename)?;
    std::fs::copy(&path, &dest)
        .map_err(|e| AppError::Validation(format!("Copie du fichier impossible : {}", e)))?;
    Ok(dest.to_string_lossy().to_string())
}

/// Inspecte l'état sur disque d'un document joint à une candidature (CV ou
/// lettre de motivation), pour affichage honnête dans la page de détail —
/// le chemin enregistré en base peut pointer vers un fichier déplacé ou supprimé.
///
/// # Arguments
///
/// * `path` - Chemin absolu du document (`candidatures.cv_path` ou `lettre_motivation_path`).
///
/// # Returns
///
/// `DocumentFileInfo` avec `exists = false` et `size = 0` si le fichier est introuvable.
pub fn document_info(path: &str) -> DocumentFileInfo {
    match std::fs::metadata(path) {
        Ok(meta) => DocumentFileInfo {
            exists: true,
            size: meta.len() as i64,
        },
        Err(_) => DocumentFileInfo {
            exists: false,
            size: 0,
        },
    }
}

/// Lit un document PDF joint à une candidature et l'encode en base64, pour
/// affichage dans un aperçu (iframe) côté frontend — même mécanique que
/// `compile_reference`, sans étape de compilation puisque le fichier est déjà
/// un PDF finalisé.
///
/// # Arguments
///
/// * `path` - Chemin absolu du document PDF.
///
/// # Returns
///
/// Contenu du PDF encodé en base64.
pub fn read_document_base64(path: &str) -> Result<String, AppError> {
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le document : {}", e)))?;

    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// Finalise un document mis en scène en le déplaçant vers son emplacement
/// définitif, nommé d'après l'identifiant de la candidature.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `staged_path` - Chemin temporaire renvoyé par `upload_candidature_document`
///   ou `select_candidature_document`, s'il y en a un.
/// * `candidature_id` - Identifiant de la candidature à laquelle rattacher le document.
///
/// # Returns
///
/// `Some(chemin_final)` si un document était en attente, `None` sinon.
pub fn finalize_candidature_document(
    app: &tauri::AppHandle,
    session_id: i64,
    doc_type: &str,
    staged_path: Option<String>,
    candidature_id: i64,
) -> Result<Option<String>, AppError> {
    let Some(staged) = staged_path else {
        return Ok(None);
    };
    let staged_pb = PathBuf::from(&staged);
    if !staged_pb.exists() {
        return Err(AppError::Validation(
            "Le document sélectionné est introuvable.".to_string(),
        ));
    }
    let artefact_dir = resolve_doc_dir(app, session_id, doc_type).join("artefact");
    std::fs::create_dir_all(&artefact_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;
    let dest = artefact_dir.join(format!("{}.pdf", candidature_id));
    std::fs::rename(&staged_pb, &dest)
        .map_err(|e| AppError::Validation(format!("Impossible de finaliser le document : {}", e)))?;
    Ok(Some(dest.to_string_lossy().to_string()))
}
