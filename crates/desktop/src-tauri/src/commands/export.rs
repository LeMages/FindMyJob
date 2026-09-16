//! Commande d'export complet des données applicatives en un fichier JSON,
//! déclenchée depuis les Paramètres ("Sauvegarde / Export").

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use chrono::Local;
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;

/// Exporte l'intégralité des données utilisateur dans un fichier JSON choisi
/// via une boîte de dialogue native. Les données elles-mêmes sont récupérées
/// auprès de l'agent (`GET /api/export`).
///
/// # Arguments
///
/// * `app` - Handle de l'application (métadonnées de version, dialogue natif).
/// * `api` - Client HTTP vers l'agent.
///
/// # Returns
///
/// Le chemin absolu du fichier écrit, ou `None` si l'utilisateur a annulé
/// la boîte de dialogue (ce n'est pas une erreur).
#[tauri::command]
pub async fn export_data_json(
    app: AppHandle,
    api: tauri::State<'_, ApiClient>,
) -> Result<Option<String>, AppError> {
    // ------------------------------------------------------------------
    // 1. Récupère les données auprès de l'agent et assemble les métadonnées
    //    d'export
    // ------------------------------------------------------------------
    let data: serde_json::Value = api.get("/api/export").await?;
    let payload = json!({
        "export_date": Local::now().to_rfc3339(),
        "app_version": app.package_info().version.to_string(),
        "data": data,
    });
    let pretty = serde_json::to_string_pretty(&payload)
        .map_err(|e| AppError::Validation(format!("Erreur de sérialisation JSON : {e}")))?;

    // ------------------------------------------------------------------
    // 2. Boîte de dialogue native (bloquante -> thread dédié pour ne pas
    //    bloquer le runtime async)
    // ------------------------------------------------------------------
    let default_name = format!("findmyjob_export_{}.json", Local::now().format("%Y-%m-%d"));
    let app_for_dialog = app.clone();
    let chosen = tauri::async_runtime::spawn_blocking(move || {
        app_for_dialog
            .dialog()
            .file()
            .add_filter("JSON", &["json"])
            .set_file_name(&default_name)
            .blocking_save_file()
    })
    .await
    .map_err(|e| AppError::Validation(format!("Erreur interne : {e}")))?;

    let Some(file_path) = chosen else {
        return Ok(None); // Annulation utilisateur : cas normal, pas une erreur.
    };

    // ------------------------------------------------------------------
    // 3. Écriture du fichier
    // ------------------------------------------------------------------
    let path: PathBuf = file_path
        .into_path()
        .map_err(|e| AppError::Validation(format!("Chemin de fichier invalide : {e}")))?;
    std::fs::write(&path, pretty)
        .map_err(|e| AppError::Validation(format!("Écriture du fichier impossible : {e}")))?;

    Ok(Some(path.to_string_lossy().to_string()))
}
