//! Stockage sur disque des documents de référence (CV, lettre de motivation)
//! et des documents générés pour une candidature : sauvegarde versionnée,
//! compilation Typst → PDF, lecture et suppression.
//!
//! Chaque session de candidature a son propre CV et sa propre lettre de
//! motivation : tout est stocké sous `documents/sessions/<session_id>/` dans
//! le dossier de données (chemins construits par `crate::storage_paths`).
//! Chaque upload de référence crée une version immuable sous
//! `<doc_type>/versions/<version_id>/` ; la ligne `documents` de la session
//! pointe vers la version courante (voir `db::documents`).

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use base64::Engine as _;
use sqlx::SqlitePool;

// Importation des modules internes
use crate::app_state::AppState;
use crate::clock;
use crate::db;
use crate::db::document_versions::NewVersion;
use crate::db::transaction::begin_write;
use crate::documents::templatize::{templatize, template_artifacts, write_template_files, ValidatedTemplate};
use crate::documents::typst::{compile_document, compile_to_pdf};
use crate::startup::run_mode::RunMode;
use crate::storage_paths;
use crate::error::AppError;
use crate::models::{DocumentFileInfo, DocumentReferenceView};
use crate::trace_log;

/// Nom du source Typst d'une version de référence.
const REFERENCE_TYP: &str = "reference.typ";
/// Nom du PDF d'une version de référence.
const REFERENCE_PDF: &str = "reference.pdf";

/// Contenu d'une nouvelle version de référence, validé et templatisé.
///
/// # Attributes
///
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `filename` - Nom du fichier uploadé.
/// * `content` - Source Typst uploadé.
/// * `pdf` - Octets du PDF compilé.
/// * `validated` - Template validé.
struct PendingVersion<'a> {
    session_id: i64,
    doc_type: &'a str,
    filename: &'a str,
    content: &'a str,
    pdf: &'a [u8],
    validated: &'a ValidatedTemplate,
}

/// Lit un fichier du dossier de données et l'encode en base64.
///
/// # Arguments
///
/// * `path` - Chemin absolu du fichier.
///
/// # Returns
///
/// Contenu encodé en base64.
pub(crate) fn read_base64(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le document {} : {e}", path.display())))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// Récupère la référence courante d'un type de document pour une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si une référence est définie et que son
/// fichier existe sur disque, `None` sinon.
pub async fn get_reference(
    pool: &SqlitePool,
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
) -> Result<Option<DocumentReferenceView>, AppError> {
    let doc = db::documents::get_current(pool, session_id, doc_type).await?;
    Ok(doc
        .filter(|d| storage_paths::to_absolute(data_dir, &d.file_path).exists())
        .map(DocumentReferenceView::from))
}

/// Écrit `reference.typ` et `reference.pdf` d'une version.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `version_dir` - Dossier relatif de la version.
/// * `content` - Source Typst uploadé.
/// * `pdf` - Octets du PDF compilé.
///
/// # Returns
///
/// Chemins relatifs `(typst_path, pdf_path)`.
fn write_version_files(
    data_dir: &Path,
    version_dir: &str,
    content: &str,
    pdf: &[u8],
) -> Result<(String, String), AppError> {
    let dir = storage_paths::to_absolute(data_dir, version_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier {} : {e}", dir.display())))?;

    let typst_path = storage_paths::file_in(version_dir, REFERENCE_TYP);
    let pdf_path = storage_paths::file_in(version_dir, REFERENCE_PDF);
    for (relative, bytes) in [(&typst_path, content.as_bytes()), (&pdf_path, pdf)] {
        let path = storage_paths::to_absolute(data_dir, relative);
        std::fs::write(&path, bytes)
            .map_err(|e| AppError::Validation(format!("Écriture de {} impossible : {e}", path.display())))?;
    }
    Ok((typst_path, pdf_path))
}

/// Supprime le dossier d'une version qui n'a pas pu être enregistrée, pour
/// ne pas laisser de fichiers orphelins. Un échec de suppression est
/// seulement tracé : l'erreur d'origine reste celle remontée à l'appelant.
///
/// # Arguments
///
/// * `dir` - Dossier absolu de la version.
fn discard_version_dir(dir: &Path) {
    match std::fs::remove_dir_all(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => trace_log::log(&format!(
            "Suppression du dossier de version orphelin {} impossible : {e}",
            dir.display()
        )),
    }
}

/// Enregistre une version de référence et en fait la référence courante.
///
/// La ligne de version est insérée dans une transaction d'écriture avant
/// l'écriture de ses fichiers ; la transaction n'est validée qu'une fois les
/// fichiers en place et le pointeur mis à jour. En cas d'échec, la
/// transaction est annulée et le dossier de la version supprimé.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `pending` - Version à enregistrer.
///
/// # Returns
///
/// `DocumentReferenceView` de la nouvelle référence courante.
async fn persist_version(
    pool: &SqlitePool,
    data_dir: &Path,
    pending: &PendingVersion<'_>,
) -> Result<DocumentReferenceView, AppError> {
    let now = clock::now_utc();
    let mut tx = begin_write(pool).await?;

    // ------------------------------------------------------------------
    // 1. Identifiant (nomme le dossier), chemins et ligne de version
    // ------------------------------------------------------------------
    let version_id = db::document_versions::next_id(&mut tx).await?;
    let version_dir = storage_paths::version_dir(pending.session_id, pending.doc_type, version_id);
    let template = template_artifacts(&version_dir, pending.validated)?;
    let typst_path = storage_paths::file_in(&version_dir, REFERENCE_TYP);
    let pdf_path = storage_paths::file_in(&version_dir, REFERENCE_PDF);
    let new = NewVersion {
        id: version_id,
        session_id: pending.session_id,
        doc_type: pending.doc_type,
        original_filename: pending.filename,
        typst_path: &typst_path,
        pdf_path: &pdf_path,
        template: &template,
    };
    db::document_versions::insert_version(&mut tx, &new, &now).await?;

    // ------------------------------------------------------------------
    // 2. Fichiers, pointeur, validation — tout ou rien
    // ------------------------------------------------------------------
    let absolute_dir = storage_paths::to_absolute(data_dir, &version_dir);
    let (version_dir, template, now) = (version_dir.as_str(), &template, now.as_str());
    let written = async move {
        write_version_files(data_dir, version_dir, pending.content, pending.pdf)?;
        write_template_files(data_dir, template, pending.validated)?;
        db::documents::set_current(&mut tx, pending.session_id, pending.doc_type, version_id, now).await?;
        let doc = db::documents::get_current(&mut *tx, pending.session_id, pending.doc_type)
            .await?
            .ok_or_else(|| AppError::Database("Référence courante absente après enregistrement.".to_string()))?;
        tx.commit().await.map_err(db::common::map_db_err)?;
        Ok(DocumentReferenceView::from(doc))
    }
    .await;

    // Transaction abandonnée (donc annulée) sur erreur : seul le dossier
    // reste à nettoyer.
    written.inspect_err(|_| discard_version_dir(&absolute_dir))
}

/// Compile le document, le convertit en template (appel LLM validé par
/// compilation, voir `documents::templatize`), puis l'enregistre comme
/// nouvelle version de référence et en fait la référence courante de la
/// session. Les versions précédentes sont conservées.
///
/// La compilation et la templatisation ont lieu en mémoire, avant toute
/// écriture : si l'une échoue, rien n'est modifié (ni fichier, ni base).
///
/// # Arguments
///
/// * `state` - État de l'application (base, dossiers de données et de
///   polices, clé API pour la templatisation).
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `filename` - Nom du fichier fourni par l'utilisateur (doit finir par `.typ`).
/// * `content` - Contenu Typst du document.
///
/// # Returns
///
/// `DocumentReferenceView` pointant vers les fichiers `.typ` et `.pdf` sauvegardés.
pub async fn save_reference(
    state: &AppState,
    session_id: i64,
    doc_type: &'static str,
    filename: String,
    content: String,
) -> Result<DocumentReferenceView, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation
    // ------------------------------------------------------------------
    if !filename.to_lowercase().ends_with(".typ") {
        return Err(AppError::Validation("Seuls les fichiers .typ sont acceptés.".to_string()));
    }
    let (pool, data_dir, fonts_dir) = (&state.pool, state.data_dir.as_path(), state.fonts_dir.as_path());

    // ------------------------------------------------------------------
    // 2. Compilation en PDF, en mémoire (racine = dossier du type de
    //    document, comme avant le versioning)
    // ------------------------------------------------------------------
    let doc_dir = storage_paths::to_absolute(data_dir, &storage_paths::doc_type_dir(session_id, doc_type));
    std::fs::create_dir_all(&doc_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier {} : {e}", doc_dir.display())))?;

    let reference = {
        let (to_compile, root, fonts) = (content.clone(), doc_dir.clone(), fonts_dir.to_path_buf());
        tokio::task::spawn_blocking(move || compile_document(to_compile, root, fonts))
            .await
            .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
            .map_err(AppError::Validation)?
    };

    // ------------------------------------------------------------------
    // 3. Conversion en template via l'API Claude, validée par compilation
    //    — hors transaction : les appels LLM ne bloquent pas les écritures
    // ------------------------------------------------------------------
    let target = state.claude().resolve().await?;
    // En DEV seulement, les sorties rejetées sont conservées pour le debug.
    let debug_dir = (state.run_mode == RunMode::Dev).then(|| {
        storage_paths::to_absolute(
            data_dir,
            &storage_paths::templatize_debug_dir(session_id, doc_type, &clock::file_stamp()),
        )
    });
    let validated =
        templatize(&content, &reference, doc_type, &doc_dir, fonts_dir, &target, debug_dir.as_deref()).await?;

    // ------------------------------------------------------------------
    // 4. Version en base, fichiers et référence courante, atomiquement
    // ------------------------------------------------------------------
    let pending = PendingVersion {
        session_id,
        doc_type,
        filename: &filename,
        content: &content,
        pdf: &reference.pdf,
        validated: &validated,
    };
    persist_version(pool, data_dir, &pending).await
}

/// Compile la référence courante d'un type de document et retourne le PDF encodé en base64.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `fonts_dir` - Dossier des polices importées.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// PDF encodé en base64, prêt à être affiché côté frontend.
pub async fn compile_reference(
    pool: &SqlitePool,
    data_dir: &Path,
    fonts_dir: &Path,
    session_id: i64,
    doc_type: &'static str,
) -> Result<String, AppError> {
    let reference = get_reference(pool, data_dir, session_id, doc_type)
        .await?
        .ok_or_else(|| AppError::NotFound("Aucun fichier de référence trouvé.".to_string()))?;
    let typ_path = storage_paths::to_absolute(data_dir, &reference.file_path);

    let content = std::fs::read_to_string(&typ_path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le fichier : {}", e)))?;
    let root = typ_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::Validation(format!("Chemin de référence invalide : {}", reference.file_path)))?;
    let fonts_dir = fonts_dir.to_path_buf();

    let pdf_bytes = tokio::task::spawn_blocking(move || compile_to_pdf(content, root, fonts_dir))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

/// Inspecte l'état sur disque d'un document joint à une candidature (CV ou
/// lettre de motivation), pour affichage honnête dans la page de détail —
/// le fichier peut avoir été déplacé ou supprimé après coup.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `path` - Chemin relatif reçu du frontend (`cv_path` ou `lettre_motivation_path`).
///
/// # Returns
///
/// `DocumentFileInfo` avec `exists = false` et `size = 0` si le fichier est
/// introuvable, `AppError::Validation` si le chemin sort du dossier de données.
pub fn document_info(data_dir: &Path, path: &str) -> Result<DocumentFileInfo, AppError> {
    let resolved = storage_paths::resolve_untrusted(data_dir, path)?;
    Ok(match std::fs::metadata(resolved) {
        Ok(meta) => DocumentFileInfo { exists: true, size: meta.len() as i64 },
        Err(_) => DocumentFileInfo { exists: false, size: 0 },
    })
}

/// Lit un document PDF joint à une candidature et l'encode en base64, pour
/// affichage dans un aperçu (iframe) côté frontend.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `path` - Chemin relatif reçu du frontend.
///
/// # Returns
///
/// Contenu du PDF encodé en base64, ou `AppError::Validation` si le chemin
/// sort du dossier de données ou si le fichier est illisible.
pub fn read_document_base64(data_dir: &Path, path: &str) -> Result<String, AppError> {
    read_base64(&storage_paths::resolve_untrusted(data_dir, path)?)
}

/// Lit les octets bruts d'un document PDF joint à une candidature, pour
/// l'exporter vers un fichier choisi par l'utilisateur.
///
/// Le chemin vient du frontend : il doit désigner un `.pdf` du dossier de
/// données. Tout refus renvoie `NotFound`, sans distinguer « absent » de
/// « interdit ».
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `path` - Chemin relatif reçu du frontend.
///
/// # Returns
///
/// Octets du PDF.
pub fn read_document_bytes(data_dir: &Path, path: &str) -> Result<Vec<u8>, AppError> {
    let not_found = || AppError::NotFound(format!("Document introuvable : {path}"));

    let resolved = storage_paths::resolve_untrusted(data_dir, path).map_err(|_| not_found())?;
    let is_pdf = resolved.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"));
    if !is_pdf || !resolved.is_file() {
        return Err(not_found());
    }
    std::fs::read(&resolved).map_err(|e| AppError::Validation(format!("Impossible de lire le document {path} : {e}")))
}

/// Sauvegarde un document généré par le pipeline IA (`crate::pipeline`) à
/// l'emplacement définitif d'une candidature : le PDF, et à côté les
/// données passées au template (`artefact/<candidature_id>.{pdf,data.json}`).
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `candidature_id` - Identifiant de la candidature.
/// * `pdf` - Octets du PDF compilé.
/// * `data_json` - Données passées au template.
///
/// # Returns
///
/// Chemin relatif du PDF sauvegardé.
pub fn save_generated_document(
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
    candidature_id: i64,
    pdf: &[u8],
    data_json: &str,
) -> Result<String, AppError> {
    let data_path = storage_paths::artefact_file(session_id, doc_type, candidature_id, "data.json");
    let pdf_path = storage_paths::artefact_file(session_id, doc_type, candidature_id, "pdf");
    let artefact_dir = storage_paths::to_absolute(data_dir, &pdf_path);
    if let Some(dir) = artefact_dir.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier {} : {e}", dir.display())))?;
    }

    for (relative, bytes) in [(&data_path, data_json.as_bytes()), (&pdf_path, pdf)] {
        let path = storage_paths::to_absolute(data_dir, relative);
        std::fs::write(&path, bytes)
            .map_err(|e| AppError::Validation(format!("Écriture de {} impossible : {e}", path.display())))?;
    }
    Ok(pdf_path)
}

/// Écrit les fichiers de trace d'une génération (analyse d'offre, matching,
/// contenus rédigés, rapports du linter, essais) dans son dossier de debug.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `candidature_id` - Identifiant de la candidature.
/// * `files` - Fichiers à écrire : nom et contenu.
///
/// # Returns
///
/// Chemin absolu du dossier de debug.
pub fn save_generation_trace(
    data_dir: &Path,
    session_id: i64,
    candidature_id: i64,
    files: &[(&str, String)],
) -> Result<PathBuf, AppError> {
    let dir = storage_paths::to_absolute(data_dir, &storage_paths::debug_dir(session_id, candidature_id));
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer {} : {e}", dir.display())))?;
    for (name, content) in files {
        let path = dir.join(name);
        std::fs::write(&path, content.as_bytes())
            .map_err(|e| AppError::Validation(format!("Écriture de {} impossible : {e}", path.display())))?;
    }
    Ok(dir)
}

/// Supprime un fichier, sans erreur s'il est déjà absent.
fn remove_if_exists(path: &Path) -> Result<(), AppError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Validation(format!("Suppression de {} impossible : {e}", path.display()))),
    }
}

/// Supprime un dossier et son contenu, sans erreur s'il est déjà absent.
fn remove_dir_if_exists(path: &Path) -> Result<(), AppError> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Validation(format!("Suppression de {} impossible : {e}", path.display()))),
    }
}

/// Supprime les documents générés d'une candidature (CV et lettre de
/// motivation : `.pdf`, `.data.json`, et `.typ` des générations antérieures
/// aux templates) ainsi que son dossier de debug. Un fichier déjà absent
/// n'est pas une erreur : la génération a pu échouer avant l'écriture.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` si tous les fichiers existants ont été supprimés, sinon l'erreur
/// du premier fichier impossible à supprimer.
pub fn delete_generated_documents(data_dir: &Path, session_id: i64, candidature_id: i64) -> Result<(), AppError> {
    for doc_type in ["cv", "lettre_motivation"] {
        for extension in ["pdf", "data.json", "typ"] {
            let file = storage_paths::artefact_file(session_id, doc_type, candidature_id, extension);
            remove_if_exists(&storage_paths::to_absolute(data_dir, &file))?;
        }
    }
    remove_dir_if_exists(&storage_paths::to_absolute(data_dir, &storage_paths::debug_dir(session_id, candidature_id)))
}

/// Supprime tous les fichiers d'une session (références, versions,
/// documents générés, traces).
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session supprimée.
///
/// # Returns
///
/// `Ok(())` si le dossier a été supprimé ou n'existait pas.
pub fn delete_session_documents(data_dir: &Path, session_id: i64) -> Result<(), AppError> {
    remove_dir_if_exists(&storage_paths::to_absolute(data_dir, &storage_paths::session_dir(session_id)))
}
