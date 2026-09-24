//! Accès et logique de persistance pour les documents de référence (CV,
//! lettre de motivation) : le filesystem reste la source de vérité du
//! contenu, cette table ne tracke que les métadonnées et le cache
//! `style_analysis`.

// Importation des crates externes
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{Document, DocumentsStatus};

/// Construit un `Document` à partir d'une ligne SQL brute.
///
/// Mapping manuel (pas de `#[derive(sqlx::FromRow)]` sur `Document` : ce
/// type vit dans `findmyjob-core`, qui n'a aucune dépendance à `sqlx`).
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête `SELECT`/`RETURNING` sur `documents`.
///
/// # Returns
///
/// `Document` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_document(row: &PgRow) -> Result<Document, sqlx::Error> {
    Ok(Document {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        doc_type: row.try_get("doc_type")?,
        original_filename: row.try_get("original_filename")?,
        file_path: row.try_get("file_path")?,
        pdf_path: row.try_get("pdf_path")?,
        style_analysis: row.try_get("style_analysis")?,
        current_version_id: row.try_get("current_version_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Insère ou met à jour le document de référence d'un type donné pour une session.
///
/// Un seul CV et une seule lettre de motivation existent par session
/// (contrainte `UNIQUE(session_id, doc_type)`) : un upload ultérieur du même
/// type remplace les métadonnées existantes plutôt que d'en créer de nouvelles.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `original_filename` - Nom du fichier tel qu'uploadé par l'utilisateur.
/// * `file_path` - Chemin absolu vers `reference.typ`.
/// * `pdf_path` - Chemin absolu vers `reference.pdf`, si disponible.
/// * `style_analysis` - JSON d'analyse de style produit par le pipeline IA, si disponible.
/// * `current_version_id` - Version (`document_versions.id`) dont ce document devient la copie courante.
///
/// # Returns
///
/// `Document` créé ou mis à jour.
#[allow(clippy::too_many_arguments)]
pub async fn upsert(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    original_filename: &str,
    file_path: &str,
    pdf_path: Option<&str>,
    style_analysis: Option<&str>,
    current_version_id: i64,
) -> Result<Document, AppError> {
    let row = sqlx::query(
        "INSERT INTO search.documents \
             (session_id, doc_type, original_filename, file_path, pdf_path, style_analysis, current_version_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (session_id, doc_type) DO UPDATE SET \
             original_filename = excluded.original_filename, \
             file_path = excluded.file_path, \
             pdf_path = excluded.pdf_path, \
             style_analysis = excluded.style_analysis, \
             current_version_id = excluded.current_version_id \
         RETURNING id, session_id, doc_type, original_filename, file_path, pdf_path, \
                   style_analysis, current_version_id, created_at, updated_at",
    )
    .bind(session_id)
    .bind(doc_type)
    .bind(original_filename)
    .bind(file_path)
    .bind(pdf_path)
    .bind(style_analysis)
    .bind(current_version_id)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    row_to_document(&row).map_err(map_db_err)
}

/// Récupère le document de référence d'un type donné pour une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(Document)` si un document est tracké en base, `None` sinon.
pub async fn get_by_session_and_type(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
) -> Result<Option<Document>, AppError> {
    let row = sqlx::query(
        "SELECT id, session_id, doc_type, original_filename, file_path, pdf_path, \
                style_analysis, current_version_id, created_at, updated_at \
         FROM search.documents WHERE session_id = $1 AND doc_type = $2",
    )
    .bind(session_id)
    .bind(doc_type)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    row.as_ref().map(row_to_document).transpose().map_err(map_db_err)
}

/// Calcule l'état de préparation des documents de référence d'une session
/// (CV et lettre de motivation) : prêt = document tracké et `style_analysis`
/// non nul, seule condition sous laquelle une candidature peut être créée
/// (voir `api::candidatures::create_candidature`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `DocumentsStatus` reflétant l'état des deux documents de référence.
pub async fn status_for_session(pool: &PgPool, session_id: i64) -> Result<DocumentsStatus, AppError> {
    let cv_ready = get_by_session_and_type(pool, session_id, "cv")
        .await?
        .is_some_and(|d| d.style_analysis.is_some());
    let lm_ready = get_by_session_and_type(pool, session_id, "lettre_motivation")
        .await?
        .is_some_and(|d| d.style_analysis.is_some());
    Ok(DocumentsStatus { cv_ready, lm_ready, all_ready: cv_ready && lm_ready })
}
