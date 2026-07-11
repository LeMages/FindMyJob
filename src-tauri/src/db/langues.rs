//! Accès et logique de persistance pour les langues du profil.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::{delete_owned_row, normalize};
use crate::error::AppError;
use crate::models::{CreateLangInput, LangView};

/// Liste les langues d'un utilisateur, triées par nom.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Langues triées par nom.
pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<LangView>, AppError> {
    let rows = sqlx::query("SELECT id, nom, level FROM lang WHERE user_id = ? ORDER BY nom")
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    rows.iter()
        .map(|r| -> Result<LangView, sqlx::Error> {
            Ok(LangView {
                id: r.try_get("id")?,
                nom: r.try_get("nom")?,
                level: r.try_get("level")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}

/// Ajoute une langue au profil d'un utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la langue à créer.
///
/// # Returns
///
/// `LangView` fraîchement créée.
pub async fn insert(pool: &SqlitePool, user_id: i64, input: CreateLangInput) -> Result<LangView, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO lang (user_id, nom, level) VALUES (?, ?, ?) \
         RETURNING id, nom, level",
    )
    .bind(user_id)
    .bind(&nom)
    .bind(normalize(input.level))
    .fetch_one(pool)
    .await?;

    Ok(LangView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        level: row.try_get("level").map_err(AppError::from)?,
    })
}

/// Met à jour une langue existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la langue à modifier.
/// * `input` - Nouvelles données de la langue.
///
/// # Returns
///
/// `LangView` mise à jour, ou `AppError::NotFound` si absente.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateLangInput) -> Result<LangView, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE lang SET nom = ?, level = ? \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, nom, level",
    )
    .bind(&nom)
    .bind(normalize(input.level))
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Langue introuvable.".to_string()))?;

    Ok(LangView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        level: row.try_get("level").map_err(AppError::from)?,
    })
}

/// Supprime une langue appartenant à l'utilisateur.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la langue à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    delete_owned_row(pool, "lang", id, "Langue introuvable.").await
}
