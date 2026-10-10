//! Point d'entrée de la bibliothèque Tauri : déclaration des modules et
//! configuration de l'application (initialisation au démarrage, état
//! partagé, commandes exposées à l'IPC).

// Importation des modules internes
pub mod app_state;
pub mod clock;
pub mod commands;
pub mod db;
pub mod docgen;
pub mod documents;
pub mod error;
pub mod models;
pub mod offer_extraction;
pub mod pipeline;
pub mod progress;
pub mod services;
pub mod startup;
pub mod storage_paths;
pub mod url_normalize;

// Importation des crates externes
use tauri::Manager;

/// Configure et lance l'application Tauri : initialise l'état (`.env`, mode
/// d'exécution, traces, dossiers, base SQLite), l'injecte comme état
/// partagé, crée la fenêtre principale, puis enregistre toutes les
/// commandes IPC.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let startup = startup::initialize(app.handle());
            app.manage(startup.context);
            startup::main_window::create(app.handle(), startup.main_webview_dir.as_deref())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            commands::get_documents_status,
            commands::get_document_versions,
            commands::get_document_version_pdf,
            commands::restore_document_version,
            commands::open_url,
            commands::update_user_info,
            commands::add_projet,
            commands::delete_projet,
            commands::update_projet,
            commands::add_skill,
            commands::delete_skill,
            commands::update_skill,
            commands::create_candidature,
            commands::find_candidatures_by_url,
            commands::scrape_offer,
            commands::extract_offer_from_text,
            commands::open_capture_window,
            commands::capture_offer_page,
            commands::close_capture_window,
            commands::get_candidatures,
            commands::get_candidature,
            commands::get_generation_progress,
            commands::update_candidature,
            commands::delete_candidature,
            commands::postuler_sans_message,
            commands::postuler_par_mail,
            commands::postuler_par_message,
            commands::get_contacts,
            commands::get_status_history,
            commands::get_available_events,
            commands::add_timeline_event,
            commands::get_pending_interviews,
            commands::get_all_interviews_for_session,
            commands::get_document_info,
            commands::get_document_pdf,
            commands::download_document,
            commands::get_downloads_dir,
            commands::save_text_file,
            commands::create_session,
            commands::get_sessions,
            commands::update_session,
            commands::delete_session,
            commands::get_llm_model,
            commands::set_llm_model,
            commands::get_theme,
            commands::set_theme,
            commands::test_api_connection,
            commands::list_fonts,
            commands::import_font_folder,
            commands::delete_font,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
