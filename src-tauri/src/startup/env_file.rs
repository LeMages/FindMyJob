//! Chargement du fichier `.env` qui porte la clé API Anthropic.
//!
//! Ordre de recherche : le dossier de configuration de l'application (app
//! installée), puis le répertoire courant et ses parents (développement,
//! `.env` à la racine du dépôt). Les variables déjà définies dans
//! l'environnement ne sont jamais écrasées.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des modules internes
use crate::trace_log;

/// Nom du fichier recherché.
const ENV_FILE: &str = ".env";

/// Charge le premier `.env` trouvé. L'absence de fichier n'est pas une
/// erreur : les fonctions IA signaleront l'absence de clé à l'usage.
///
/// # Arguments
///
/// * `config_dir` - Dossier de configuration de l'application, s'il a pu être déterminé.
///
/// # Returns
///
/// Chemin du `.env` chargé, ou `None` si aucun n'a été trouvé ou s'il est illisible.
pub fn load(config_dir: Option<&Path>) -> Option<PathBuf> {
    // ------------------------------------------------------------------
    // 1. Dossier de configuration de l'application
    // ------------------------------------------------------------------
    if let Some(path) = config_dir.map(|dir| dir.join(ENV_FILE)).filter(|path| path.is_file()) {
        match dotenvy::from_path(&path) {
            Ok(()) => {
                trace_log::log(&format!("[démarrage] .env chargé : {}", path.display()));
                return Some(path);
            }
            Err(e) => trace_log::log(&format!("[démarrage] .env illisible {} : {e}", path.display())),
        }
    }

    // ------------------------------------------------------------------
    // 2. Répertoire courant et ses parents
    // ------------------------------------------------------------------
    match dotenvy::dotenv() {
        Ok(path) => {
            trace_log::log(&format!("[démarrage] .env chargé : {}", path.display()));
            Some(path)
        }
        Err(e) if e.not_found() => {
            trace_log::log("[démarrage] aucun fichier .env trouvé");
            None
        }
        Err(e) => {
            trace_log::log(&format!("[démarrage] .env illisible : {e}"));
            None
        }
    }
}
