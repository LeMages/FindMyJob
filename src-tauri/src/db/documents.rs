//! Accès et logique de persistance pour les documents de référence (CV,
//! lettre de motivation) : le filesystem reste la source de vérité du
//! contenu, cette table ne tracke que les métadonnées et le cache
//! `style_analysis`.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::error::AppError;
use crate::models::Document;

/// Insère ou met à jour le document de référence d'un type donné pour une session.
///
/// Un seul CV et une seule lettre de motivation existent par session
/// (contrainte `UNIQUE(session_id, doc_type)`) : un upload ultérieur du même
/// type remplace les métadonnées existantes plutôt que d'en créer de nouvelles.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `original_filename` - Nom du fichier tel qu'uploadé par l'utilisateur.
/// * `file_path` - Chemin absolu vers `reference.typ`.
/// * `pdf_path` - Chemin absolu vers `reference.pdf`, si disponible.
/// * `style_analysis` - JSON d'analyse de style produit par le pipeline IA, si disponible.
///
/// # Returns
///
/// `Document` créé ou mis à jour.
pub async fn upsert(
    pool: &SqlitePool,
    session_id: i64,
    doc_type: &str,
    original_filename: &str,
    file_path: &str,
    pdf_path: Option<&str>,
    style_analysis: Option<&str>,
) -> Result<Document, AppError> {
    let doc = sqlx::query_as::<_, Document>(
        "INSERT INTO documents (session_id, doc_type, original_filename, file_path, pdf_path, style_analysis) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(session_id, doc_type) DO UPDATE SET \
             original_filename = excluded.original_filename, \
             file_path = excluded.file_path, \
             pdf_path = excluded.pdf_path, \
             style_analysis = excluded.style_analysis, \
             updated_at = datetime('now') \
         RETURNING id, session_id, doc_type, original_filename, file_path, pdf_path, \
                   style_analysis, created_at, updated_at",
    )
    .bind(session_id)
    .bind(doc_type)
    .bind(original_filename)
    .bind(file_path)
    .bind(pdf_path)
    .bind(style_analysis)
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

/// Récupère le document de référence d'un type donné pour une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(Document)` si un document est tracké en base, `None` sinon.
pub async fn get_by_session_and_type(
    pool: &SqlitePool,
    session_id: i64,
    doc_type: &str,
) -> Result<Option<Document>, AppError> {
    let doc = sqlx::query_as::<_, Document>(
        "SELECT id, session_id, doc_type, original_filename, file_path, pdf_path, \
                style_analysis, created_at, updated_at \
         FROM documents WHERE session_id = ? AND doc_type = ?",
    )
    .bind(session_id)
    .bind(doc_type)
    .fetch_optional(pool)
    .await?;

    Ok(doc)
}

/// Récupère tous les documents de référence trackés pour une session (CV et/ou lettre).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
///
/// # Returns
///
/// Documents trouvés, sans ordre garanti.
pub async fn get_by_session(pool: &SqlitePool, session_id: i64) -> Result<Vec<Document>, AppError> {
    let docs = sqlx::query_as::<_, Document>(
        "SELECT id, session_id, doc_type, original_filename, file_path, pdf_path, \
                style_analysis, created_at, updated_at \
         FROM documents WHERE session_id = ?",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(docs)
}

/// Met à jour le cache d'analyse de style LLM d'un document.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `document_id` - Identifiant du document à mettre à jour.
/// * `style_analysis` - Contenu JSON de l'analyse à mettre en cache.
///
/// # Returns
///
/// `Ok(())` si la mise à jour a réussi.
pub async fn set_style_analysis(
    pool: &SqlitePool,
    document_id: i64,
    style_analysis: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE documents SET style_analysis = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(style_analysis)
        .bind(document_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Supprime un document de référence tracké en base.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant du document à supprimer.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM documents WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
