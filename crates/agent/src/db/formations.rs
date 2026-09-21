//! Accès et logique de persistance pour les formations du profil.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{delete_owned_row, map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateFormationInput, FormationView};

/// Liste les formations d'un utilisateur, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Formations triées par date de fin puis de début décroissantes.
pub async fn list_for_user(pool: &PgPool, user_id: i64) -> Result<Vec<FormationView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, titre, ecole, date_debut, date_fin \
         FROM profile.formations WHERE user_id = $1 \
         ORDER BY COALESCE(date_fin, DATE '9999-12-31') DESC, date_debut DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

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
        .map_err(map_db_err)
}

/// Ajoute une formation au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la formation à créer.
///
/// # Returns
///
/// `FormationView` fraîchement créée.
pub async fn insert(
    pool: &PgPool,
    user_id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO profile.formations (user_id, titre, ecole, date_debut, date_fin) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, titre, ecole, date_debut, date_fin",
    )
    .bind(user_id)
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(input.date_debut)
    .bind(input.date_fin)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(FormationView {
        id: row.try_get("id").map_err(map_db_err)?,
        titre: row.try_get("titre").map_err(map_db_err)?,
        ecole: row.try_get("ecole").map_err(map_db_err)?,
        date_debut: row.try_get("date_debut").map_err(map_db_err)?,
        date_fin: row.try_get("date_fin").map_err(map_db_err)?,
    })
}

/// Met à jour une formation existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la formation à modifier.
/// * `input` - Nouvelles données de la formation.
///
/// # Returns
///
/// `FormationView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(
    pool: &PgPool,
    id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE profile.formations \
         SET titre = $1, ecole = $2, date_debut = $3, date_fin = $4 \
         WHERE id = $5 AND user_id = (SELECT id FROM profile.users LIMIT 1) \
         RETURNING id, titre, ecole, date_debut, date_fin",
    )
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(input.date_debut)
    .bind(input.date_fin)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Formation introuvable.".to_string()))?;

    Ok(FormationView {
        id: row.try_get("id").map_err(map_db_err)?,
        titre: row.try_get("titre").map_err(map_db_err)?,
        ecole: row.try_get("ecole").map_err(map_db_err)?,
        date_debut: row.try_get("date_debut").map_err(map_db_err)?,
        date_fin: row.try_get("date_fin").map_err(map_db_err)?,
    })
}

/// Supprime une formation appartenant à l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la formation à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    delete_owned_row(pool, "profile.formations", id, "Formation introuvable.").await
}
