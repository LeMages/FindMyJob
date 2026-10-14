//! Accès et logique de persistance pour les références courantes (CV,
//! lettre de motivation) d'une session.
//!
//! La table `documents` n'est qu'un pointeur `(session_id, doc_type) →
//! current_version_id` : fichiers et templatisation sont lus sur la version
//! pointée (`document_versions`), par jointure.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::error::AppError;
use crate::models::{Document, DocumentsStatus};

/// Construit un `Document` à partir d'une ligne de la jointure pointeur × version.
///
/// Mapping manuel (pas de `#[derive(sqlx::FromRow)]` sur `Document` : ce
/// type vit dans `models/`, qui n'a aucune dépendance à `sqlx`).
///
/// # Arguments
///
/// * `row` - Ligne issue de la requête de `get_current`.
///
/// # Returns
///
/// `Document` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_document(row: &SqliteRow) -> Result<Document, sqlx::Error> {
    Ok(Document {
        session_id: row.try_get("session_id")?,
        doc_type: row.try_get("doc_type")?,
        current_version_id: row.try_get("current_version_id")?,
        original_filename: row.try_get("original_filename")?,
        file_path: row.try_get("typst_path")?,
        pdf_path: row.try_get("pdf_path")?,
        template_path: row.try_get("template_path")?,
        example_data_path: row.try_get("example_data_path")?,
        page_count: row.try_get("page_count")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Fait pointer la référence d'un type donné d'une session vers une version
/// (premier upload, nouvel upload ou restauration).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version_id` - Version (`document_versions.id`) qui devient courante.
/// * `now` - Horodatage du changement, exposé comme `updated_at`.
///
/// # Returns
///
/// `Ok(())` une fois le pointeur écrit.
pub async fn set_current(
    conn: &mut SqliteConnection,
    session_id: i64,
    doc_type: &str,
    version_id: i64,
    now: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO documents (session_id, doc_type, current_version_id, updated_at) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (session_id, doc_type) DO UPDATE SET \
             current_version_id = excluded.current_version_id, \
             updated_at = excluded.updated_at",
    )
    .bind(session_id)
    .bind(doc_type)
    .bind(version_id)
    .bind(now)
    .execute(conn)
    .await
    .map_err(map_db_err)?;
    Ok(())
}

/// Récupère la référence courante d'un type donné pour une session.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(Document)` si une référence est définie, `None` sinon.
pub async fn get_current<'e, E: SqliteExecutor<'e>>(
    executor: E,
    session_id: i64,
    doc_type: &str,
) -> Result<Option<Document>, AppError> {
    let row = sqlx::query(
        "SELECT d.session_id, d.doc_type, d.current_version_id, d.updated_at, \
                v.original_filename, v.typst_path, v.pdf_path, \
                v.template_path, v.example_data_path, v.page_count \
         FROM documents d \
         JOIN document_versions v ON v.id = d.current_version_id \
         WHERE d.session_id = $1 AND d.doc_type = $2",
    )
    .bind(session_id)
    .bind(doc_type)
    .fetch_optional(executor)
    .await
    .map_err(map_db_err)?;

    row.as_ref().map(row_to_document).transpose().map_err(map_db_err)
}

/// Calcule l'état de préparation des références d'une session (CV et
/// lettre de motivation) : prêt = référence définie et templatisée, seule
/// condition sous laquelle une candidature peut être créée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// `DocumentsStatus` reflétant l'état des deux références.
pub async fn status_for_session(pool: &SqlitePool, session_id: i64) -> Result<DocumentsStatus, AppError> {
    let cv_ready = get_current(pool, session_id, "cv")
        .await?
        .is_some_and(|d| d.template().is_some());
    let lm_ready = get_current(pool, session_id, "lettre_motivation")
        .await?
        .is_some_and(|d| d.template().is_some());
    Ok(DocumentsStatus { cv_ready, lm_ready, all_ready: cv_ready && lm_ready })
}
