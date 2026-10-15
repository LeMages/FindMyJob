//! Tests de la décision du mode d'exécution (DEV ou PROD) et de la création
//! de ses dossiers.

// Importation des modules standard
use std::fs;
use std::path::{Path, PathBuf};

// Importation des modules internes
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::error::AppError;
use findmyjob_lib::startup::run_mode::{self, RunLocation, RunMode, SystemDirs};

/// Dossiers système fictifs.
fn system() -> SystemDirs {
    SystemDirs {
        data: PathBuf::from("/sys/data"),
        log: PathBuf::from("/sys/log"),
        local_data: PathBuf::from("/sys/local"),
    }
}

/// Emplacement attendu en mode PROD : les dossiers système, inchangés.
fn prod() -> RunLocation {
    RunLocation {
        mode: RunMode::Prod,
        data_dir: PathBuf::from("/sys/data"),
        log_dir: PathBuf::from("/sys/log"),
        capture_profile_dir: PathBuf::from("/sys/local").join("capture_webview"),
        main_webview_dir: None,
    }
}

/// Chemin d'un `.env` fictif à la racine d'un projet.
fn env_file() -> PathBuf {
    PathBuf::from("/projet").join(".env")
}

/// Résout avec le `.env` fictif et la valeur de `MODE` donnée.
fn resolve_with(mode: Option<&str>) -> Result<RunLocation, AppError> {
    run_mode::resolve(Some(&env_file()), mode, system())
}

#[test]
fn sans_env_le_mode_est_prod() {
    assert_eq!(run_mode::resolve(None, None, system()).unwrap(), prod());
    // Sans .env, une valeur venue d'ailleurs n'est pas prise en compte.
    assert_eq!(run_mode::resolve(None, Some("DEV"), system()).unwrap(), prod());
}

#[test]
fn mode_absent_donne_prod() {
    assert_eq!(resolve_with(None).unwrap(), prod());
}

#[test]
fn mode_prod_donne_les_dossiers_systeme() {
    assert_eq!(resolve_with(Some("PROD")).unwrap(), prod());
}

#[test]
fn mode_dev_met_tout_dans_dev_data_a_cote_du_env() {
    let root = Path::new("/projet").join("dev-data");
    let expected = RunLocation {
        mode: RunMode::Dev,
        data_dir: root.clone(),
        log_dir: root.join("logs"),
        capture_profile_dir: root.join("capture_webview"),
        main_webview_dir: Some(root.join("webview")),
    };
    assert_eq!(resolve_with(Some("DEV")).unwrap(), expected);
}

#[test]
fn la_casse_est_ignoree() {
    assert_eq!(resolve_with(Some("dEv")).unwrap().mode, RunMode::Dev);
    assert_eq!(resolve_with(Some("prod")).unwrap(), prod());
    assert_eq!(resolve_with(Some(" Dev ")).unwrap().mode, RunMode::Dev);
}

#[test]
fn une_valeur_invalide_est_une_erreur_explicite() {
    match resolve_with(Some("staging")) {
        Err(AppError::Config(message)) => {
            assert!(message.contains("staging"), "{message}");
            assert!(message.contains("DEV") && message.contains("PROD"), "{message}");
        }
        other => panic!("erreur de configuration attendue, obtenu {other:?}"),
    }
    assert!(resolve_with(Some("")).is_err());
}

#[test]
fn ensure_dirs_cree_dev_data_et_ses_logs() {
    let project = unique_temp_dir("run-mode");

    let location = run_mode::resolve(Some(&project.join(".env")), Some("DEV"), system()).unwrap();
    assert!(!location.data_dir.exists());
    run_mode::ensure_dirs(&location).unwrap();
    assert!(project.join("dev-data").is_dir());
    assert!(project.join("dev-data").join("logs").is_dir());
    // Déjà présents : pas d'erreur.
    run_mode::ensure_dirs(&location).unwrap();

    fs::remove_dir_all(&project).unwrap();
}
