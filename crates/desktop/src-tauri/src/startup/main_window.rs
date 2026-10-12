//! Création de la fenêtre principale. Elle est déclarée dans
//! `tauri.conf.json` avec `"create": false` et créée ici, au démarrage, pour
//! que son profil WebView suive le mode d'exécution (en DEV, sous
//! `dev-data/`).

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use tauri::{AppHandle, WebviewWindowBuilder};

/// Label de la fenêtre principale dans `tauri.conf.json`.
const MAIN_LABEL: &str = "main";

/// Crée la fenêtre principale à partir de sa configuration.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `webview_dir` - Profil WebView à utiliser ; `None` laisse
///   l'emplacement par défaut de Tauri.
///
/// # Returns
///
/// `Ok(())`, ou l'erreur de configuration ou de création de la fenêtre.
pub fn create(app: &AppHandle, webview_dir: Option<&Path>) -> tauri::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == MAIN_LABEL)
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    let builder = WebviewWindowBuilder::from_config(app, config)?;
    let builder = match webview_dir {
        Some(dir) => builder.data_directory(dir.to_path_buf()),
        None => builder,
    };
    builder.build()?;
    Ok(())
}
