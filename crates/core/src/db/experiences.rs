//! Accès et logique de persistance pour les expériences professionnelles.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::{delete_owned_row, normalize};
use crate::error::AppError;
use crate::models::{CreateExperienceInput, ExperienceView};

const TYPES_CONTRAT_VALIDES: [&str; 5] = ["stage", "alternance", "interim", "cdd", "cdi"];

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

/// Liste les expériences d'un utilisateur, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Expériences triées par date de fin puis de début décroissantes.
pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<ExperienceView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat \
         FROM experiences WHERE user_id = ? \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<ExperienceView, sqlx::Error> {
            Ok(ExperienceView {
                id: r.try_get("id")?,
                poste: r.try_get("poste")?,
                entreprise: r.try_get("entreprise")?,
                lieu: r.try_get("lieu")?,
                description: r.try_get("description")?,
                date_debut: r.try_get("date_debut")?,
                date_fin: r.try_get("date_fin")?,
                type_contrat: r.try_get("type_contrat")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}

/// Ajoute une expérience au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de l'expérience à créer.
///
/// # Returns
///
/// `ExperienceView` fraîchement créée.
pub async fn insert(
    pool: &SqlitePool,
    user_id: i64,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation(
            "L'intitulé du poste est requis.".to_string(),
        ));
    }
    let type_contrat = validate_type_contrat(input.type_contrat)?;

    let row = sqlx::query(
        "INSERT INTO experiences \
           (user_id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat",
    )
    .bind(user_id)
    .bind(&poste)
    .bind(normalize(input.entreprise))
    .bind(normalize(input.lieu))
    .bind(normalize(input.description))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(&type_contrat)
    .fetch_one(pool)
    .await?;

    Ok(ExperienceView {
        id: row.try_get("id").map_err(AppError::from)?,
        poste: row.try_get("poste").map_err(AppError::from)?,
        entreprise: row.try_get("entreprise").map_err(AppError::from)?,
        lieu: row.try_get("lieu").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
        type_contrat: row.try_get("type_contrat").map_err(AppError::from)?,
    })
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
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation(
            "L'intitulé du poste est requis.".to_string(),
        ));
    }
    let type_contrat = validate_type_contrat(input.type_contrat)?;

    let row = sqlx::query(
        "UPDATE experiences \
         SET poste = ?, entreprise = ?, lieu = ?, description = ?, \
             date_debut = ?, date_fin = ?, type_contrat = ?, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat",
    )
    .bind(&poste)
    .bind(normalize(input.entreprise))
    .bind(normalize(input.lieu))
    .bind(normalize(input.description))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(&type_contrat)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Expérience introuvable.".to_string()))?;

    Ok(ExperienceView {
        id: row.try_get("id").map_err(AppError::from)?,
        poste: row.try_get("poste").map_err(AppError::from)?,
        entreprise: row.try_get("entreprise").map_err(AppError::from)?,
        lieu: row.try_get("lieu").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
        type_contrat: row.try_get("type_contrat").map_err(AppError::from)?,
    })
}

/// Supprime une expérience appartenant à l'utilisateur.
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
    delete_owned_row(pool, "experiences", id, "Expérience introuvable.").await
}
