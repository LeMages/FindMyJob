//! Consultation et restauration des versions d'un document de référence
//! (CV, lettre de motivation) d'une session. La création d'une version a
//! lieu à l'upload (`storage::save_reference`) ; ce module ne fait que lire
//! les versions existantes ou rebasculer la référence courante sur l'une d'elles.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use sqlx::PgPool;

// Importation des modules internes
use crate::db;
use crate::documents::storage::read_document_base64;
use findmyjob_core::error::AppError;
use findmyjob_core::models::DocumentReferenceView;

/// Lit le PDF d'une version précise d'un document de référence.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro d'affichage de la version.
///
/// # Returns
///
/// PDF encodé en base64, ou `AppError::NotFound` si la version n'existe pas.
pub async fn read_version_pdf(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<String, AppError> {
    let found = db::document_versions::get_version(pool, session_id, doc_type, version)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Version {version} introuvable.")))?;
    read_document_base64(&found.pdf_path)
}

/// Lit le PDF de la version la plus récente d'un document de référence.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// PDF encodé en base64, ou `AppError::NotFound` si aucune version n'existe.
pub async fn read_latest_version_pdf(pool: &PgPool, session_id: i64, doc_type: &str) -> Result<String, AppError> {
    let latest = db::document_versions::get_latest_version(pool, session_id, doc_type)
        .await?
        .ok_or_else(|| AppError::NotFound("Aucune version enregistrée pour ce document.".to_string()))?;
    read_document_base64(&latest.pdf_path)
}

/// Restaure une version d'un document de référence comme référence courante
/// de la session : c'est elle que le pipeline de génération utilisera pour
/// les prochaines candidatures. Aucune version n'est créée ni supprimée, et
/// l'analyse de style stockée avec la version est réutilisée (pas d'appel à Claude).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
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
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<DocumentReferenceView, AppError> {
    let record = db::document_versions::get_version_record(pool, session_id, doc_type, version)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Version {version} introuvable.")))?;

    // Une référence dont le fichier manque ferait échouer toutes les
    // générations suivantes : on refuse la bascule plutôt que de la subir.
    if !Path::new(&record.typst_path).exists() || !Path::new(&record.pdf_path).exists() {
        return Err(AppError::Validation(format!(
            "Les fichiers de la version {version} sont introuvables sur le disque ({}).",
            record.typst_path
        )));
    }

    let doc = db::documents::upsert(
        pool,
        session_id,
        doc_type,
        &record.original_filename,
        &record.typst_path,
        Some(&record.pdf_path),
        Some(&record.style_analysis),
        record.id,
    )
    .await?;

    Ok(DocumentReferenceView::from(doc))
}
