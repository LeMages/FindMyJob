//! Persistance des offres scrapées dans la table `offers`.

// Importation des crates externes
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::scraping::types::ScrapedOffer;
use findmyjob_core::error::AppError;

/// Vue d'une offre scrapée, exposée par l'API agent-specific `/api/offers`
/// (offres recommandées). N'expose pas `embedding` (donnée technique interne
/// au scoring, étape 3).
#[derive(serde::Serialize)]
pub struct OfferView {
    pub id: i64,
    pub source: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub contract_type: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub status: String,
    pub score: Option<f32>,
    pub created_at: DateTime<Utc>,
}

/// Construit un `OfferView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête `SELECT`/`RETURNING` sur `offers`.
///
/// # Returns
///
/// `OfferView` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_offer(row: &PgRow) -> Result<OfferView, sqlx::Error> {
    Ok(OfferView {
        id: row.try_get("id")?,
        source: row.try_get("source")?,
        title: row.try_get("title")?,
        company: row.try_get("company")?,
        location: row.try_get("location")?,
        url: row.try_get("url")?,
        contract_type: row.try_get("contract_type")?,
        posted_at: row.try_get("posted_at")?,
        status: row.try_get("status")?,
        score: row.try_get("score")?,
        created_at: row.try_get("created_at")?,
    })
}

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

/// Liste les offres recommandées : non encore traitées (`new`/`reviewed`) et
/// scorées positivement par le pipeline de matching (étape 3).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// Offres triées par score décroissant, les mieux notées en premier.
pub async fn list_new(pool: &PgPool) -> Result<Vec<OfferView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, source, title, company, location, url, contract_type, posted_at, \
                status, score, created_at \
         FROM offers \
         WHERE status IN ('new', 'reviewed') AND score > 0 \
         ORDER BY score DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_offer).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Récupère une offre par son identifiant.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de l'offre.
///
/// # Returns
///
/// `OfferView` correspondante, ou `AppError::NotFound` si elle n'existe pas.
pub async fn fetch_one(pool: &PgPool, id: i64) -> Result<OfferView, AppError> {
    let row = sqlx::query(
        "SELECT id, source, title, company, location, url, contract_type, posted_at, \
                status, score, created_at \
         FROM offers WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?
    .ok_or_else(|| AppError::NotFound("Offre introuvable.".to_string()))?;

    row_to_offer(&row).map_err(map_db_err)
}

/// Met à jour le statut d'une offre, à condition qu'elle existe.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de l'offre.
/// * `status` - Nouveau statut (`dismissed` ou `applied`).
///
/// # Returns
///
/// `Ok(())` si la mise à jour a réussi, `AppError::NotFound` sinon.
async fn set_status(pool: &PgPool, id: i64, status: &str) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE offers SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Offre introuvable.".to_string()));
    }
    Ok(())
}

/// Marque une offre comme écartée par l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de l'offre.
///
/// # Returns
///
/// `Ok(())` si la mise à jour a réussi, `AppError::NotFound` sinon.
pub async fn dismiss(pool: &PgPool, id: i64) -> Result<(), AppError> {
    set_status(pool, id, "dismissed").await
}

/// Marque une offre comme ayant donné lieu à une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de l'offre.
///
/// # Returns
///
/// `Ok(())` si la mise à jour a réussi, `AppError::NotFound` sinon.
pub async fn mark_applied(pool: &PgPool, id: i64) -> Result<(), AppError> {
    set_status(pool, id, "applied").await
}
