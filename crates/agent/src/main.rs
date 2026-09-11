//! Agent FindMyJob (service headless) : scraping d'offres d'emploi.

// Importation des modules internes
mod scraping;

use scraping::types::{JobScraper, SearchQuery};
use scraping::wttj::WttjScraper;

#[tokio::main]
async fn main() {
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
                println!("  Date:       {}", offer.posted_at.as_deref().unwrap_or("N/A"));
                println!("  URL:        {}", offer.url);
                println!();
            }
        }
        Err(e) => eprintln!("Erreur: {e}"),
    }
}
