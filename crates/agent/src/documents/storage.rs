//! Stockage sur disque des documents de référence (CV, lettre de motivation)
//! et des documents joints à une candidature : résolution de chemins,
//! sauvegarde versionnée, compilation Typst → PDF, mise en scène des
//! fichiers avant rattachement définitif à une candidature.
//!
//! Chaque session de candidature a son propre CV et sa propre lettre de
//! motivation : tout est stocké sous `documents/sessions/<session_id>/`.
//! Chaque upload de référence crée une version immuable sous
//! `<doc_type>/versions/<version_id>/reference.{typ,pdf}` ; la ligne
//! `documents` de la session pointe vers la version courante (voir
//! `db::document_versions`).
//!
//! La résolution de chemins liée au dossier de données applicatif (et
//! l'émission d'événements de progression pour un éventuel appelant
//! interactif) vit chez l'appelant ; ce module reçoit ces chemins déjà résolus.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use sqlx::PgPool;

// Importation des modules internes
use crate::db;
use crate::documents::typst::compile_to_pdf;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{DocumentFileInfo, DocumentReferenceView};

/// Nombre total d'étapes du flux de `save_reference`, communiqué à
/// l'appelant pour calculer une progression (`step`/`total`).
///
/// Non utilisé par l'API HTTP de l'étape 3d (pas de canal de progression
/// exposé, voir `api::documents::save_cv`) — conservé pour un futur appelant
/// interactif (desktop).
#[allow(dead_code)]
pub const SAVE_STEPS_TOTAL: u32 = 5;

/// Résout le dossier d'un type de document (`cv` ou `lettre_motivation`)
/// propre à une session de candidature.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire des documents.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"` — le nom du dossier correspond exactement.
///
/// # Returns
///
/// Chemin absolu vers `documents/sessions/<session_id>/<doc_type>/`.
fn resolve_doc_dir(data_dir: &Path, session_id: i64, doc_type: &str) -> PathBuf {
    data_dir
        .join("documents")
        .join("sessions")
        .join(session_id.to_string())
        .join(doc_type)
}

/// Écrit les fichiers `.typ` et `.pdf` d'une nouvelle version de référence
/// dans son dossier dédié (créé au passage).
///
/// # Arguments
///
/// * `version_dir` - Dossier `versions/<version_id>/` de la version.
/// * `content` - Contenu Typst uploadé.
/// * `pdf` - Octets du PDF compilé.
///
/// # Returns
///
/// Chemins absolus `(typst_path, pdf_path)` des fichiers écrits.
fn write_version_files(version_dir: &Path, content: &str, pdf: &[u8]) -> Result<(String, String), AppError> {
    std::fs::create_dir_all(version_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier de version : {}", e)))?;

    let typ_path = version_dir.join("reference.typ");
    std::fs::write(&typ_path, content.as_bytes())
        .map_err(|e| AppError::Validation(format!("Écriture du fichier impossible : {}", e)))?;

    let pdf_path = version_dir.join("reference.pdf");
    std::fs::write(&pdf_path, pdf)
        .map_err(|e| AppError::Validation(format!("Écriture du PDF impossible : {}", e)))?;

    Ok((typ_path.to_string_lossy().to_string(), pdf_path.to_string_lossy().to_string()))
}

/// Supprime le dossier d'une version qui n'a pas pu être enregistrée en base,
/// pour ne pas laisser de fichiers orphelins. Un échec de suppression est
/// seulement loggé : l'erreur d'origine reste celle remontée à l'appelant.
///
/// # Arguments
///
/// * `version_dir` - Dossier `versions/<version_id>/` à supprimer.
fn discard_version_dir(version_dir: &Path) {
    if let Err(e) = std::fs::remove_dir_all(version_dir) {
        eprintln!("Suppression du dossier de version orphelin {} impossible : {e}", version_dir.display());
    }
}

/// Récupère la référence courante d'un type de document pour une session,
/// d'après sa ligne `documents` (source de vérité du chemin, qui pointe vers
/// la version courante — ou vers `reference.typ` pour une référence
/// antérieure au versioning).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si une référence est trackée et que son
/// fichier existe sur disque, `None` sinon.
pub async fn get_reference(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
) -> Result<Option<DocumentReferenceView>, AppError> {
    let doc = db::documents::get_by_session_and_type(pool, session_id, doc_type).await?;
    Ok(doc
        .filter(|d| Path::new(&d.file_path).exists())
        .map(DocumentReferenceView::from))
}

/// Analyse le style du document via l'API Claude, le compile en PDF, puis
/// l'enregistre comme nouvelle version de référence (fichiers sous
/// `versions/<version_id>/` + ligne `document_versions`) et en fait la
/// référence courante de la session. Les versions précédentes sont conservées.
///
/// L'analyse de style et la compilation ont lieu avant toute écriture
/// disque : si l'une échoue, rien n'est modifié (ni fichier, ni entrée DB).
/// Si l'enregistrement en base de la version échoue, ses fichiers sont
/// supprimés.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `data_dir` - Dossier de données de l'application, déjà résolu par l'appelant.
/// * `fonts_dir` - Dossier des polices importées, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `filename` - Nom du fichier fourni par l'utilisateur (doit finir par `.typ`).
/// * `content` - Contenu Typst du document.
/// * `on_progress` - Callback appelée à chaque étape (`step` 1-based, libellé), pour reporter la progression à l'appelant.
///
/// # Returns
///
/// `DocumentReferenceView` pointant vers les fichiers `.typ` et `.pdf` sauvegardés.
#[allow(clippy::too_many_arguments)]
pub async fn save_reference(
    pool: &PgPool,
    data_dir: &Path,
    fonts_dir: &Path,
    session_id: i64,
    doc_type: &'static str,
    filename: String,
    content: String,
    on_progress: impl Fn(u32, &str),
) -> Result<DocumentReferenceView, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation
    // ------------------------------------------------------------------
    if !filename.to_lowercase().ends_with(".typ") {
        return Err(AppError::Validation(
            "Seuls les fichiers .typ sont acceptés.".to_string(),
        ));
    }

    // ------------------------------------------------------------------
    // 1bis. Analyse de style via l'API Claude — placée avant toute
    //    écriture disque : si l'appel échoue ou renvoie un format
    //    inattendu, rien n'est créé (ni dossier, ni version, ni fichier).
    // ------------------------------------------------------------------
    let model = db::settings::resolve_llm_model(pool, findmyjob_core::llm::client::DEFAULT_MODEL).await?;

    on_progress(1, "Analyse du style (IA)…");
    let style_analysis = findmyjob_core::llm::analyze_style(&content, doc_type, &model).await?;

    // ------------------------------------------------------------------
    // 2. Compilation en PDF, en mémoire (racine = dossier du type de
    //    document, comme les références d'avant le versioning)
    // ------------------------------------------------------------------
    on_progress(2, "Compilation en PDF…");
    let doc_dir = resolve_doc_dir(data_dir, session_id, doc_type);
    std::fs::create_dir_all(&doc_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let root = doc_dir.clone();
    let fonts_dir = fonts_dir.to_path_buf();
    let to_compile = content.clone();
    let pdf_bytes = tokio::task::spawn_blocking(move || compile_to_pdf(to_compile, root, fonts_dir))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    // ------------------------------------------------------------------
    // 3. Écriture des fichiers de la version, dans un dossier nommé par
    //    l'identifiant réservé en base (jamais réutilisé)
    // ------------------------------------------------------------------
    on_progress(3, "Écriture de la version…");
    let version_id = db::document_versions::reserve_id(pool).await?;
    let version_dir = doc_dir.join("versions").join(version_id.to_string());
    let written = write_version_files(&version_dir, &content, &pdf_bytes);

    // ------------------------------------------------------------------
    // 4. Enregistrement de la version en base — bloquant : une version
    //    sans ligne en base n'existe pas, ses fichiers sont supprimés
    // ------------------------------------------------------------------
    on_progress(4, "Enregistrement de la version…");
    let recorded = match written {
        Ok((typ_path, pdf_path)) => db::document_versions::insert_version(
            pool,
            version_id,
            session_id,
            doc_type,
            &filename,
            &typ_path,
            &pdf_path,
            &style_analysis,
        )
        .await
        .map(|_| (typ_path, pdf_path)),
        Err(e) => Err(e),
    };
    let (typ_path, pdf_path) = recorded.inspect_err(|_| discard_version_dir(&version_dir))?;

    // ------------------------------------------------------------------
    // 5. Bascule de la référence courante sur cette version
    // ------------------------------------------------------------------
    on_progress(5, "Mise à jour de la référence…");
    let doc = db::documents::upsert(
        pool,
        session_id,
        doc_type,
        &filename,
        &typ_path,
        Some(&pdf_path),
        Some(&style_analysis),
        version_id,
    )
    .await?;

    Ok(DocumentReferenceView::from(doc))
}

/// Compile la référence courante d'un type de document et retourne le PDF encodé en base64.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `fonts_dir` - Dossier des polices importées, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// PDF encodé en base64, prêt à être affiché côté frontend.
pub async fn compile_reference(
    pool: &PgPool,
    fonts_dir: &Path,
    session_id: i64,
    doc_type: &'static str,
) -> Result<String, AppError> {
    let reference = get_reference(pool, session_id, doc_type)
        .await?
        .ok_or_else(|| AppError::NotFound("Aucun fichier de référence trouvé.".to_string()))?;
    let typ_path = PathBuf::from(&reference.file_path);

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

    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
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

/// Sauvegarde un document généré par le pipeline IA (`crate::pipeline`) à
/// l'emplacement définitif d'une candidature : le code Typst est conservé à
/// côté du PDF pour le debug/la traçabilité (`artefact/<candidature_id>.*`).
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `candidature_id` - Identifiant de la candidature à laquelle rattacher le document.
/// * `typst` - Code Typst généré (sauvegardé à des fins de traçabilité).
/// * `pdf` - Octets du PDF compilé.
///
/// # Returns
///
/// Chemin absolu du PDF sauvegardé.
pub fn save_generated_document(
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
    candidature_id: i64,
    typst: &str,
    pdf: &[u8],
) -> Result<String, AppError> {
    let artefact_dir = resolve_doc_dir(data_dir, session_id, doc_type).join("artefact");
    std::fs::create_dir_all(&artefact_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let typ_path = artefact_dir.join(format!("{}.typ", candidature_id));
    std::fs::write(&typ_path, typst.as_bytes())
        .map_err(|e| AppError::Validation(format!("Écriture du fichier Typst impossible : {}", e)))?;

    let pdf_path = artefact_dir.join(format!("{}.pdf", candidature_id));
    std::fs::write(&pdf_path, pdf)
        .map_err(|e| AppError::Validation(format!("Écriture du PDF impossible : {}", e)))?;

    Ok(pdf_path.to_string_lossy().to_string())
}
