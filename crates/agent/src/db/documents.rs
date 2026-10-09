//! Accès et logique de persistance pour les documents de référence (CV,
//! lettre de motivation) : le filesystem reste la source de vérité du
//! contenu, cette table ne tracke que les métadonnées et les chemins des
//! fichiers de templatisation (`template_path`, `example_data_path`).

// Importation des crates externes
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{Document, DocumentsStatus, TemplateArtifacts};

/// Colonnes lues pour reconstruire un `Document`.
const DOCUMENT_COLUMNS: &str = "id, session_id, doc_type, original_filename, file_path, pdf_path, \
     template_path, example_data_path, page_count, current_version_id, created_at, updated_at";

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
        template_path: row.try_get("template_path")?,
        example_data_path: row.try_get("example_data_path")?,
        page_count: row.try_get("page_count")?,
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
/// * `template` - Fichiers de templatisation de la version, `None` si elle n'est pas templatisée.
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
    template: Option<&TemplateArtifacts>,
    current_version_id: i64,
) -> Result<Document, AppError> {
    let row = sqlx::query(&format!(
        "INSERT INTO search.documents \
             (session_id, doc_type, original_filename, file_path, pdf_path, \
              template_path, example_data_path, page_count, current_version_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         ON CONFLICT (session_id, doc_type) DO UPDATE SET \
             original_filename = excluded.original_filename, \
             file_path = excluded.file_path, \
             pdf_path = excluded.pdf_path, \
             template_path = excluded.template_path, \
             example_data_path = excluded.example_data_path, \
             page_count = excluded.page_count, \
             current_version_id = excluded.current_version_id \
         RETURNING {DOCUMENT_COLUMNS}"
    ))
    .bind(session_id)
    .bind(doc_type)
    .bind(original_filename)
    .bind(file_path)
    .bind(pdf_path)
    .bind(template.map(|t| t.template_path.as_str()))
    .bind(template.map(|t| t.example_data_path.as_str()))
    .bind(template.map(|t| t.page_count))
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
    let row = sqlx::query(&format!(
        "SELECT {DOCUMENT_COLUMNS} FROM search.documents WHERE session_id = $1 AND doc_type = $2"
    ))
    .bind(session_id)
    .bind(doc_type)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    row.as_ref().map(row_to_document).transpose().map_err(map_db_err)
}

/// Calcule l'état de préparation des documents de référence d'une session
/// (CV et lettre de motivation) : prêt = document tracké et templatisé
/// (`template_path` non nul), seule condition sous laquelle une candidature peut être créée
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
        .is_some_and(|d| d.template().is_some());
    let lm_ready = get_by_session_and_type(pool, session_id, "lettre_motivation")
        .await?
        .is_some_and(|d| d.template().is_some());
    Ok(DocumentsStatus { cv_ready, lm_ready, all_ready: cv_ready && lm_ready })
}

/// Enregistre les fichiers de templatisation de la référence courante d'une
/// session, sans changer de version (re-templatisation d'une référence
/// existante, voir `documents::templatize::retemplatize_current`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `template` - Fichiers de templatisation à enregistrer.
///
/// # Returns
///
/// `Document` mis à jour, ou `AppError::NotFound` si aucune référence n'est trackée.
pub async fn set_template(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    template: &TemplateArtifacts,
) -> Result<Document, AppError> {
    let row = sqlx::query(&format!(
        "UPDATE search.documents \
         SET template_path = $3, example_data_path = $4, page_count = $5 \
         WHERE session_id = $1 AND doc_type = $2 \
         RETURNING {DOCUMENT_COLUMNS}"
    ))
    .bind(session_id)
    .bind(doc_type)
    .bind(&template.template_path)
    .bind(&template.example_data_path)
    .bind(template.page_count)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Aucun document de référence trouvé.".to_string()))?;
    row_to_document(&row).map_err(map_db_err)
}
