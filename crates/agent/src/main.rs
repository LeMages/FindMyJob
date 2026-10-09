//! Agent FindMyJob (service headless) : serveur API REST (Axum) qui porte
//! la persistance et les pipelines IA de l'application.

// Importation des modules internes
mod api;
mod db;
mod docgen;
mod documents;
mod pipeline;
#[cfg(test)]
mod pipeline_manual_run;
mod progress;
mod security;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Connexion Postgres injoignable au démarrage : on préfère planter tout
    // de suite plutôt que de tourner sans persistance possible.
    let db_config = db::DbConfig::from_env();
    let pool = db::init_db(&db_config).await.expect("Failed to connect to database");

    // ------------------------------------------------------------------
    // Dossiers de données (documents générés, polices), créés au démarrage
    // s'ils n'existent pas.
    // ------------------------------------------------------------------
    let data_dir = std::path::PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()));
    let fonts_dir = std::env::var("FONTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| documents::resolve_fonts_dir(&data_dir));
    std::fs::create_dir_all(&data_dir).expect("Impossible de créer DATA_DIR");
    std::fs::create_dir_all(&fonts_dir).expect("Impossible de créer FONTS_DIR");

    if std::env::var("API_TOKEN").map(|t| t.is_empty()).unwrap_or(true) {
        eprintln!("[WARN] API_TOKEN non configuré : l'API accepte les requêtes sans authentification.");
    }

    let state = api::AppState {
        pool,
        data_dir,
        fonts_dir,
        progress: progress::ProgressStore::new(),
    };

    let app = api::router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Impossible d'écouter sur le port configuré");
    println!("API listening on {addr}");
    axum::serve(listener, app).await.expect("Le serveur HTTP s'est arrêté de façon inattendue");
}
