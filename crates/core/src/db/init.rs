//! Logique SQLite pure : initialisation du pool et application des
//! migrations. La résolution de chemins liée à `tauri::AppHandle` (base
//! SQLite, dossier de données, `DbState`) vit dans
//! `crates/desktop/src-tauri/src/db/init.rs`, qui appelle `init_db` ici une
//! fois le chemin résolu.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;

/// Initialise le pool SQLite et applique les migrations.
///
/// Les dossiers de documents (`documents/sessions/<session_id>/{cv,lettre_motivation}/...`)
/// sont propres à chaque session et créés à la volée par `documents::storage`,
/// puisque les sessions n'existent pas encore à ce stade de l'initialisation.
///
/// # Arguments
///
/// * `db_path` - Chemin absolu du fichier SQLite à ouvrir (ou créer).
///
/// # Returns
///
/// Pool de connexions prêt à l'emploi, migrations appliquées.
pub async fn init_db(
    db_path: &PathBuf,
) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(data_dir) = db_path.parent() {
        std::fs::create_dir_all(data_dir)?;
    }

    // Les migrations tournent avec les clés étrangères désactivées : plusieurs
    // d'entre elles recréent une table parente pour changer un CHECK ou une
    // contrainte NOT NULL (seule façon de faire en SQLite — voir
    // 0005_candidatures_add_interim.sql et 0014_status_history_timeline.sql).
    // Avec les clés étrangères actives, le DROP TABLE de la table parente
    // échoue dès que des tables enfants contiennent des lignes qui la
    // référencent ("FOREIGN KEY constraint failed").
    let migration_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        // sqlx active les clés étrangères par défaut (contrairement à SQLite
        // lui-même) : il faut les désactiver explicitement pour ce pool.
        .foreign_keys(false)
        .journal_mode(SqliteJournalMode::Wal);
    let migration_pool = SqlitePool::connect_with(migration_options).await?;
    // crates/core -> crates -> racine du repo : deux niveaux au-dessus du
    // CARGO_MANIFEST_DIR de ce crate.
    sqlx::migrate!("../../db/migrations/sqlite").run(&migration_pool).await?;
    migration_pool.close().await;

    // Le pool applicatif, lui, doit appliquer les clés étrangères pour toute
    // la durée de vie normale de l'application.
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    Ok(pool)
}

/// Liste les tables applicatives de la base (hors tables système et table de migrations).
///
/// Utilisé uniquement par la commande de diagnostic `db_status`.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Noms de tables triés alphabétiquement.
pub async fn list_tables(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master \
         WHERE type = 'table' \
           AND name NOT LIKE 'sqlite_%' \
           AND name != '_sqlx_migrations' \
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}
