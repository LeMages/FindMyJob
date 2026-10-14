//! Consultation et restauration des versions d'un document de référence
//! (CV, lettre de motivation) d'une session. La création d'une version a
//! lieu à l'upload (`storage::save_reference`) ; ce module ne fait que lire
//! les versions existantes ou rebasculer la référence courante sur l'une d'elles.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::clock;
use crate::db;
use crate::db::transaction::begin_write;
use crate::documents::storage::read_base64;
use crate::storage_paths;
use crate::error::AppError;
use crate::models::DocumentReferenceView;

/// Lit le PDF d'une version précise d'un document de référence.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro d'affichage de la version.
///
/// # Returns
///
/// PDF encodé en base64, ou `AppError::NotFound` si la version n'existe pas.
pub async fn read_version_pdf(
    pool: &SqlitePool,
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<String, AppError> {
    let found = db::document_versions::get_version(pool, session_id, doc_type, version)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Version {version} introuvable.")))?;
    read_base64(&storage_paths::to_absolute(data_dir, &found.pdf_path))
}

/// Restaure une version d'un document de référence comme référence courante
/// de la session : c'est elle que le pipeline de génération utilisera pour
/// les prochaines candidatures. Aucune version n'est créée ni supprimée, et
/// le template stocké avec la version est réutilisé (pas d'appel à Claude).
/// Lecture de la version et bascule du pointeur partagent une transaction.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro d'affichage de la version à restaurer.
///
/// # Returns
///
/// `DocumentReferenceView` de la nouvelle référence courante,
/// `AppError::NotFound` si la version n'existe pas, `AppError::Validation`
/// si ses fichiers ont disparu du disque.
pub async fn restore_version(
    pool: &SqlitePool,
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<DocumentReferenceView, AppError> {
    let now = clock::now_utc();
    let mut tx = begin_write(pool).await?;
    let record = db::document_versions::get_version_record(&mut tx, session_id, doc_type, version)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Version {version} introuvable.")))?;

    // Une référence dont le fichier manque ferait échouer toutes les
    // générations suivantes : on refuse la bascule plutôt que de la subir.
    let typst_exists = storage_paths::to_absolute(data_dir, &record.typst_path).exists();
    let pdf_exists = storage_paths::to_absolute(data_dir, &record.pdf_path).exists();
    if !typst_exists || !pdf_exists {
        return Err(AppError::Validation(format!(
            "Les fichiers de la version {version} sont introuvables sur le disque ({}).",
            record.typst_path
        )));
    }

    db::documents::set_current(&mut tx, session_id, doc_type, record.id, &now).await?;
    let doc = db::documents::get_current(&mut *tx, session_id, doc_type)
        .await?
        .ok_or_else(|| AppError::Database("Référence courante absente après restauration.".to_string()))?;
    tx.commit().await.map_err(db::common::map_db_err)?;

    Ok(DocumentReferenceView::from(doc))
}
