//! Ouverture du pool SQLite et application des migrations embarquées.

// Importation des modules standard
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

// Importation des crates externes
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;

/// Délai d'attente d'un verrou d'écriture avant `SQLITE_BUSY` : la tâche de
/// génération et l'interface peuvent écrire en même temps.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Nombre maximal de connexions du pool (lectures concurrentes en WAL ; un
/// seul écrivain à la fois de toute façon).
const MAX_CONNECTIONS: u32 = 5;

/// Applique les réglages communs à toute connexion : clés étrangères
/// (désactivées par défaut dans SQLite) et attente sur verrou.
///
/// # Arguments
///
/// * `options` - Options de connexion de départ (fichier ou mémoire).
///
/// # Returns
///
/// Options complétées.
fn with_common_pragmas(options: SqliteConnectOptions) -> SqliteConnectOptions {
    options.foreign_keys(true).busy_timeout(BUSY_TIMEOUT)
}

/// Applique les migrations embarquées dans le binaire.
///
/// # Arguments
///
/// * `pool` - Pool fraîchement ouvert.
///
/// # Returns
///
/// `Ok(())` une fois le schéma à jour, `AppError::Database` sinon.
async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    // crates/desktop/src-tauri -> racine du dépôt : trois niveaux au-dessus
    // du CARGO_MANIFEST_DIR de ce crate.
    sqlx::migrate!("../../../db/migrations/sqlite")
        .run(pool)
        .await
        .map_err(|e| AppError::Database(format!("Migration de la base locale impossible : {e}")))
}

/// Ouvre (en la créant au besoin) la base de l'application et applique les
/// migrations en attente.
///
/// # Arguments
///
/// * `db_path` - Chemin du fichier `findmyjob.db`.
///
/// # Returns
///
/// Pool prêt à l'emploi (WAL, clés étrangères, `busy_timeout` de 5 s).
pub async fn open_pool(db_path: &Path) -> Result<SqlitePool, AppError> {
    let options = with_common_pragmas(
        SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal),
    );
    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect_with(options)
        .await
        .map_err(|e| {
            AppError::Database(format!("Ouverture de la base {} impossible : {e}", db_path.display()))
        })?;
    run_migrations(&pool).await?;
    Ok(pool)
}

/// Ouvre une base en mémoire, migrée, pour les tests.
///
/// Une base `:memory:` est propre à chaque connexion : le pool est limité à
/// une connexion, jamais recyclée, pour que toutes les requêtes voient la
/// même base.
///
/// # Returns
///
/// Pool à connexion unique sur une base vide migrée.
pub async fn open_memory_pool() -> Result<SqlitePool, AppError> {
    let options = with_common_pragmas(SqliteConnectOptions::from_str("sqlite::memory:").map_err(map_db_err)?);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options)
        .await
        .map_err(map_db_err)?;
    run_migrations(&pool).await?;
    Ok(pool)
}
