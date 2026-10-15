//! Accès et logique de persistance pour l'historique des documents de
//! référence (CV, lettre de motivation) d'une session : une ligne par upload,
//! jamais écrasée. La version courante est pointée par
//! `documents.current_version_id` (voir `db::documents`).
//!
//! L'`id` d'une version nomme aussi son dossier sur disque. Il est réservé
//! par `next_id`, dans la transaction d'écriture qui insère la version : le
//! verrou d'écriture garantit qu'aucun autre écrivain ne prend le même, et
//! `AUTOINCREMENT` garantit qu'un id supprimé n'est jamais réattribué.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::error::AppError;
use crate::models::{DocumentVersionView, TemplateArtifacts};

/// Colonnes de `DocumentVersionView`, `is_current` calculé par jointure sur
/// `documents.current_version_id` (alias `v` pour `document_versions`, `d`
/// pour `documents`).
const VIEW_COLUMNS: &str = "v.id, v.session_id, v.doc_type, v.version, v.original_filename, \
     v.typst_path, v.pdf_path, v.created_at, (d.current_version_id IS NOT NULL) AS is_current";

/// Jointure utilisée par toutes les lectures de `DocumentVersionView`.
const VIEW_FROM: &str = "document_versions v LEFT JOIN documents d ON d.current_version_id = v.id";

/// Version complète d'un document de référence, avec ses fichiers de
/// templatisation, nécessaires pour la relire ou la restaurer comme
/// référence courante sans rappeler Claude. Interne, jamais exposée au frontend.
///
/// # Attributes
///
/// * `id` - Identifiant de la version.
/// * `original_filename` - Nom du fichier tel qu'uploadé.
/// * `typst_path` - Chemin du `.typ`, relatif au dossier de données.
/// * `pdf_path` - Chemin du `.pdf`, relatif au dossier de données.
/// * `template` - Fichiers de templatisation, `None` si la version n'a pas été templatisée.
pub struct VersionRecord {
    pub id: i64,
    pub original_filename: String,
    pub typst_path: String,
    pub pdf_path: String,
    pub template: Option<TemplateArtifacts>,
}

/// Nouvelle version à insérer, fichiers déjà en place dans son dossier.
///
/// # Attributes
///
/// * `id` - Identifiant réservé par `next_id` (nomme le dossier de la version).
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `original_filename` - Nom du fichier tel qu'uploadé.
/// * `typst_path` - Chemin du `.typ`, relatif au dossier de données.
/// * `pdf_path` - Chemin du `.pdf`, relatif au dossier de données.
/// * `template` - Fichiers de templatisation produits à l'upload.
pub struct NewVersion<'a> {
    pub id: i64,
    pub session_id: i64,
    pub doc_type: &'a str,
    pub original_filename: &'a str,
    pub typst_path: &'a str,
    pub pdf_path: &'a str,
    pub template: &'a TemplateArtifacts,
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
fn row_to_view(row: &SqliteRow) -> Result<DocumentVersionView, sqlx::Error> {
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

/// Réserve l'identifiant de la prochaine version, à appeler dans la
/// transaction d'écriture (`begin_write`) qui insère ensuite la version :
/// le verrou d'écriture est déjà pris, aucun autre écrivain ne peut
/// réserver le même identifiant entre-temps.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
///
/// # Returns
///
/// Identifiant à passer à `insert_version`, jamais attribué auparavant.
pub async fn next_id(conn: &mut SqliteConnection) -> Result<i64, AppError> {
    // `sqlite_sequence` n'a de ligne pour la table qu'après sa première
    // insertion ; elle porte le plus grand id jamais attribué (AUTOINCREMENT).
    sqlx::query_scalar(
        "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name = 'document_versions'), 0) + 1",
    )
    .fetch_one(conn)
    .await
    .map_err(map_db_err)
}

/// Insère une nouvelle version d'un document de référence, avec
/// l'identifiant réservé par `next_id`. Le numéro `version` est calculé en
/// base (`MAX + 1` pour ce couple session/doc_type).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `new` - Version à insérer.
/// * `now` - Horodatage d'upload.
///
/// # Returns
///
/// `DocumentVersionView` de la version créée (`is_current = false` tant que
/// `documents.current_version_id` n'a pas été mis à jour).
pub async fn insert_version(
    conn: &mut SqliteConnection,
    new: &NewVersion<'_>,
    now: &str,
) -> Result<DocumentVersionView, AppError> {
    // `$2` et `$3` sont réutilisés dans la sous-requête : sqlx lie les
    // paramètres `$N` par leur numéro, pas par leur position.
    let row = sqlx::query(
        "INSERT INTO document_versions \
             (id, session_id, doc_type, version, original_filename, typst_path, pdf_path, \
              template_path, example_data_path, page_count, created_at) \
         VALUES ($1, $2, $3, \
                 (SELECT COALESCE(MAX(version), 0) + 1 FROM document_versions \
                  WHERE session_id = $2 AND doc_type = $3), \
                 $4, $5, $6, $7, $8, $9, $10) \
         RETURNING id, session_id, doc_type, version, original_filename, typst_path, pdf_path, \
                   created_at, FALSE AS is_current",
    )
    .bind(new.id)
    .bind(new.session_id)
    .bind(new.doc_type)
    .bind(new.original_filename)
    .bind(new.typst_path)
    .bind(new.pdf_path)
    .bind(&new.template.template_path)
    .bind(&new.template.example_data_path)
    .bind(new.template.page_count)
    .bind(now)
    .fetch_one(conn)
    .await
    .map_err(map_db_err)?;

    row_to_view(&row).map_err(map_db_err)
}

/// Liste toutes les versions d'un document de référence, de la plus récente
/// à la plus ancienne.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Versions triées par `version` décroissante.
pub async fn list_versions(
    pool: &SqlitePool,
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

/// Récupère une version précise d'un document de référence.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro d'affichage de la version.
///
/// # Returns
///
/// `Some(DocumentVersionView)` si la version existe, `None` sinon.
pub async fn get_version(
    pool: &SqlitePool,
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

/// Récupère une version précise avec ses fichiers de templatisation, pour
/// la restaurer comme référence courante.
///
/// # Arguments
///
/// * `conn` - Connexion (transaction de restauration).
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro d'affichage de la version.
///
/// # Returns
///
/// `Some(VersionRecord)` si la version existe, `None` sinon.
pub async fn get_version_record(
    conn: &mut SqliteConnection,
    session_id: i64,
    doc_type: &str,
    version: i32,
) -> Result<Option<VersionRecord>, AppError> {
    let row = sqlx::query(
        "SELECT id, original_filename, typst_path, pdf_path, template_path, example_data_path, page_count \
         FROM document_versions \
         WHERE session_id = $1 AND doc_type = $2 AND version = $3",
    )
    .bind(session_id)
    .bind(doc_type)
    .bind(version)
    .fetch_optional(conn)
    .await
    .map_err(map_db_err)?;

    let Some(row) = row else {
        return Ok(None);
    };
    let template_path: Option<String> = row.try_get("template_path").map_err(map_db_err)?;
    let example_data_path: Option<String> = row.try_get("example_data_path").map_err(map_db_err)?;
    let page_count: Option<i32> = row.try_get("page_count").map_err(map_db_err)?;
    let template = match (template_path, example_data_path, page_count) {
        (Some(template_path), Some(example_data_path), Some(page_count)) => {
            Some(TemplateArtifacts { template_path, example_data_path, page_count })
        }
        _ => None,
    };

    Ok(Some(VersionRecord {
        id: row.try_get("id").map_err(map_db_err)?,
        original_filename: row.try_get("original_filename").map_err(map_db_err)?,
        typst_path: row.try_get("typst_path").map_err(map_db_err)?,
        pdf_path: row.try_get("pdf_path").map_err(map_db_err)?,
        template,
    }))
}
