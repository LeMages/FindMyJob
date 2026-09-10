//! Résolution de chemins liée à `tauri::AppHandle` (base SQLite, dossier de
//! données de l'application) et état partagé `tauri::State`. La logique
//! SQLite pure (pool, migrations) vit dans `findmyjob_core::db::init`.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
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
        // Le CARGO_MANIFEST_DIR de ce crate est crates/desktop/src-tauri : trois
        // parents remontent jusqu'à la racine du repo (crates/desktop/src-tauri
        // -> crates/desktop -> crates -> racine). Ce unwrap ne peut échouer que
        // si le crate est placé à la racine du disque.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
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
            .parent()
            .unwrap()
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
