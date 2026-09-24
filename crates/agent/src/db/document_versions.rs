//! Accès et logique de persistance pour l'historique des documents de
//! référence (CV, lettre de motivation) d'une session : une ligne par upload,
//! jamais écrasée. La version courante est pointée par
//! `documents.current_version_id` (voir `db::documents`).
//!
//! L'`id` d'une version nomme aussi son dossier sur disque ; il est réservé
//! avant l'écriture des fichiers (`reserve_id`) pour que le chemin ne dépende
//! jamais du numéro d'affichage `version`.

// Importation des crates externes
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::DocumentVersionView;

/// Colonnes de `DocumentVersionView`, `is_current` calculé par jointure sur
/// `documents.current_version_id` (alias `v` pour `document_versions`, `d`
/// pour `documents`).
const VIEW_COLUMNS: &str = "v.id, v.session_id, v.doc_type, v.version, v.original_filename, \
     v.typst_path, v.pdf_path, v.created_at, (d.current_version_id IS NOT NULL) AS is_current";

/// Jointure utilisée par toutes les lectures de `DocumentVersionView`.
const VIEW_FROM: &str = "search.document_versions v \
     LEFT JOIN search.documents d ON d.current_version_id = v.id";

/// Version complète d'un document de référence, avec son analyse de style —
/// nécessaire pour restaurer la version comme référence courante sans
/// rappeler Claude. Interne à l'agent, jamais exposée par l'API.
///
/// # Attributes
///
/// * `id` - Identifiant de la version.
/// * `original_filename` - Nom du fichier tel qu'uploadé.
/// * `typst_path` - Chemin absolu du `.typ`.
/// * `pdf_path` - Chemin absolu du `.pdf`.
/// * `style_analysis` - Analyse de style calculée à l'upload.
pub struct VersionRecord {
    pub id: i64,
    pub original_filename: String,
    pub typst_path: String,
    pub pdf_path: String,
    pub style_analysis: String,
}

/// Construit un `DocumentVersionView` à partir d'une ligne SQL sélectionnant `VIEW_COLUMNS`.
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête sur `VIEW_FROM`.
///
/// # Returns
///
/// `DocumentVersionView` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &PgRow) -> Result<DocumentVersionView, sqlx::Error> {
    Ok(DocumentVersionView {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        doc_type: row.try_get("doc_type")?,
        version: row.try_get("version")?,
        original_filename: row.try_get("original_filename")?,
        typst_path: row.try_get("typst_path")?,
        pdf_path: row.try_get("pdf_path")?,
        created_at: row.try_get("created_at")?,
        is_current: row.try_get("is_current")?,
    })
}

/// Réserve un identifiant de version dans la séquence de `document_versions`,
/// avant toute écriture disque : le dossier de la version est nommé par cet
/// identifiant. Une valeur réservée mais jamais insérée (échec d'écriture)
/// est simplement perdue, jamais réutilisée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// Identifiant à passer à `insert_version`.
pub async fn reserve_id(pool: &PgPool) -> Result<i64, AppError> {
    sqlx::query("SELECT nextval(pg_get_serial_sequence('search.document_versions', 'id')) AS id")
        .fetch_one(pool)
        .await
        .map_err(map_db_err)?
        .try_get("id")
        .map_err(map_db_err)
}

/// Insère une nouvelle version d'un document de référence, avec l'identifiant
/// réservé par `reserve_id`. Le numéro `version` est calculé en base
/// (`MAX + 1` pour ce couple session/doc_type) ; une insertion concurrente
/// sur le même couple échoue sur la contrainte `uq_document_version`.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant réservé via `reserve_id`.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `original_filename` - Nom du fichier tel qu'uploadé.
/// * `typst_path` - Chemin absolu du `.typ` de la version.
/// * `pdf_path` - Chemin absolu du `.pdf` de la version.
/// * `style_analysis` - Analyse de style calculée pour ce contenu.
///
/// # Returns
///
/// `DocumentVersionView` de la version créée (`is_current = false` tant que
/// `documents.current_version_id` n'a pas été mis à jour).
#[allow(clippy::too_many_arguments)]
pub async fn insert_version(
    pool: &PgPool,
    id: i64,
    session_id: i64,
    doc_type: &str,
    original_filename: &str,
    typst_path: &str,
    pdf_path: &str,
    style_analysis: &str,
) -> Result<DocumentVersionView, AppError> {
    let row = sqlx::query(
        "INSERT INTO search.document_versions \
             (id, session_id, doc_type, version, original_filename, typst_path, pdf_path, style_analysis) \
         VALUES ($1, $2, $3, \
                 (SELECT COALESCE(MAX(version), 0) + 1 FROM search.document_versions \
                  WHERE session_id = $2 AND doc_type = $3), \
                 $4, $5, $6, $7) \
         RETURNING id, session_id, doc_type, version, original_filename, typst_path, pdf_path, \
                   created_at, FALSE AS is_current",
    )
    .bind(id)
    .bind(session_id)
    .bind(doc_type)
    .bind(original_filename)
    .bind(typst_path)
    .bind(pdf_path)
    .bind(style_analysis)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    row_to_view(&row).map_err(map_db_err)
}

/// Liste toutes les versions d'un document de référence, de la plus récente
/// à la plus ancienne.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Versions triées par `version` décroissante (vide pour une référence
/// antérieure au versioning).
pub async fn list_versions(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
) -> Result<Vec<DocumentVersionView>, AppError> {
    let rows = sqlx::query(&format!(
        "SELECT {VIEW_COLUMNS} FROM {VIEW_FROM} \
         WHERE v.session_id = $1 AND v.doc_type = $2 \
         ORDER BY v.version DESC"
    ))
    .bind(session_id)
    .bind(doc_type)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Récupère la version la plus récente (numéro le plus élevé) d'un document
/// de référence — pas forcément la version courante après une restauration.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `Some(DocumentVersionView)` si au moins une version existe, `None` sinon.
pub async fn get_latest_version(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
) -> Result<Option<DocumentVersionView>, AppError> {
    let row = sqlx::query(&format!(
        "SELECT {VIEW_COLUMNS} FROM {VIEW_FROM} \
         WHERE v.session_id = $1 AND v.doc_type = $2 \
         ORDER BY v.version DESC LIMIT 1"
    ))
    .bind(session_id)
    .bind(doc_type)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    row.as_ref().map(row_to_view).transpose().map_err(map_db_err)
}

/// Récupère une version précise d'un document de référence.
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
/// `Some(DocumentVersionView)` si la version existe, `None` sinon.
pub async fn get_version(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<Option<DocumentVersionView>, AppError> {
    let row = sqlx::query(&format!(
        "SELECT {VIEW_COLUMNS} FROM {VIEW_FROM} \
         WHERE v.session_id = $1 AND v.doc_type = $2 AND v.version = $3"
    ))
    .bind(session_id)
    .bind(doc_type)
    .bind(version)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    row.as_ref().map(row_to_view).transpose().map_err(map_db_err)
}

/// Récupère une version précise avec son analyse de style, pour la restaurer
/// comme référence courante.
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
/// `Some(VersionRecord)` si la version existe, `None` sinon.
pub async fn get_version_record(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<Option<VersionRecord>, AppError> {
    let row = sqlx::query(
        "SELECT id, original_filename, typst_path, pdf_path, style_analysis \
         FROM search.document_versions \
         WHERE session_id = $1 AND doc_type = $2 AND version = $3",
    )
    .bind(session_id)
    .bind(doc_type)
    .bind(version)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(VersionRecord {
        id: row.try_get("id").map_err(map_db_err)?,
        original_filename: row.try_get("original_filename").map_err(map_db_err)?,
        typst_path: row.try_get("typst_path").map_err(map_db_err)?,
        pdf_path: row.try_get("pdf_path").map_err(map_db_err)?,
        style_analysis: row.try_get("style_analysis").map_err(map_db_err)?,
    }))
}
