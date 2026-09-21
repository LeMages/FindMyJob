//! Accès et logique de persistance pour les langues du profil.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{delete_owned_row, map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateLangInput, LangView};

/// Liste les langues d'un utilisateur, triées par nom.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Langues triées par nom.
pub async fn list_for_user(pool: &PgPool, user_id: i64) -> Result<Vec<LangView>, AppError> {
    let rows = sqlx::query("SELECT id, nom, level FROM profile.lang WHERE user_id = $1 ORDER BY nom")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<LangView, sqlx::Error> {
            Ok(LangView {
                id: r.try_get("id")?,
                nom: r.try_get("nom")?,
                level: r.try_get("level")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Ajoute une langue au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la langue à créer.
///
/// # Returns
///
/// `LangView` fraîchement créée.
pub async fn insert(pool: &PgPool, user_id: i64, input: CreateLangInput) -> Result<LangView, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO profile.lang (user_id, nom, level) VALUES ($1, $2, $3) \
         RETURNING id, nom, level",
    )
    .bind(user_id)
    .bind(&nom)
    .bind(normalize(input.level))
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(LangView {
        id: row.try_get("id").map_err(map_db_err)?,
        nom: row.try_get("nom").map_err(map_db_err)?,
        level: row.try_get("level").map_err(map_db_err)?,
    })
}

/// Met à jour une langue existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la langue à modifier.
/// * `input` - Nouvelles données de la langue.
///
/// # Returns
///
/// `LangView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(pool: &PgPool, id: i64, input: CreateLangInput) -> Result<LangView, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE profile.lang SET nom = $1, level = $2 \
         WHERE id = $3 AND user_id = (SELECT id FROM profile.users LIMIT 1) \
         RETURNING id, nom, level",
    )
    .bind(&nom)
    .bind(normalize(input.level))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;

    let row = row.ok_or_else(|| AppError::NotFound("Langue introuvable.".to_string()))?;

    Ok(LangView {
        id: row.try_get("id").map_err(map_db_err)?,
        nom: row.try_get("nom").map_err(map_db_err)?,
        level: row.try_get("level").map_err(map_db_err)?,
    })
}

/// Supprime une langue appartenant à l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la langue à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    delete_owned_row(pool, "profile.lang", id, "Langue introuvable.").await
}
