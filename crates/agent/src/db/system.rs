//! Diagnostic de la connexion PostgreSQL : nom de la base courante et tables
//! applicatives présentes (utilisé par l'endpoint `/api/system/db-status`).

// Importation des crates externes
use sqlx::PgPool;

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;

/// Récupère le nom de la base de données courante et la liste des tables du
/// schéma `public`, triées par nom.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// `(nom_de_la_base, tables)`.
pub async fn status(pool: &PgPool) -> Result<(String, Vec<String>), AppError> {
    let database: String =
        sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await.map_err(map_db_err)?;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    Ok((database, tables))
}
