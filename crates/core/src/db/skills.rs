//! Accès et logique de persistance pour les compétences du profil.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::delete_owned_row;
use crate::error::AppError;
use crate::models::{CreateSkillInput, SkillView};

/// Valide le nom et le domaine d'une compétence.
///
/// # Arguments
///
/// * `input` - Données brutes saisies par l'utilisateur.
///
/// # Returns
///
/// Nom et domaine trimés, ou erreur de validation si l'un des deux est vide.
fn validate(input: &CreateSkillInput) -> Result<(String, String), AppError> {
    let nom = input.nom.trim().to_string();
    let domaine = input.domaine.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la compétence est requis.".to_string()));
    }
    if domaine.is_empty() {
        return Err(AppError::Validation("Le domaine est requis.".to_string()));
    }
    Ok((nom, domaine))
}

/// Liste les compétences d'un utilisateur, groupées par domaine puis triées par nom.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Compétences triées par domaine puis nom.
pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<SkillView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, domaine, nom FROM skills WHERE user_id = ? ORDER BY domaine, nom",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<SkillView, sqlx::Error> {
            Ok(SkillView {
                id: r.try_get("id")?,
                domaine: r.try_get("domaine")?,
                nom: r.try_get("nom")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}

/// Ajoute une compétence au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la compétence à créer.
///
/// # Returns
///
/// `SkillView` fraîchement créée.
pub async fn insert(pool: &SqlitePool, user_id: i64, input: CreateSkillInput) -> Result<SkillView, AppError> {
    let (nom, domaine) = validate(&input)?;

    let row = sqlx::query(
        "INSERT INTO skills (user_id, nom, domaine) VALUES (?, ?, ?) \
         RETURNING id, nom, domaine",
    )
    .bind(user_id)
    .bind(&nom)
    .bind(&domaine)
    .fetch_one(pool)
    .await?;

    Ok(SkillView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        domaine: row.try_get("domaine").map_err(AppError::from)?,
    })
}

/// Met à jour une compétence existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la compétence à modifier.
/// * `input` - Nouvelles données de la compétence.
///
/// # Returns
///
/// `SkillView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateSkillInput) -> Result<SkillView, AppError> {
    let (nom, domaine) = validate(&input)?;

    let row = sqlx::query(
        "UPDATE skills SET nom = ?, domaine = ?, updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, nom, domaine",
    )
    .bind(&nom)
    .bind(&domaine)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Compétence introuvable.".to_string()))?;

    Ok(SkillView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        domaine: row.try_get("domaine").map_err(AppError::from)?,
    })
}

/// Supprime une compétence appartenant à l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la compétence à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    delete_owned_row(pool, "skills", id, "Compétence introuvable.").await
}
