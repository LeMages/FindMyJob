//! Ouverture des transactions d'écriture.
//!
//! SQLite n'a qu'un écrivain. Une transaction ouverte par `BEGIN` simple ne
//! prend le verrou d'écriture qu'à sa première écriture : une transaction
//! « lecture puis écriture » peut alors échouer en `SQLITE_BUSY` sans attendre.
//! `BEGIN IMMEDIATE` prend le verrou dès l'ouverture (en respectant le
//! `busy_timeout`) et remplace le `SELECT … FOR UPDATE` de PostgreSQL.

// Importation des crates externes
use sqlx::{Sqlite, SqlitePool, Transaction};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;

/// Ouvre une transaction d'écriture en mode immédiat.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Transaction active, à valider par `commit()` (annulée si abandonnée).
pub async fn begin_write(pool: &SqlitePool) -> Result<Transaction<'static, Sqlite>, AppError> {
    pool.begin_with("BEGIN IMMEDIATE").await.map_err(map_db_err)
}
