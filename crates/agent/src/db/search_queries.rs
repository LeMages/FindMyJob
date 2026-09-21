//! Accès et logique de persistance pour les requêtes de recherche d'une session.

// Importation des crates externes
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateSearchQueryInput, SearchQueryView};

/// Convertit une ligne `search_queries` en `SearchQueryView`.
///
/// # Arguments
///
/// * `row` - Ligne retournée par une requête sur `search_queries`.
///
/// # Returns
///
/// `SearchQueryView` correspondante, ou erreur si une colonne attendue est absente.
fn row_to_search_query(row: &PgRow) -> Result<SearchQueryView, AppError> {
    Ok(SearchQueryView {
        id: row.try_get("id").map_err(map_db_err)?,
        session_id: row.try_get("session_id").map_err(map_db_err)?,
        keywords: row.try_get("keywords").map_err(map_db_err)?,
        location: row.try_get("location").map_err(map_db_err)?,
        country_code: row.try_get("country_code").map_err(map_db_err)?,
        created_at: row.try_get("created_at").map_err(map_db_err)?,
    })
}

/// Ajoute une requête de recherche à une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `input` - Mots-clés et critères de la requête.
///
/// # Returns
///
/// `SearchQueryView` fraîchement créée.
pub async fn insert(
    pool: &PgPool,
    session_id: i64,
    input: &CreateSearchQueryInput,
) -> Result<SearchQueryView, AppError> {
    let keywords = input.keywords.trim().to_string();
    if keywords.is_empty() {
        return Err(AppError::Validation("Les mots-clés de la recherche sont requis.".to_string()));
    }
    let location = normalize(input.location.clone());
    let country_code = normalize(input.country_code.clone());

    let row = sqlx::query(
        "INSERT INTO search.search_queries (session_id, keywords, location, country_code) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, session_id, keywords, location, country_code, created_at",
    )
    .bind(session_id)
    .bind(&keywords)
    .bind(&location)
    .bind(&country_code)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    row_to_search_query(&row)
}

/// Liste les requêtes de recherche d'une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Requêtes triées par date de création croissante.
pub async fn list_for_session(pool: &PgPool, session_id: i64) -> Result<Vec<SearchQueryView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, session_id, keywords, location, country_code, created_at \
         FROM search.search_queries WHERE session_id = $1 ORDER BY created_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_search_query).collect()
}

/// Supprime une requête de recherche d'une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `id` - Identifiant de la requête à supprimer.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi, `AppError::NotFound` si absente.
pub async fn delete(pool: &PgPool, session_id: i64, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM search.search_queries WHERE id = $1 AND session_id = $2")
        .bind(id)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Recherche introuvable.".to_string()));
    }
    Ok(())
}
