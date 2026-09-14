//! Persistance des offres scrapées dans la table `offers`.

// Importation des crates externes
use sqlx::PgPool;

// Importation des modules internes
use crate::scraping::types::ScrapedOffer;

/// Insère les offres scrapées en base, en ignorant les doublons.
///
/// La dédup se fait sur la contrainte unique `offers.url` : une offre déjà
/// connue est silencieusement ignorée (`ON CONFLICT DO NOTHING`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `offers` - Offres à insérer.
///
/// # Returns
///
/// Nombre d'offres effectivement insérées (nouvelles).
pub async fn insert_offers(pool: &PgPool, offers: &[ScrapedOffer]) -> Result<u64, sqlx::Error> {
    let mut inserted = 0u64;

    for offer in offers {
        let result = sqlx::query(
            "INSERT INTO offers (source, source_id, title, company, location, url, contract_type, posted_at, scraped_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (url) DO NOTHING",
        )
        .bind(&offer.source)
        .bind(&offer.source_id)
        .bind(&offer.title)
        .bind(&offer.company)
        .bind(&offer.location)
        .bind(&offer.url)
        .bind(&offer.contract_type)
        .bind(offer.posted_at)
        .bind(offer.scraped_at)
        .execute(pool)
        .await?;

        inserted += result.rows_affected();
    }

    let duplicates = offers.len() as u64 - inserted;
    eprintln!("{duplicates} offre(s) déjà connue(s), ignorée(s)");

    Ok(inserted)
}
