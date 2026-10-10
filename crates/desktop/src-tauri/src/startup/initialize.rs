//! Initialisation de l'application au démarrage : traces, `.env`, dossiers
//! de l'application, base SQLite et migrations, nettoyage des générations
//! interrompues.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use tauri::{AppHandle, Manager};

// Importation des modules internes
use crate::app_state::{AppContext, AppState};
use crate::db;
use crate::progress::ProgressStore;
use crate::services::generation;
use crate::startup::env_file;
use crate::storage_paths;
use findmyjob_core::error::AppError;
use findmyjob_core::trace_log;

/// Nom du fichier de traces, dans le dossier de logs de l'application.
const LOG_FILE: &str = "findmyjob.log";

/// Crée un dossier (et ses parents).
///
/// # Arguments
///
/// * `dir` - Dossier à créer.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Config` avec le chemin en cause.
fn ensure_dir(dir: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::Config(format!("Création du dossier {} impossible : {e}", dir.display())))
}

/// Oriente les traces vers le fichier de log de l'application. Un échec
/// laisse les traces sur la seule sortie d'erreur : il n'empêche pas de
/// démarrer.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
fn setup_log_file(app: &AppHandle) {
    let log_dir = match app.path().app_log_dir() {
        Ok(dir) => dir,
        Err(e) => return trace_log::log(&format!("[démarrage] dossier de logs introuvable : {e}")),
    };
    if let Err(e) = ensure_dir(&log_dir) {
        return trace_log::log(&format!("[démarrage] {e}"));
    }
    if let Err(previous) = trace_log::set_log_file(log_dir.join(LOG_FILE)) {
        trace_log::log(&format!("[démarrage] fichier de log déjà configuré : {}", previous.display()));
    }
}

/// Construit l'état de l'application.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
///
/// # Returns
///
/// `AppState` prêt, ou l'erreur d'ouverture de la base.
async fn build_state(data_dir: PathBuf) -> Result<AppState, AppError> {
    // ------------------------------------------------------------------
    // 1. Dossiers de données et des polices
    // ------------------------------------------------------------------
    let fonts_dir = storage_paths::fonts_dir(&data_dir);
    ensure_dir(&data_dir)?;
    ensure_dir(&fonts_dir)?;

    // ------------------------------------------------------------------
    // 2. Base et migrations
    // ------------------------------------------------------------------
    let pool = db::init::open_pool(&storage_paths::db_file(&data_dir)).await?;

    // ------------------------------------------------------------------
    // 3. Générations coupées par une fermeture de l'application : la
    //    candidature et ses fichiers partiels sont supprimés
    // ------------------------------------------------------------------
    match generation::clean_interrupted(&pool, &data_dir).await {
        Ok(0) => {}
        Ok(n) => trace_log::log(&format!("[démarrage] {n} génération(s) interrompue(s) nettoyée(s)")),
        Err(e) => trace_log::log(&format!("[démarrage] nettoyage des générations interrompues impossible : {e}")),
    }

    Ok(AppState { pool, data_dir, fonts_dir, progress: ProgressStore::new() })
}

/// Initialise l'application : traces, `.env`, dossiers, base. Appelée une
/// fois depuis `setup`, avant toute commande.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// `AppContext` prêt, ou en échec avec la cause (affichée par l'interface).
pub fn initialize(app: &AppHandle) -> AppContext {
    setup_log_file(app);
    env_file::load(app.path().app_config_dir().ok().as_deref());

    let result = match app.path().app_data_dir() {
        Ok(data_dir) => tauri::async_runtime::block_on(build_state(data_dir)),
        Err(e) => Err(AppError::Config(format!("Dossier de données de l'application introuvable : {e}"))),
    };
    match result {
        Ok(state) => AppContext::ready(state),
        Err(e) => {
            trace_log::log(&format!("[démarrage] initialisation impossible : {e}"));
            AppContext::failed(e.to_string())
        }
    }
}
