//! Stockage sur disque des documents de référence (CV, lettre de motivation)
//! et des documents joints à une candidature : résolution de chemins,
//! sauvegarde avec historique, compilation Typst → PDF, mise en scène des
//! fichiers avant rattachement définitif à une candidature.
//!
//! Chaque session de candidature a son propre CV et sa propre lettre de
//! motivation : tout est stocké sous `documents/sessions/<session_id>/`.
//!
//! La résolution de chemins liée à `tauri::AppHandle` (dossier de données,
//! dossier de téléchargements) et l'émission d'événements de progression
//! vivent dans `crates/desktop/src-tauri/src/documents/storage.rs`, qui
//! délègue à ce module une fois ces chemins résolus.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::documents::typst::compile_to_pdf;
use crate::error::AppError;
use crate::models::{CandidatureDetail, DocumentFileInfo, DocumentReferenceView};

/// Nombre total d'étapes du flux de `save_reference`, communiqué à
/// l'appelant pour calculer une progression (`step`/`total`).
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
fn updated_at_millis(path: &Path) -> String {
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
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(DocumentReferenceView)` si un fichier de référence existe, `None` sinon.
pub async fn get_reference(
    pool: &SqlitePool,
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
) -> Option<DocumentReferenceView> {
    let doc_dir = resolve_doc_dir(data_dir, session_id, doc_type);
    let typ_path = doc_dir.join("reference.typ");
    if !typ_path.exists() {
        return None;
    }

    // Lecture DB best-effort : prépare l'enrichissement futur de la réponse
    // (ex. style_analysis), sans influencer le comportement actuel — le
    // filesystem reste la source de vérité de l'existence du document.
    if let Err(e) = db::documents::get_by_session_and_type(pool, session_id, doc_type).await {
        eprintln!("Lecture DB du document ({doc_type}, session {session_id}) échouée : {e}");
    }

    let pdf_path = doc_dir.join("reference.pdf");
    Some(DocumentReferenceView {
        file_path: typ_path.to_string_lossy().to_string(),
        pdf_path: pdf_path.to_string_lossy().to_string(),
        updated_at: updated_at_millis(&typ_path),
    })
}

/// Analyse le style du document via l'API Claude, puis sauvegarde le nouveau
/// document de référence (.typ) en archivant l'ancien s'il existe, et le
/// compile immédiatement en PDF.
///
/// L'analyse de style est bloquante et a lieu avant toute écriture disque :
/// si l'appel à Claude échoue ou renvoie un format inattendu, le save entier
/// échoue et rien n'est modifié (ni fichier, ni entrée DB).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
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
    pool: &SqlitePool,
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
    //    inattendu, rien n'est créé (ni dossier, ni backup, ni fichier).
    // ------------------------------------------------------------------
    let model = db::settings::resolve_llm_model(pool, crate::llm::client::DEFAULT_MODEL).await?;

    on_progress(1, "Analyse du style (IA)…");
    let style_analysis = crate::llm::analyze_style(&content, doc_type, &model).await?;

    let doc_dir = resolve_doc_dir(data_dir, session_id, doc_type);
    std::fs::create_dir_all(&doc_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let typ_path = doc_dir.join("reference.typ");
    let pdf_path = doc_dir.join("reference.pdf");

    // ------------------------------------------------------------------
    // 2. Sauvegarde de l'ancienne référence (si elle existe) dans backup/<timestamp>/
    // ------------------------------------------------------------------
    on_progress(2, "Sauvegarde de l'historique…");
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
    on_progress(3, "Écriture du fichier…");
    std::fs::write(&typ_path, content.as_bytes())
        .map_err(|e| AppError::Validation(format!("Écriture du fichier impossible : {}", e)))?;

    // ------------------------------------------------------------------
    // 4. Compilation en PDF
    // ------------------------------------------------------------------
    on_progress(4, "Compilation en PDF…");
    let root = doc_dir.clone();
    let fonts_dir = fonts_dir.to_path_buf();
    let pdf_bytes = tokio::task::spawn_blocking(move || compile_to_pdf(content, root, fonts_dir))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    std::fs::write(&pdf_path, &pdf_bytes)
        .map_err(|e| AppError::Validation(format!("Écriture du PDF impossible : {}", e)))?;

    // ------------------------------------------------------------------
    // 5. Trace en DB (non bloquant : le filesystem reste la source de
    //    vérité, on ne veut pas perdre un fichier correctement sauvegardé
    //    à cause d'une erreur DB)
    // ------------------------------------------------------------------
    on_progress(5, "Enregistrement en base…");
    if let Err(e) = db::documents::upsert(
        pool,
        session_id,
        doc_type,
        &filename,
        &typ_path.to_string_lossy(),
        Some(&pdf_path.to_string_lossy()),
        Some(&style_analysis),
    )
    .await
    {
        eprintln!("Trace DB du document ({doc_type}, session {session_id}) échouée : {e}");
    }

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
/// * `data_dir` - Dossier de données de l'application, déjà résolu par l'appelant.
/// * `fonts_dir` - Dossier des polices importées, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// PDF encodé en base64, prêt à être affiché côté frontend.
pub async fn compile_reference(
    data_dir: &Path,
    fonts_dir: &Path,
    session_id: i64,
    doc_type: &'static str,
) -> Result<String, AppError> {
    let doc_dir = resolve_doc_dir(data_dir, session_id, doc_type);
    let typ_path = doc_dir.join("reference.typ");
    if !typ_path.exists() {
        return Err(AppError::NotFound("Aucun fichier de référence trouvé.".to_string()));
    }

    let content = std::fs::read_to_string(&typ_path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le fichier : {}", e)))?;
    let root = doc_dir;
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

/// Finalise un document mis en scène en le déplaçant vers son emplacement
/// définitif, nommé d'après l'identifiant de la candidature.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application, déjà résolu par l'appelant.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `staged_path` - Chemin temporaire du fichier mis en scène, s'il y en a un.
/// * `candidature_id` - Identifiant de la candidature à laquelle rattacher le document.
///
/// # Returns
///
/// `Some(chemin_final)` si un document était en attente, `None` sinon.
pub fn finalize_candidature_document(
    data_dir: &Path,
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
    let artefact_dir = resolve_doc_dir(data_dir, session_id, doc_type).join("artefact");
    std::fs::create_dir_all(&artefact_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;
    let dest = artefact_dir.join(format!("{}.pdf", candidature_id));
    std::fs::rename(&staged_pb, &dest)
        .map_err(|e| AppError::Validation(format!("Impossible de finaliser le document : {}", e)))?;
    Ok(Some(dest.to_string_lossy().to_string()))
}

/// Remplace les caractères invalides pour un nom de fichier/dossier Windows
/// (`< > : " / \ | ? *`) par `_`, pour bâtir un nom de dossier sûr à partir de
/// données utilisateur (poste, entreprise).
///
/// # Arguments
///
/// * `name` - Nom brut à assainir.
///
/// # Returns
///
/// Nom assaini, débarrassé des espaces superflus en tête/fin.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if r#"<>:"/\|?*"#.contains(c) { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Construit le nom du sous-dossier de destination `<poste>_<entreprise>_<id>`
/// utilisé pour exporter les documents d'une candidature vers le dossier de
/// téléchargements de l'utilisateur.
///
/// # Arguments
///
/// * `detail` - Détail de la candidature (poste, entreprise, id).
///
/// # Returns
///
/// Nom de dossier assaini, sûr sur tous les OS.
pub fn build_export_dir_name(detail: &CandidatureDetail) -> String {
    let entreprise = detail.entreprise.as_deref().unwrap_or("entreprise");
    sanitize_filename(&format!("{}_{}_{}", detail.poste, entreprise, detail.id))
}

/// Copie le CV et la lettre de motivation d'une candidature vers `dest_dir`
/// — logique commune aux trois façons de postuler.
///
/// # Arguments
///
/// * `dest_dir` - Dossier de destination, déjà résolu par l'appelant (dossier de téléchargements + nom de sous-dossier).
/// * `detail` - Détail de la candidature (chemins des documents).
///
/// # Returns
///
/// `Ok(())` une fois les deux fichiers copiés, ou `AppError::Validation` si un
/// document est manquant (candidature sans CV/lettre) ou introuvable sur le disque.
pub fn copy_candidature_documents(dest_dir: &Path, detail: &CandidatureDetail) -> Result<(), AppError> {
    let cv_path = detail.cv_path.as_deref().ok_or_else(|| {
        AppError::Validation("Aucun CV disponible pour cette candidature.".to_string())
    })?;
    let lettre_path = detail.lettre_motivation_path.as_deref().ok_or_else(|| {
        AppError::Validation("Aucune lettre de motivation disponible pour cette candidature.".to_string())
    })?;

    std::fs::create_dir_all(dest_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    std::fs::copy(cv_path, dest_dir.join("CV.pdf"))
        .map_err(|e| AppError::Validation(format!("Copie du CV impossible : {}", e)))?;
    std::fs::copy(lettre_path, dest_dir.join("Lettre_de_motivation.pdf"))
        .map_err(|e| AppError::Validation(format!("Copie de la lettre de motivation impossible : {}", e)))?;

    Ok(())
}

/// Écrit le message de candidature rédigé par l'IA (mail ou message court)
/// dans le dossier d'export — utilisé par « Postuler par mail » et « Postuler
/// par message », après `copy_candidature_documents`.
///
/// # Arguments
///
/// * `dest_dir` - Dossier de destination (même que `copy_candidature_documents`).
/// * `message` - Contenu texte du message rédigé par l'IA.
/// * `message_filename` - Nom du fichier `.txt` à créer (ex. `Mail_de_candidature.txt`).
///
/// # Returns
///
/// `Ok(())` une fois le fichier écrit.
pub fn write_export_message(dest_dir: &Path, message: &str, message_filename: &str) -> Result<(), AppError> {
    std::fs::write(dest_dir.join(message_filename), message)
        .map_err(|e| AppError::Validation(format!("Écriture du message impossible : {}", e)))
}

/// Sauvegarde un document généré par le pipeline IA (`llm::pipeline`) à
/// l'emplacement définitif d'une candidature : le code Typst est conservé à
/// côté du PDF pour le debug/la traçabilité, sous le même nom que
/// `finalize_candidature_document` (`artefact/<candidature_id>.*`).
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
