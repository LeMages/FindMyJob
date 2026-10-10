//! Accès et logique de persistance pour les expériences professionnelles.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::clock::format_date;
use crate::db::common::{delete_by_id, map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateExperienceInput, ExperienceView};

const TYPES_CONTRAT_VALIDES: [&str; 5] = ["stage", "alternance", "interim", "cdd", "cdi"];

/// Colonnes lues pour reconstruire un `ExperienceView`.
const COLUMNS: &str = "id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat";

/// Valide et normalise le type de contrat d'une expérience.
///
/// # Arguments
///
/// * `type_contrat` - Type de contrat brut saisi par l'utilisateur.
///
/// # Returns
///
/// Type de contrat normalisé, ou erreur si non reconnu.
fn validate_type_contrat(type_contrat: Option<String>) -> Result<Option<String>, AppError> {
    let type_contrat = normalize(type_contrat);
    if let Some(ref tc) = type_contrat {
        if !TYPES_CONTRAT_VALIDES.contains(&tc.as_str()) {
            return Err(AppError::Validation("Type de contrat invalide.".to_string()));
        }
    }
    Ok(type_contrat)
}

/// Valide l'intitulé du poste.
///
/// # Arguments
///
/// * `input` - Données saisies.
///
/// # Returns
///
/// Intitulé trimé, ou `AppError::Validation` s'il est vide.
fn validate_poste(input: &CreateExperienceInput) -> Result<String, AppError> {
    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation("L'intitulé du poste est requis.".to_string()));
    }
    Ok(poste)
}

/// Construit un `ExperienceView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne sélectionnant `COLUMNS`.
///
/// # Returns
///
/// `ExperienceView` reconstruite, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &SqliteRow) -> Result<ExperienceView, sqlx::Error> {
    Ok(ExperienceView {
        id: row.try_get("id")?,
        poste: row.try_get("poste")?,
        entreprise: row.try_get("entreprise")?,
        lieu: row.try_get("lieu")?,
        description: row.try_get("description")?,
        date_debut: row.try_get("date_debut")?,
        date_fin: row.try_get("date_fin")?,
        type_contrat: row.try_get("type_contrat")?,
    })
}

/// Liste les expériences du profil, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Expériences triées par date de fin (en cours d'abord) puis de début
/// décroissantes ; une date de début absente passe en tête, comme sous
/// PostgreSQL.
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ExperienceView>, AppError> {
    let rows = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM experiences \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC NULLS FIRST, id"
    ))
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Ajoute une expérience au profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données de l'expérience à créer.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `ExperienceView` fraîchement créée.
pub async fn insert(pool: &SqlitePool, input: CreateExperienceInput, now: &str) -> Result<ExperienceView, AppError> {
    let poste = validate_poste(&input)?;
    let type_contrat = validate_type_contrat(input.type_contrat)?;

    let row = sqlx::query(&format!(
        "INSERT INTO experiences \
           (poste, entreprise, lieu, description, date_debut, date_fin, type_contrat, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING {COLUMNS}"
    ))
    .bind(&poste)
    .bind(normalize(input.entreprise))
    .bind(normalize(input.lieu))
    .bind(normalize(input.description))
    .bind(input.date_debut.map(format_date))
    .bind(input.date_fin.map(format_date))
    .bind(&type_contrat)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    row_to_view(&row).map_err(map_db_err)
}

/// Met à jour une expérience existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de l'expérience à modifier.
/// * `input` - Nouvelles données de l'expérience.
///
/// # Returns
///
/// `ExperienceView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateExperienceInput) -> Result<ExperienceView, AppError> {
    let poste = validate_poste(&input)?;
    let type_contrat = validate_type_contrat(input.type_contrat)?;

    let row = sqlx::query(&format!(
        "UPDATE experiences \
         SET poste = $1, entreprise = $2, lieu = $3, description = $4, \
             date_debut = $5, date_fin = $6, type_contrat = $7 \
         WHERE id = $8 \
         RETURNING {COLUMNS}"
    ))
    .bind(&poste)
    .bind(normalize(input.entreprise))
    .bind(normalize(input.lieu))
    .bind(normalize(input.description))
    .bind(input.date_debut.map(format_date))
    .bind(input.date_fin.map(format_date))
    .bind(&type_contrat)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Expérience introuvable.".to_string()))?;
    row_to_view(&row).map_err(map_db_err)
}

/// Supprime une expérience.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de l'expérience à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    delete_by_id(pool, "experiences", id, "Expérience introuvable.").await
}
