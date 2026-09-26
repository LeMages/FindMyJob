//! Persistance des offres scrapées dans la table `offers`.

// Importation des modules standard
use std::collections::HashSet;

// Importation des crates externes
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::scraping::types::ScrapedOffer;
use findmyjob_core::error::AppError;
use findmyjob_core::models::OfferView;

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
        session_id: row.try_get("session_id")?,
        source: row.try_get("source")?,
        title: row.try_get("title")?,
        company: row.try_get("company")?,
        location: row.try_get("location")?,
        url: row.try_get("url")?,
        contract_type: row.try_get("contract_type")?,
        posted_at: row.try_get("posted_at")?,
        description: row.try_get("description")?,
        status: row.try_get("status")?,
        score: row.try_get("score")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Produit l'expression SQL qui normalise une URL d'offre pour comparaison.
///
/// Une URL collée à la main dans une candidature diffère souvent de l'URL
/// scrapée (`?utm_source=…`, `/` final, `http` vs `https`, `www.`, casse) :
/// on retire le schéma, le `www.`, le fragment, la query string et les `/`
/// finaux, puis on passe en minuscules.
///
/// # Arguments
///
/// * `column` - Colonne SQL (qualifiée) contenant l'URL à normaliser.
///
/// # Returns
///
/// Fragment SQL à insérer dans une requête.
fn normalized_url_sql(column: &str) -> String {
    format!(
        "lower(regexp_replace(split_part(split_part(\
         regexp_replace(btrim({column}), '^https?://(www\\.)?', '', 'i'), '#', 1), '?', 1), '/+$', ''))"
    )
}

/// Insère les offres scrapées en base, en ignorant les doublons.
///
/// La dédup se fait sur la contrainte unique `offers.url` : une offre déjà
/// connue est silencieusement ignorée (`ON CONFLICT DO NOTHING`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session à laquelle rattacher les offres insérées.
/// * `offers` - Offres à insérer.
///
/// # Returns
///
/// Nombre d'offres effectivement insérées (nouvelles).
pub async fn insert_offers(pool: &PgPool, session_id: i64, offers: &[ScrapedOffer]) -> Result<u64, sqlx::Error> {
    let mut inserted = 0u64;

    for offer in offers {
        let result = sqlx::query(
            "INSERT INTO search.offers (session_id, source, source_id, title, company, location, url, contract_type, posted_at, scraped_at, description) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (url) DO NOTHING",
        )
        .bind(session_id)
        .bind(&offer.source)
        .bind(&offer.source_id)
        .bind(&offer.title)
        .bind(&offer.company)
        .bind(&offer.location)
        .bind(&offer.url)
        .bind(&offer.contract_type)
        .bind(offer.posted_at)
        .bind(offer.scraped_at)
        .bind(&offer.description)
        .execute(pool)
        .await?;

        inserted += result.rows_affected();
    }

    let duplicates = offers.len() as u64 - inserted;
    eprintln!("{duplicates} offre(s) déjà connue(s), ignorée(s)");

    Ok(inserted)
}

/// Filtre, parmi des URLs d'offres, celles déjà présentes en base (toutes
/// sessions confondues, `offers.url` étant unique globalement).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `urls` - URLs à vérifier.
///
/// # Returns
///
/// L'ensemble des URLs déjà connues.
pub async fn find_existing_urls(pool: &PgPool, urls: &[String]) -> Result<HashSet<String>, AppError> {
    let existing: Vec<String> = sqlx::query_scalar("SELECT url FROM search.offers WHERE url = ANY($1)")
        .bind(urls)
        .fetch_all(pool)
        .await
        .map_err(map_db_err)?;

    Ok(existing.into_iter().collect())
}

/// Liste les offres d'une session dont le score dépasse un seuil, quel que
/// soit leur statut, en excluant celles pour lesquelles une candidature
/// existe déjà.
///
/// Une offre est considérée comme déjà candidatée si une candidature (toutes
/// sessions et tous statuts confondus) la référence via `offer_id` (créée
/// depuis le Marché) ou porte la même URL normalisée dans `url_offre`
/// (saisie manuelle).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session dont on veut les offres.
/// * `min_score` - Seuil exclusif : seules les offres de score strictement supérieur sont retournées.
///
/// # Returns
///
/// Offres triées par score décroissant, les mieux notées en premier.
pub async fn list_for_session(pool: &PgPool, session_id: i64, min_score: f32) -> Result<Vec<OfferView>, AppError> {
    let sql = format!(
        "SELECT id, session_id, source, title, company, location, url, contract_type, posted_at, \
                description, status, score, created_at \
         FROM search.offers \
         WHERE session_id = $1 AND score > $2 \
           AND NOT EXISTS ( \
               SELECT 1 FROM search.candidatures c \
               WHERE c.offer_id = offers.id \
                  OR (c.url_offre IS NOT NULL AND {} = {}) \
           ) \
         ORDER BY score DESC",
        normalized_url_sql("c.url_offre"),
        normalized_url_sql("offers.url"),
    );
    let rows = sqlx::query(&sql)
        .bind(session_id)
        .bind(min_score)
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
        "SELECT id, session_id, source, title, company, location, url, contract_type, posted_at, \
                description, status, score, created_at \
         FROM search.offers WHERE id = $1",
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
    let result = sqlx::query("UPDATE search.offers SET status = $1, updated_at = NOW() WHERE id = $2")
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

/// Marque une offre comme ayant donné lieu à une candidature, dans une
/// transaction existante — variante tx de `mark_applied`, utilisée par
/// `db::candidatures::insert` pour que la création de la candidature et le
/// changement de statut de l'offre restent atomiques.
///
/// # Arguments
///
/// * `tx` - Transaction PostgreSQL active.
/// * `id` - Identifiant de l'offre.
///
/// # Returns
///
/// `Ok(())` si la mise à jour a réussi, `AppError::NotFound` sinon.
pub async fn mark_applied_tx(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE search.offers SET status = 'applied', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Offre introuvable.".to_string()));
    }
    Ok(())
}

/// Vue minimale d'une offre non encore scorée, utilisée par le filtre
/// mécanique et le scoring LLM (`filtering::mechanical`, `filtering::scoring`).
/// Vue interne au pipeline, distincte d'`OfferView` : sans les colonnes de
/// statut, score et session exposées par l'API.
pub struct UnscoredOffer {
    pub id: i64,
    pub title: String,
    pub company: String,
    pub location: String,
    pub contract_type: Option<String>,
    pub description: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
}

/// Construit un `UnscoredOffer` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête `SELECT` sur `offers`.
///
/// # Returns
///
/// `UnscoredOffer` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_unscored_offer(row: &PgRow) -> Result<UnscoredOffer, sqlx::Error> {
    Ok(UnscoredOffer {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        company: row.try_get("company")?,
        location: row.try_get("location")?,
        contract_type: row.try_get("contract_type")?,
        description: row.try_get("description")?,
        posted_at: row.try_get("posted_at")?,
    })
}

/// Liste les offres pas encore traitées par le pipeline de scoring
/// (`score IS NULL`) — inclut celles pas encore vues par le filtre
/// mécanique et celles qui l'ont passé mais pas encore été scorées par le LLM.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session dont on veut les offres non scorées.
///
/// # Returns
///
/// Les offres non scorées de la session, sans ordre garanti.
pub async fn get_unscored_offers(pool: &PgPool, session_id: i64) -> Result<Vec<UnscoredOffer>, AppError> {
    let rows = sqlx::query(
        "SELECT id, title, company, location, contract_type, description, posted_at \
         FROM search.offers WHERE session_id = $1 AND score IS NULL",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_unscored_offer).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Fixe le score et le statut d'une offre (filtre mécanique ou scoring LLM).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de l'offre.
/// * `score` - Score attribué (`-1.0` pour un rejet mécanique, `0..1` pour le LLM).
/// * `status` - Nouveau statut (`"dismissed"` ou `"reviewed"`).
///
/// # Returns
///
/// `Ok(())` si la mise à jour a réussi.
pub async fn set_score(pool: &PgPool, id: i64, score: f32, status: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE search.offers SET score = $1, status = $2, updated_at = NOW() WHERE id = $3")
        .bind(score)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_db_err)?;
    Ok(())
}
