//! Accès et logique de persistance pour les formations du profil.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::{delete_owned_row, normalize};
use crate::error::AppError;
use crate::models::{CreateFormationInput, FormationView};

/// Liste les formations d'un utilisateur, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Formations triées par date de fin puis de début décroissantes.
pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<FormationView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, titre, ecole, \
                CAST(date_debut AS TEXT) as date_debut, \
                CAST(date_fin   AS TEXT) as date_fin \
         FROM formations WHERE user_id = ? \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<FormationView, sqlx::Error> {
            Ok(FormationView {
                id: r.try_get("id")?,
                titre: r.try_get("titre")?,
                ecole: r.try_get("ecole")?,
                date_debut: r.try_get("date_debut")?,
                date_fin: r.try_get("date_fin")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}

/// Ajoute une formation au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la formation à créer.
///
/// # Returns
///
/// `FormationView` fraîchement créée.
pub async fn insert(
    pool: &SqlitePool,
    user_id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO formations (user_id, titre, ecole, date_debut, date_fin) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, titre, ecole, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(user_id)
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .fetch_one(pool)
    .await?;

    Ok(FormationView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        ecole: row.try_get("ecole").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
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
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE formations \
         SET titre = ?, ecole = ?, date_debut = ?, date_fin = ?, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, titre, ecole, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Formation introuvable.".to_string()))?;

    Ok(FormationView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        ecole: row.try_get("ecole").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
}

/// Supprime une formation appartenant à l'utilisateur.
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
    delete_owned_row(pool, "formations", id, "Formation introuvable.").await
}
