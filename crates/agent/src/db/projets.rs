//! Accès et logique de persistance pour les projets personnels du profil.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{delete_owned_row, map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateProjetInput, ProjetView};

/// Liste les projets d'un utilisateur, les plus récents en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Projets triés par date de fin puis de début décroissantes.
pub async fn list_for_user(pool: &PgPool, user_id: i64) -> Result<Vec<ProjetView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, titre, description, stack, url, pour_qui, date_debut, date_fin \
         FROM profile.projets WHERE user_id = $1 \
         ORDER BY COALESCE(date_fin, DATE '9999-12-31') DESC, date_debut DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<ProjetView, sqlx::Error> {
            Ok(ProjetView {
                id: r.try_get("id")?,
                titre: r.try_get("titre")?,
                description: r.try_get("description")?,
                stack: r.try_get("stack")?,
                url: r.try_get("url")?,
                pour_qui: r.try_get("pour_qui")?,
                date_debut: r.try_get("date_debut")?,
                date_fin: r.try_get("date_fin")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Ajoute un projet au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données du projet à créer.
///
/// # Returns
///
/// `ProjetView` fraîchement créé.
pub async fn insert(
    pool: &PgPool,
    user_id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO profile.projets (user_id, titre, description, stack, url, pour_qui, date_debut, date_fin) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, titre, description, stack, url, pour_qui, date_debut, date_fin",
    )
    .bind(user_id)
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(input.date_debut)
    .bind(input.date_fin)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(ProjetView {
        id: row.try_get("id").map_err(map_db_err)?,
        titre: row.try_get("titre").map_err(map_db_err)?,
        description: row.try_get("description").map_err(map_db_err)?,
        stack: row.try_get("stack").map_err(map_db_err)?,
        url: row.try_get("url").map_err(map_db_err)?,
        pour_qui: row.try_get("pour_qui").map_err(map_db_err)?,
        date_debut: row.try_get("date_debut").map_err(map_db_err)?,
        date_fin: row.try_get("date_fin").map_err(map_db_err)?,
    })
}

/// Met à jour un projet existant.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant du projet à modifier.
/// * `input` - Nouvelles données du projet.
///
/// # Returns
///
/// `ProjetView` mis à jour, ou `AppError::NotFound` si absent.
pub async fn update(
    pool: &PgPool,
    id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE profile.projets \
         SET titre = $1, description = $2, stack = $3, url = $4, pour_qui = $5, \
             date_debut = $6, date_fin = $7 \
         WHERE id = $8 AND user_id = (SELECT id FROM profile.users LIMIT 1) \
         RETURNING id, titre, description, stack, url, pour_qui, date_debut, date_fin",
    )
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(input.date_debut)
    .bind(input.date_fin)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Projet introuvable.".to_string()))?;

    Ok(ProjetView {
        id: row.try_get("id").map_err(map_db_err)?,
        titre: row.try_get("titre").map_err(map_db_err)?,
        description: row.try_get("description").map_err(map_db_err)?,
        stack: row.try_get("stack").map_err(map_db_err)?,
        url: row.try_get("url").map_err(map_db_err)?,
        pour_qui: row.try_get("pour_qui").map_err(map_db_err)?,
        date_debut: row.try_get("date_debut").map_err(map_db_err)?,
        date_fin: row.try_get("date_fin").map_err(map_db_err)?,
    })
}

/// Supprime un projet appartenant à l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant du projet à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimé, `AppError::NotFound` si absent.
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    delete_owned_row(pool, "profile.projets", id, "Projet introuvable.").await
}
