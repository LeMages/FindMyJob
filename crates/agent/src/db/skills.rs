//! Accès et logique de persistance pour les compétences du profil.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{delete_owned_row, map_db_err};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateSkillInput, SkillView};

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
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Compétences triées par domaine puis nom.
pub async fn list_for_user(pool: &PgPool, user_id: i64) -> Result<Vec<SkillView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, domaine, nom FROM skills WHERE user_id = $1 ORDER BY domaine, nom",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<SkillView, sqlx::Error> {
            Ok(SkillView {
                id: r.try_get("id")?,
                domaine: r.try_get("domaine")?,
                nom: r.try_get("nom")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Ajoute une compétence au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la compétence à créer.
///
/// # Returns
///
/// `SkillView` fraîchement créée.
pub async fn insert(pool: &PgPool, user_id: i64, input: CreateSkillInput) -> Result<SkillView, AppError> {
    let (nom, domaine) = validate(&input)?;

    let row = sqlx::query(
        "INSERT INTO skills (user_id, nom, domaine) VALUES ($1, $2, $3) \
         RETURNING id, nom, domaine",
    )
    .bind(user_id)
    .bind(&nom)
    .bind(&domaine)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(SkillView {
        id: row.try_get("id").map_err(map_db_err)?,
        nom: row.try_get("nom").map_err(map_db_err)?,
        domaine: row.try_get("domaine").map_err(map_db_err)?,
    })
}

/// Met à jour une compétence existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la compétence à modifier.
/// * `input` - Nouvelles données de la compétence.
///
/// # Returns
///
/// `SkillView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(pool: &PgPool, id: i64, input: CreateSkillInput) -> Result<SkillView, AppError> {
    let (nom, domaine) = validate(&input)?;

    let row = sqlx::query(
        "UPDATE skills SET nom = $1, domaine = $2 \
         WHERE id = $3 AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, nom, domaine",
    )
    .bind(&nom)
    .bind(&domaine)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Compétence introuvable.".to_string()))?;

    Ok(SkillView {
        id: row.try_get("id").map_err(map_db_err)?,
        nom: row.try_get("nom").map_err(map_db_err)?,
        domaine: row.try_get("domaine").map_err(map_db_err)?,
    })
}

/// Supprime une compétence appartenant à l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la compétence à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    delete_owned_row(pool, "skills", id, "Compétence introuvable.").await
}
