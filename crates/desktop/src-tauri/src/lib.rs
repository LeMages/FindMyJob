//! Point d'entrée de la bibliothèque Tauri : déclaration des modules et
//! configuration de l'application (client API partagé, commandes exposées à l'IPC).

// Importation des modules internes
pub mod api_client;
pub mod commands;
pub mod error;
pub mod models;

// Importation des crates externes
use api_client::ApiClient;

/// Configure et lance l'application Tauri : construit le client HTTP vers
/// l'agent, l'injecte comme état partagé, puis enregistre toutes les
/// commandes IPC.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Charge le .env à la racine du projet (API_URL, API_TOKEN, etc.) — no-op
    // silencieux si le fichier est absent (ex. build de production packagée).
    dotenvy::dotenv().ok();

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let api_token = std::env::var("API_TOKEN").ok().filter(|t| !t.is_empty());
    let client = ApiClient::new(&api_url, api_token);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(client)
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
            commands::scrape_offer,
            commands::extract_offer_from_text,
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
            commands::get_all_interviews_for_session,
            commands::get_document_info,
            commands::get_document_pdf,
            commands::create_session,
            commands::get_sessions,
            commands::update_session,
            commands::delete_session,
            commands::get_search_queries,
            commands::create_search_query,
            commands::delete_search_query,
            commands::run_session_search,
            commands::get_session_offers,
            commands::get_api_key_status,
            commands::set_api_key,
            commands::delete_api_key,
            commands::get_llm_model,
            commands::set_llm_model,
            commands::get_theme,
            commands::set_theme,
            commands::get_documents_path,
            commands::open_folder,
            commands::test_api_connection,
            commands::export_data_json,
            commands::list_fonts,
            commands::import_font_folder,
            commands::delete_font,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
