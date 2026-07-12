//! Accès et logique de persistance pour les projets personnels du profil.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::{delete_owned_row, normalize};
use crate::error::AppError;
use crate::models::{CreateProjetInput, ProjetView};

/// Liste les projets d'un utilisateur, les plus récents en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Projets triés par date de fin puis de début décroissantes.
pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<ProjetView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, titre, description, stack, url, pour_qui, \
                CAST(date_debut AS TEXT) as date_debut, \
                CAST(date_fin   AS TEXT) as date_fin \
         FROM projets WHERE user_id = ? \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

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
        .map_err(AppError::from)
}

/// Ajoute un projet au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données du projet à créer.
///
/// # Returns
///
/// `ProjetView` fraîchement créé.
pub async fn insert(
    pool: &SqlitePool,
    user_id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO projets (user_id, titre, description, stack, url, pour_qui, date_debut, date_fin) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, titre, description, stack, url, pour_qui, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(user_id)
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .fetch_one(pool)
    .await?;

    Ok(ProjetView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        stack: row.try_get("stack").map_err(AppError::from)?,
        url: row.try_get("url").map_err(AppError::from)?,
        pour_qui: row.try_get("pour_qui").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
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
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE projets \
         SET titre = ?, description = ?, stack = ?, url = ?, pour_qui = ?, \
             date_debut = ?, date_fin = ?, updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, titre, description, stack, url, pour_qui, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Projet introuvable.".to_string()))?;

    Ok(ProjetView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        stack: row.try_get("stack").map_err(AppError::from)?,
        url: row.try_get("url").map_err(AppError::from)?,
        pour_qui: row.try_get("pour_qui").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
}

/// Supprime un projet appartenant à l'utilisateur.
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
    delete_owned_row(pool, "projets", id, "Projet introuvable.").await
}
