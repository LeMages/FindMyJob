//! Export complet des données utilisateur au format JSON (sauvegarde
//! manuelle). Conversion générique ligne SQL -> JSON : `SELECT * FROM
//! <table>` sur des tables hétérogènes ne peut pas raisonnablement passer
//! par une struct typée par table sans duplication massive.

// Importation des crates externes
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Column, Row, SqlitePool};

// Importation des modules internes
use crate::error::AppError;

/// Tables applicatives incluses dans l'export, dans l'ordre d'apparition
/// dans le JSON produit.
///
/// `embeddings` (données techniques binaires) et `email_attachments` (table
/// supprimée par la migration `0009_refactor_documents.sql`, n'existe plus)
/// sont volontairement absentes.
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
    "emails",
    "settings",
];

/// Convertit une ligne SQLite en objet JSON, colonne par colonne.
///
/// # Arguments
///
/// * `row` - Ligne SQLite à convertir.
///
/// # Returns
///
/// Un objet JSON où chaque colonne est décodée en essayant successivement
/// `i64`, `f64`, `String`, puis un repli base64 pour les valeurs binaires
/// (`null` si la colonne est elle-même `NULL`).
fn row_to_json_map(row: &SqliteRow) -> Result<Map<String, Value>, AppError> {
    let mut map = Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        let value = if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else {
            let bytes: Option<Vec<u8>> = row.try_get(i).map_err(AppError::from)?;
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
/// * `pool` - Pool de connexions SQLite.
/// * `table` - Nom de la table à exporter, doit provenir de [`EXPORTED_TABLES`].
///
/// # Returns
///
/// Un `Value::Array` contenant une entrée par ligne de la table.
async fn export_table(pool: &SqlitePool, table: &str) -> Result<Value, AppError> {
    // `table` provient uniquement de EXPORTED_TABLES (littéraux internes),
    // jamais d'une entrée utilisateur : pas d'injection SQL possible ici.
    let sql = format!("SELECT * FROM {table}");
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    let values = rows.iter().map(row_to_json_map).collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Array(values.into_iter().map(Value::Object).collect()))
}

/// Assemble l'export complet des données utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Un objet JSON `{ "users": [...], "lang": [...], ... }`, une clé par
/// table listée dans [`EXPORTED_TABLES`]. Ne contient ni métadonnées
/// d'export (date, version applicative) ni contenu binaire de fichiers —
/// ces responsabilités appartiennent à la couche commande.
pub async fn export_all_data(pool: &SqlitePool) -> Result<Map<String, Value>, AppError> {
    let mut data = Map::new();
    for &table in EXPORTED_TABLES {
        data.insert(table.to_string(), export_table(pool, table).await?);
    }
    Ok(data)
}
