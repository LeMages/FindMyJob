//! Initialisation de la base de données SQLite et résolution des chemins
//! de données de l'application (base + dossiers de documents).

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
// N'est utilisé qu'en release, où resolve_db_path/resolve_data_dir appellent app.path().
#[cfg(not(debug_assertions))]
use tauri::Manager;

/// État partagé injecté dans toutes les commandes Tauri via `tauri::State`.
///
/// # Attributes
///
/// * `pool` - Pool de connexions SQLite partagé par toute l'application.
/// * `db_path` - Chemin absolu du fichier SQLite, exposé pour diagnostic (`db_status`).
pub struct DbState {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
}

/// Résout le chemin du fichier SQLite.
///
/// En debug, la base vit dans `dev-data/` à la racine du projet pour rester
/// visible et versionnable facilement pendant le développement. En release,
/// elle vit dans le dossier de données de l'application (jamais dans l'arbre projet).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri (utilisé uniquement en release).
///
/// # Returns
///
/// Chemin absolu vers `findmyjob.db`.
#[cfg_attr(debug_assertions, allow(unused_variables))]
pub fn resolve_db_path(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        // Le CARGO_MANIFEST_DIR d'un crate Tauri a toujours un parent (le dossier projet) :
        // ce unwrap ne peut échouer que si le crate est placé à la racine du disque.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dev-data")
            .join("findmyjob.db")
    }
    #[cfg(not(debug_assertions))]
    {
        app.path()
            .app_data_dir()
            .expect("failed to resolve app data dir")
            .join("findmyjob.db")
    }
}

/// Résout le dossier de données de l'application (contient la base et les documents).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri (utilisé uniquement en release).
///
/// # Returns
///
/// Chemin absolu vers le dossier de données.
#[cfg_attr(debug_assertions, allow(unused_variables))]
pub fn resolve_data_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        // Voir resolve_db_path : même garantie sur le parent.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dev-data")
    }
    #[cfg(not(debug_assertions))]
    {
        app.path()
            .app_data_dir()
            .expect("failed to resolve app data dir")
    }
}

/// Initialise le pool SQLite et applique les migrations.
///
/// Crée également l'arborescence de dossiers `documents/{cv,lettre_motivation}/{backup,artefact}`
/// nécessaire au stockage des documents de référence.
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
        for doc_type in ["cv", "lettre_motivation"] {
            let dir = data_dir.join("documents").join(doc_type);
            std::fs::create_dir_all(dir.join("backup"))?;
            std::fs::create_dir_all(dir.join("artefact"))?;
        }
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!("../db/migrations").run(&pool).await?;

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
