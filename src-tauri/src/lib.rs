//! Point d'entrée de la bibliothèque Tauri : déclaration des modules et
//! configuration de l'application (état partagé, commandes exposées à l'IPC).

// Importation des modules internes
pub mod commands;
pub mod db;
pub mod documents;
pub mod error;
pub mod llm;
pub mod models;

// Importation des crates externes
use tauri::Manager;

// Importation des modules internes (usage local à run())
use db::init::{init_db, resolve_db_path, DbState};

/// Configure et lance l'application Tauri : initialise la base de données,
/// l'injecte comme état partagé, puis enregistre toutes les commandes IPC .
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Charge le .env à la racine du projet (ANTHROPIC_API_KEY, etc.) — no-op
    // silencieux si le fichier est absent (ex. build de production packagée).
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = resolve_db_path(app.handle());
            let pool = tauri::async_runtime::block_on(init_db(&db_path))
                .map_err(|e| e.to_string())?;
            app.manage(DbState { pool, db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::db_status,
            commands::user_exists,
            commands::create_user,
            commands::get_user,
            commands::get_profile,
            commands::update_description,
            commands::add_experience,
            commands::delete_experience,
            commands::update_experience,
            commands::add_formation,
            commands::delete_formation,
            commands::update_formation,
            commands::add_lang,
            commands::delete_lang,
            commands::update_lang,
            commands::get_cv_reference,
            commands::get_lettre_motivation_reference,
            commands::save_cv,
            commands::save_lettre_motivation,
            commands::compile_cv,
            commands::compile_lettre_motivation,
            commands::open_url,
            commands::update_user_info,
            commands::add_projet,
            commands::delete_projet,
            commands::update_projet,
            commands::add_skill,
            commands::delete_skill,
            commands::update_skill,
            commands::create_candidature,
            commands::get_candidatures,
            commands::get_candidature,
            commands::update_candidature,
            commands::abandon_candidature,
            commands::postuler_sans_message,
            commands::postuler_par_mail,
            commands::postuler_par_message,
            commands::get_contacts,
            commands::get_status_history,
            commands::get_available_events,
            commands::add_timeline_event,
            commands::get_pending_interviews,
            commands::get_document_info,
            commands::get_document_pdf,
            commands::open_document,
            commands::upload_candidature_cv,
            commands::upload_candidature_lettre_motivation,
            commands::get_cv_reference_dir,
            commands::get_lettre_motivation_reference_dir,
            commands::select_candidature_cv,
            commands::select_candidature_lettre_motivation,
            commands::create_session,
            commands::get_sessions,
            commands::update_session,
            commands::delete_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
