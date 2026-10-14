//! Accès et logique de persistance pour les projets personnels du profil.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::clock::format_date;
use crate::db::common::{delete_by_id, map_db_err, normalize};
use crate::error::AppError;
use crate::models::{CreateProjetInput, ProjetView};

/// Colonnes lues pour reconstruire un `ProjetView`.
const COLUMNS: &str = "id, titre, description, stack, url, pour_qui, date_debut, date_fin";

/// Valide le titre d'un projet.
///
/// # Arguments
///
/// * `input` - Données saisies.
///
/// # Returns
///
/// Titre trimé, ou `AppError::Validation` s'il est vide.
fn validate_titre(input: &CreateProjetInput) -> Result<String, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }
    Ok(titre)
}

/// Construit un `ProjetView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne sélectionnant `COLUMNS`.
///
/// # Returns
///
/// `ProjetView` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &SqliteRow) -> Result<ProjetView, sqlx::Error> {
    Ok(ProjetView {
        id: row.try_get("id")?,
        titre: row.try_get("titre")?,
        description: row.try_get("description")?,
        stack: row.try_get("stack")?,
        url: row.try_get("url")?,
        pour_qui: row.try_get("pour_qui")?,
        date_debut: row.try_get("date_debut")?,
        date_fin: row.try_get("date_fin")?,
    })
}

/// Liste les projets du profil, les plus récents en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Projets triés par date de fin (en cours d'abord) puis de début
/// décroissantes ; une date de début absente passe en tête, comme sous
/// PostgreSQL.
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ProjetView>, AppError> {
    let rows = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM projets \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC NULLS FIRST, id"
    ))
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Ajoute un projet au profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données du projet à créer.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `ProjetView` fraîchement créé.
pub async fn insert(pool: &SqlitePool, input: CreateProjetInput, now: &str) -> Result<ProjetView, AppError> {
    let titre = validate_titre(&input)?;

    let row = sqlx::query(&format!(
        "INSERT INTO projets (titre, description, stack, url, pour_qui, date_debut, date_fin, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING {COLUMNS}"
    ))
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(input.date_debut.map(format_date))
    .bind(input.date_fin.map(format_date))
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    row_to_view(&row).map_err(map_db_err)
}

/// Met à jour un projet existant.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant du projet à modifier.
/// * `input` - Nouvelles données du projet.
///
/// # Returns
///
/// `ProjetView` mis à jour, ou `AppError::NotFound` si absent.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateProjetInput) -> Result<ProjetView, AppError> {
    let titre = validate_titre(&input)?;

    let row = sqlx::query(&format!(
        "UPDATE projets \
         SET titre = $1, description = $2, stack = $3, url = $4, pour_qui = $5, \
             date_debut = $6, date_fin = $7 \
         WHERE id = $8 \
         RETURNING {COLUMNS}"
    ))
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(input.date_debut.map(format_date))
    .bind(input.date_fin.map(format_date))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Projet introuvable.".to_string()))?;
    row_to_view(&row).map_err(map_db_err)
}

/// Supprime un projet.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant du projet à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimé, `AppError::NotFound` si absent.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    delete_by_id(pool, "projets", id, "Projet introuvable.").await
}
