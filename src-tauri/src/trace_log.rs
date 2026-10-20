//! Traces de diagnostic : écrites sur la sortie d'erreur et, une fois le
//! fichier configuré par l'application, ajoutées à un fichier de log.
//!
//! En release Windows, l'application n'a pas de console
//! (`windows_subsystem = "windows"`) : sans fichier, ces traces seraient
//! perdues. Le chemin est fourni par l'application au démarrage ; `core` ne
//! connaît ni Tauri ni le dossier de logs.

// Importation des modules standard
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// Importation des modules internes
use crate::clock;

/// Fichier de log, fixé une seule fois au démarrage.
static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

/// Sérialise les écritures concurrentes dans le fichier (tâches de fond).
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Configure le fichier qui reçoit les traces.
///
/// # Arguments
///
/// * `path` - Chemin du fichier de log (créé à la première trace).
///
/// # Returns
///
/// `Ok(())`, ou `Err(chemin déjà configuré)` si la fonction a déjà été appelée.
pub fn set_log_file(path: PathBuf) -> Result<(), PathBuf> {
    LOG_FILE.set(path)
}

/// Émet une trace de diagnostic : sortie d'erreur, puis fichier de log s'il
/// est configuré. Un échec d'écriture du fichier est signalé sur la sortie
/// d'erreur, jamais propagé : une trace ne doit pas faire échouer l'opération
/// qu'elle décrit.
///
/// # Arguments
///
/// * `message` - Texte de la trace, sur une ligne.
pub fn log(message: &str) {
    eprintln!("{message}");
    let Some(path) = LOG_FILE.get() else {
        return;
    };
    let line = format!("{} {message}\n", clock::now_utc());
    // Un verrou empoisonné ne protège qu'un `()` : l'écriture peut continuer.
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let written = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()));
    if let Err(e) = written {
        eprintln!("Écriture du log {} impossible : {e}", path.display());
    }
}
