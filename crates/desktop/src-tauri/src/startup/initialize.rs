//! Initialisation de l'application au démarrage : `.env`, mode d'exécution,
//! traces, dossiers de l'application, base SQLite et migrations, nettoyage
//! des générations interrompues.

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
use crate::startup::run_mode::{self, RunLocation, SystemDirs};
use crate::storage_paths;
use findmyjob_core::error::AppError;
use findmyjob_core::trace_log;

/// Nom de la variable du `.env` qui porte le mode d'exécution.
const MODE_VAR: &str = "MODE";

/// Nom du fichier de traces, dans le dossier de logs de l'application.
const LOG_FILE: &str = "findmyjob.log";

/// Résultat du démarrage, consommé par `setup`.
///
/// # Attributes
///
/// * `context` - État de l'application, ou cause de l'échec d'initialisation.
/// * `main_webview_dir` - Profil WebView de la fenêtre principale ; `None`
///   laisse l'emplacement par défaut de Tauri.
pub struct Startup {
    pub context: AppContext,
    pub main_webview_dir: Option<PathBuf>,
}

/// Oriente les traces vers le fichier de log de l'application. Un échec
/// laisse les traces sur la seule sortie d'erreur : il n'empêche pas de
/// démarrer.
///
/// # Arguments
///
/// * `log_dir` - Dossier de logs retenu pour le mode d'exécution (déjà créé).
fn setup_log_file(log_dir: &Path) {
    if let Err(previous) = trace_log::set_log_file(log_dir.join(LOG_FILE)) {
        trace_log::log(&format!("[démarrage] fichier de log déjà configuré : {}", previous.display()));
    }
}

/// Lit la variable `MODE`. Une valeur non UTF-8 est transmise telle quelle
/// (avec remplacement) pour être refusée avec un message explicite.
///
/// # Returns
///
/// La valeur, ou `None` si la variable est absente.
fn read_mode_var() -> Option<String> {
    match std::env::var(MODE_VAR) {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(raw)) => Some(raw.to_string_lossy().into_owned()),
    }
}

/// Dossiers fournis par Tauri pour l'application.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// Les dossiers, ou `AppError::Config` si l'un d'eux est introuvable.
fn system_dirs(app: &AppHandle) -> Result<SystemDirs, AppError> {
    let missing = |what: &str, e: tauri::Error| AppError::Config(format!("Dossier {what} de l'application introuvable : {e}"));
    Ok(SystemDirs {
        data: app.path().app_data_dir().map_err(|e| missing("de données", e))?,
        log: app.path().app_log_dir().map_err(|e| missing("de logs", e))?,
        local_data: app.path().app_local_data_dir().map_err(|e| missing("de données locales", e))?,
    })
}

/// Décide du mode d'exécution, crée ses dossiers et y oriente les traces.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `env_file` - Chemin du `.env` chargé, ou `None`.
///
/// # Returns
///
/// Les dossiers retenus, ou l'erreur de décision ou de création.
fn locate(app: &AppHandle, env_file: Option<&Path>) -> Result<RunLocation, AppError> {
    let location = run_mode::resolve(env_file, read_mode_var().as_deref(), system_dirs(app)?)?;
    run_mode::ensure_dirs(&location)?;
    setup_log_file(&location.log_dir);
    trace_log::log(&format!(
        "[démarrage] mode {} — données : {} — logs : {}",
        location.mode.as_str(),
        location.data_dir.display(),
        location.log_dir.display()
    ));
    Ok(location)
}

/// Construit l'état de l'application.
///
/// # Arguments
///
/// * `location` - Dossiers retenus pour le mode d'exécution.
///
/// # Returns
///
/// `AppState` prêt, ou l'erreur d'ouverture de la base.
async fn build_state(location: RunLocation) -> Result<AppState, AppError> {
    let data_dir = location.data_dir;

    // ------------------------------------------------------------------
    // 1. Dossier des polices (le dossier de données est déjà créé)
    // ------------------------------------------------------------------
    let fonts_dir = storage_paths::fonts_dir(&data_dir);
    run_mode::ensure_dir(&fonts_dir)?;

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

    Ok(AppState {
        pool,
        data_dir,
        fonts_dir,
        capture_profile_dir: location.capture_profile_dir,
        progress: ProgressStore::new(),
    })
}

/// Initialise l'application : `.env`, mode d'exécution, traces, dossiers,
/// base. Appelée une fois depuis `setup`, avant toute commande.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// `Startup` : contexte prêt ou en échec avec la cause (affichée par
/// l'interface), et profil WebView de la fenêtre principale.
pub fn initialize(app: &AppHandle) -> Startup {
    let env_file = env_file::load(app.path().app_config_dir().ok().as_deref());

    // Sans dossiers résolus, la fenêtre principale garde l'emplacement par
    // défaut ; une fois résolus, son profil suit le mode même si la base échoue.
    let location = match locate(app, env_file.as_deref()) {
        Ok(location) => location,
        Err(e) => return failed(e, None),
    };
    let main_webview_dir = location.main_webview_dir.clone();
    match tauri::async_runtime::block_on(build_state(location)) {
        Ok(state) => Startup { context: AppContext::ready(state), main_webview_dir },
        Err(e) => failed(e, main_webview_dir),
    }
}

/// Démarrage en échec : la cause est tracée et portée par le contexte.
fn failed(e: AppError, main_webview_dir: Option<PathBuf>) -> Startup {
    trace_log::log(&format!("[démarrage] initialisation impossible : {e}"));
    Startup { context: AppContext::failed(e.to_string()), main_webview_dir }
}
