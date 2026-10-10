//! Mode d'exécution (DEV ou PROD) et emplacement des données qui en découle.
//!
//! Règle : sans `.env`, ou avec un `.env` sans `MODE` ou avec `MODE=PROD`,
//! l'application utilise les dossiers du système (Tauri). Avec `MODE=DEV`,
//! tout ce qu'elle écrit (base, documents, polices, logs, profils WebView)
//! va dans `dev-data/`, à côté du `.env` qui a déclaré ce mode. Toute autre
//! valeur est une erreur de démarrage. La casse est ignorée.
//!
//! La décision est prise ici seulement ; le reste du code reçoit des
//! dossiers sans savoir d'où ils viennent.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Dossier des données du mode DEV, à côté du `.env`.
const DEV_DATA_DIR: &str = "dev-data";

/// Sous-dossier des logs en mode DEV.
const DEV_LOG_DIR: &str = "logs";

/// Sous-dossier du profil WebView de la fenêtre principale en mode DEV.
const DEV_MAIN_WEBVIEW_DIR: &str = "webview";

/// Sous-dossier du profil WebView de la fenêtre de capture : la session
/// (cookies, connexion) y est conservée entre deux utilisations, isolée de
/// la fenêtre principale.
const CAPTURE_PROFILE_DIR: &str = "capture_webview";

/// Mode d'exécution de l'application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Dev,
    Prod,
}

impl RunMode {
    /// Nom du mode, tel qu'écrit dans le `.env`.
    ///
    /// # Returns
    ///
    /// `"DEV"` ou `"PROD"`.
    pub fn as_str(self) -> &'static str {
        match self {
            RunMode::Dev => "DEV",
            RunMode::Prod => "PROD",
        }
    }
}

/// Dossiers fournis par le système (Tauri) pour l'application.
///
/// # Attributes
///
/// * `data` - Dossier de données (`app_data_dir`).
/// * `log` - Dossier de logs (`app_log_dir`).
/// * `local_data` - Dossier de données locales (`app_local_data_dir`).
#[derive(Debug, Clone)]
pub struct SystemDirs {
    pub data: PathBuf,
    pub log: PathBuf,
    pub local_data: PathBuf,
}

/// Mode retenu et dossiers où l'application écrit.
///
/// # Attributes
///
/// * `mode` - Mode d'exécution.
/// * `data_dir` - Dossier de données (base, documents, polices).
/// * `log_dir` - Dossier du fichier de log.
/// * `capture_profile_dir` - Profil WebView de la fenêtre de capture.
/// * `main_webview_dir` - Profil WebView de la fenêtre principale ; `None`
///   laisse l'emplacement par défaut de Tauri.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunLocation {
    pub mode: RunMode,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub capture_profile_dir: PathBuf,
    pub main_webview_dir: Option<PathBuf>,
}

/// Décide du mode d'exécution et des dossiers de l'application. Fonction
/// pure : ne lit ni l'environnement ni le disque.
///
/// # Arguments
///
/// * `env_file` - Chemin du `.env` chargé, ou `None` si aucun n'a été trouvé.
/// * `mode` - Valeur de la variable `MODE`, ou `None` si elle est absente.
/// * `system` - Dossiers fournis par le système.
///
/// # Returns
///
/// Le mode et ses dossiers, ou `AppError::Config` nommant la valeur reçue
/// et les deux valeurs acceptées.
pub fn resolve(env_file: Option<&Path>, mode: Option<&str>, system: SystemDirs) -> Result<RunLocation, AppError> {
    // Sans .env, MODE ne peut pas avoir été déclaré : mode PROD.
    let Some(env_file) = env_file else {
        return Ok(prod_location(system));
    };
    match parse_mode(env_file, mode)? {
        RunMode::Prod => Ok(prod_location(system)),
        RunMode::Dev => Ok(dev_location(env_file)),
    }
}

/// Crée les dossiers de données et de logs (et leurs parents, donc
/// `dev-data/` en mode DEV).
///
/// # Arguments
///
/// * `location` - Dossiers retenus par `resolve`.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Config` avec le chemin en cause.
pub fn ensure_dirs(location: &RunLocation) -> Result<(), AppError> {
    ensure_dir(&location.data_dir)?;
    ensure_dir(&location.log_dir)
}

/// Crée un dossier (et ses parents).
///
/// # Arguments
///
/// * `dir` - Dossier à créer.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Config` avec le chemin en cause.
pub fn ensure_dir(dir: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::Config(format!("Création du dossier {} impossible : {e}", dir.display())))
}

/// Interprète la valeur de `MODE` lue dans un `.env`.
fn parse_mode(env_file: &Path, mode: Option<&str>) -> Result<RunMode, AppError> {
    let Some(raw) = mode else {
        return Ok(RunMode::Prod);
    };
    match raw.trim().to_ascii_uppercase().as_str() {
        "DEV" => Ok(RunMode::Dev),
        "PROD" => Ok(RunMode::Prod),
        _ => Err(AppError::Config(format!(
            "Valeur de MODE invalide dans {} : « {raw} ». Valeurs acceptées : DEV ou PROD.",
            env_file.display()
        ))),
    }
}

/// Dossiers du mode PROD : ceux du système, sans changement.
fn prod_location(system: SystemDirs) -> RunLocation {
    RunLocation {
        mode: RunMode::Prod,
        data_dir: system.data,
        log_dir: system.log,
        capture_profile_dir: system.local_data.join(CAPTURE_PROFILE_DIR),
        main_webview_dir: None,
    }
}

/// Dossiers du mode DEV : tout sous `dev-data/`, à côté du `.env`.
fn dev_location(env_file: &Path) -> RunLocation {
    let root = env_file.parent().unwrap_or(Path::new("")).join(DEV_DATA_DIR);
    RunLocation {
        mode: RunMode::Dev,
        log_dir: root.join(DEV_LOG_DIR),
        capture_profile_dir: root.join(CAPTURE_PROFILE_DIR),
        main_webview_dir: Some(root.join(DEV_MAIN_WEBVIEW_DIR)),
        data_dir: root,
    }
}
