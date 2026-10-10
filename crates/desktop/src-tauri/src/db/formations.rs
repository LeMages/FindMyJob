//! Accès et logique de persistance pour les formations du profil.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::clock::format_date;
use crate::db::common::{delete_by_id, map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateFormationInput, FormationView};

/// Colonnes lues pour reconstruire un `FormationView`.
const COLUMNS: &str = "id, titre, ecole, description, date_debut, date_fin";

/// Valide le titre d'une formation.
///
/// # Arguments
///
/// * `input` - Données saisies.
///
/// # Returns
///
/// Titre trimé, ou `AppError::Validation` s'il est vide.
fn validate_titre(input: &CreateFormationInput) -> Result<String, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }
    Ok(titre)
}

/// Construit un `FormationView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne sélectionnant `COLUMNS`.
///
/// # Returns
///
/// `FormationView` reconstruite, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &SqliteRow) -> Result<FormationView, sqlx::Error> {
    Ok(FormationView {
        id: row.try_get("id")?,
        titre: row.try_get("titre")?,
        ecole: row.try_get("ecole")?,
        description: row.try_get("description")?,
        date_debut: row.try_get("date_debut")?,
        date_fin: row.try_get("date_fin")?,
    })
}

/// Liste les formations du profil, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Formations triées par date de fin (en cours d'abord) puis de début
/// décroissantes ; une date de début absente passe en tête, comme sous
/// PostgreSQL.
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<FormationView>, AppError> {
    let rows = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM formations \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC NULLS FIRST, id"
    ))
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Ajoute une formation au profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données de la formation à créer.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `FormationView` fraîchement créée.
pub async fn insert(pool: &SqlitePool, input: CreateFormationInput, now: &str) -> Result<FormationView, AppError> {
    let titre = validate_titre(&input)?;

    let row = sqlx::query(&format!(
        "INSERT INTO formations (titre, ecole, description, date_debut, date_fin, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING {COLUMNS}"
    ))
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(normalize(input.description))
    .bind(input.date_debut.map(format_date))
    .bind(input.date_fin.map(format_date))
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    row_to_view(&row).map_err(map_db_err)
}

/// Met à jour une formation existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la formation à modifier.
/// * `input` - Nouvelles données de la formation.
///
/// # Returns
///
/// `FormationView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateFormationInput) -> Result<FormationView, AppError> {
    let titre = validate_titre(&input)?;

    let row = sqlx::query(&format!(
        "UPDATE formations \
         SET titre = $1, ecole = $2, description = $3, date_debut = $4, date_fin = $5 \
         WHERE id = $6 \
         RETURNING {COLUMNS}"
    ))
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(normalize(input.description))
    .bind(input.date_debut.map(format_date))
    .bind(input.date_fin.map(format_date))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Formation introuvable.".to_string()))?;
    row_to_view(&row).map_err(map_db_err)
}

/// Supprime une formation.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la formation à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    delete_by_id(pool, "formations", id, "Formation introuvable.").await
}
