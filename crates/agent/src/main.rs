//! Agent FindMyJob (service headless) : scraping d'offres d'emploi.

// Importation des modules internes
mod db;
mod logging;
mod scraping;

use scraping::types::{JobScraper, SearchQuery};
use scraping::wttj::WttjScraper;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Connexion Postgres injoignable au démarrage : on préfère planter tout
    // de suite plutôt que de tourner sans persistance possible.
    let config = db::DbConfig::from_env();
    let pool = db::init_db(&config)
        .await
        .expect("Failed to connect to database");

    let scraper = WttjScraper::new();
    let queries = vec![SearchQuery {
        keywords: "Stage Data Engineer".to_string(),
        location: Some("Paris".to_string()),
        country_code: Some("FR".to_string()),
    }];

    match scraper.scrape(&queries).await {
        Ok(offers) => {
            println!("Trouvé {} offres\n", offers.len());
            for (i, offer) in offers.iter().enumerate() {
                println!("#{}", i + 1);
                println!("  Titre:      {}", offer.title);
                println!("  Entreprise: {}", offer.company);
                println!("  Lieu:       {}", offer.location);
                println!(
                    "  Contrat:    {}",
                    offer.contract_type.as_deref().unwrap_or("N/A")
                );
                println!(
                    "  Date:       {}",
                    offer
                        .posted_at
                        .map(|d| d.to_rfc3339())
                        .unwrap_or_else(|| "N/A".to_string())
                );
                println!("  URL:        {}", offer.url);
                println!();
            }

            // La DB peut être tombée après la connexion initiale : on log
            // l'erreur sans faire planter l'agent, le scraping console
            // reste utile même sans persistance.
            match db::offers::insert_offers(&pool, &offers).await {
                Ok(inserted) => {
                    let duplicates = offers.len() as u64 - inserted;
                    println!("{inserted} nouvelles offres insérées ({duplicates} doublons ignorés)");
                }
                Err(e) => {
                    eprintln!("Erreur insertion: {e}");
                    logging::log_error_to_file("insertion offres", &e);
                }
            }
        }
        Err(e) => eprintln!("Erreur scraping: {e}"),
    }
}
