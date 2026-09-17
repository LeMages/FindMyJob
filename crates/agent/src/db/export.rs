//! Export complet des données utilisateur au format JSON (sauvegarde
//! manuelle). Conversion générique ligne SQL -> JSON : `SELECT * FROM
//! <table>` sur des tables hétérogènes ne peut pas raisonnablement passer
//! par une struct typée par table sans duplication massive.

// Importation des crates externes
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{Map, Value};
use sqlx::postgres::PgRow;
use sqlx::{Column, PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;

/// Tables applicatives incluses dans l'export, dans l'ordre d'apparition
/// dans le JSON produit.
///
/// `embeddings` et `emails` (tables supprimées par la migration
/// `0004_cleanup_schema.sql`) et `email_attachments` (table supprimée par la
/// migration SQLite `0009_refactor_documents.sql`, n'existe plus) sont
/// volontairement absentes.
const EXPORTED_TABLES: &[&str] = &[
    "users",
    "lang",
    "formations",
    "projets",
    "experiences",
    "skills",
    "sessions",
    "candidatures",
    "status_history",
    "contacts",
    "interviews",
    "documents",
    "settings",
];

/// Convertit une ligne PostgreSQL en objet JSON, colonne par colonne.
///
/// Contrairement à SQLite (typage dynamique), le pilote PostgreSQL de sqlx
/// vérifie le type à chaque `try_get` : la chaîne d'essais doit donc couvrir
/// tous les types réellement présents dans les tables exportées (`BOOLEAN`,
/// `BIGINT`, `DATE`, `TIMESTAMPTZ`, `TEXT`), avec un repli base64 pour
/// d'éventuelles colonnes binaires.
///
/// # Arguments
///
/// * `row` - Ligne PostgreSQL à convertir.
///
/// # Returns
///
/// Un objet JSON où chaque colonne est décodée en essayant successivement
/// `bool`, `i64`, `f64`, `NaiveDate`, `DateTime<Utc>`, `String`, puis un
/// repli base64 pour les valeurs binaires (`null` si la colonne est
/// elle-même `NULL`).
fn row_to_json_map(row: &PgRow) -> Result<Map<String, Value>, AppError> {
    let mut map = Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        let value = if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<NaiveDate>, _>(i) {
            v.map(|d| Value::String(d.to_string())).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<DateTime<Utc>>, _>(i) {
            v.map(|d| Value::String(d.to_rfc3339())).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else {
            let bytes: Option<Vec<u8>> = row.try_get(i).map_err(map_db_err)?;
            bytes.map(|b| Value::String(BASE64.encode(b))).unwrap_or(Value::Null)
        };
        map.insert(column.name().to_string(), value);
    }
    Ok(map)
}

/// Exporte l'intégralité d'une table sous forme de tableau JSON.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `table` - Nom de la table à exporter, doit provenir de [`EXPORTED_TABLES`].
///
/// # Returns
///
/// Un `Value::Array` contenant une entrée par ligne de la table.
async fn export_table(pool: &PgPool, table: &str) -> Result<Value, AppError> {
    // `table` provient uniquement de EXPORTED_TABLES (littéraux internes),
    // jamais d'une entrée utilisateur : pas d'injection SQL possible ici.
    let sql = format!("SELECT * FROM {table}");
    let rows = sqlx::query(&sql).fetch_all(pool).await.map_err(map_db_err)?;
    let values = rows.iter().map(row_to_json_map).collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Array(values.into_iter().map(Value::Object).collect()))
}

/// Assemble l'export complet des données utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// Un objet JSON `{ "users": [...], "lang": [...], ... }`, une clé par
/// table listée dans [`EXPORTED_TABLES`]. Ne contient ni métadonnées
/// d'export (date, version applicative) ni contenu binaire de fichiers —
/// ces responsabilités appartiennent à la couche commande.
pub async fn export_all_data(pool: &PgPool) -> Result<Map<String, Value>, AppError> {
    let mut data = Map::new();
    for &table in EXPORTED_TABLES {
        data.insert(table.to_string(), export_table(pool, table).await?);
    }
    Ok(data)
}
