//! Agent FindMyJob (service headless) : serveur API REST (Axum) et scraping
//! d'offres d'emploi en tâche de fond.

// Importation des modules internes
mod api;
mod db;
mod documents;
mod logging;
mod pipeline;
mod scraping;
mod security;

use scraping::types::{JobScraper, SearchQuery};
use scraping::wttj::WttjScraper;

/// Lance une passe de scraping WTTJ et insère les offres trouvées en base.
///
/// Séparé de `main` pour être lancé en tâche de fond (`tokio::spawn`), sans
/// bloquer le démarrage du serveur HTTP.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
async fn run_scraping_once(pool: sqlx::PgPool) {
    let scraper = WttjScraper::new();
    let queries = vec![SearchQuery {
        keywords: "Stage Data Engineer".to_string(),
        location: Some("Paris".to_string()),
        country_code: Some("FR".to_string()),
    }];

    // `JobScraper::scrape` retourne `Box<dyn Error>` (non `Send`) : converti en
    // `String` avant le prochain `await`, pour que le futur de cette tâche de
    // fond (lancée via `tokio::spawn`, qui exige `Send`) reste valide.
    let scrape_result = scraper.scrape(&queries).await.map_err(|e| e.to_string());

    match scrape_result {
        Ok(offers) => match db::offers::insert_offers(&pool, &offers).await {
            Ok(inserted) => {
                let duplicates = offers.len() as u64 - inserted;
                println!("Scraping : {inserted} nouvelles offres insérées ({duplicates} doublons ignorés)");
            }
            Err(e) => {
                eprintln!("Erreur insertion des offres scrapées: {e}");
                logging::log_error_to_file("insertion offres", &e);
            }
        },
        Err(e) => eprintln!("Erreur scraping: {e}"),
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Connexion Postgres injoignable au démarrage : on préfère planter tout
    // de suite plutôt que de tourner sans persistance possible.
    let config = db::DbConfig::from_env();
    let pool = db::init_db(&config).await.expect("Failed to connect to database");

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

    let state = api::AppState { pool: pool.clone(), data_dir, fonts_dir };

    // Scraping initial en tâche de fond, ne bloque pas le démarrage du serveur.
    // TODO(étape ultérieure): remplacer par un scheduler périodique (cron).
    tokio::spawn(run_scraping_once(pool));

    let app = api::router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Impossible d'écouter sur le port configuré");
    println!("API listening on {addr}");
    axum::serve(listener, app).await.expect("Le serveur HTTP s'est arrêté de façon inattendue");
}
