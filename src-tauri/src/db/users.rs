//! Accès et logique de persistance pour l'utilisateur unique de l'application.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::normalize;
use crate::error::AppError;
use crate::models::{CreateUserInput, UpdateUserInfoInput, UserView};

/// Construit un `UserView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête `SELECT` sur la table `users`.
///
/// # Returns
///
/// `UserView` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_user(row: &SqliteRow) -> Result<UserView, sqlx::Error> {
    let permis_int: Option<i64> = row.try_get("permis")?;
    Ok(UserView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        prenom: row.try_get("prenom")?,
        email: row.try_get("email")?,
        telephone: row.try_get("telephone")?,
        ville: row.try_get("ville")?,
        permis: permis_int.map(|n| n != 0),
        description: row.try_get("description")?,
        linkedin: row.try_get("linkedin")?,
        github: row.try_get("github")?,
        portfolio: row.try_get("portfolio")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Récupère l'unique utilisateur du profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// `UserView`, ou `AppError::NotFound` si le profil n'a pas encore été créé.
pub async fn fetch_first_user(pool: &SqlitePool) -> Result<UserView, AppError> {
    let row = sqlx::query(
        "SELECT id, nom, prenom, email, telephone, ville, permis, description, \
         linkedin, github, portfolio, created_at, updated_at \
         FROM users LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    let row = row.ok_or_else(|| AppError::NotFound("Profil introuvable.".to_string()))?;
    row_to_user(&row).map_err(AppError::from)
}

/// Indique si un utilisateur a déjà été créé (utilisé pour déclencher l'onboarding).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// `true` si un utilisateur existe.
pub async fn user_exists(pool: &SqlitePool) -> Result<bool, AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(pool)
        .await?;
    Ok(n != 0)
}

/// Crée l'utilisateur unique de l'application.
///
/// Refuse la création si un utilisateur existe déjà (application mono-utilisateur),
/// et valide que nom, prénom et e-mail sont renseignés.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données saisies lors de l'onboarding.
///
/// # Returns
///
/// `UserView` fraîchement créé.
pub async fn insert_user(pool: &SqlitePool, input: CreateUserInput) -> Result<UserView, AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(pool)
        .await?;
    if n != 0 {
        return Err(AppError::Conflict("Un utilisateur existe déjà.".to_string()));
    }

    let nom = input.nom.trim().to_string();
    let prenom = input.prenom.trim().to_string();
    let email = input.email.trim().to_string();

    if nom.is_empty() {
        return Err(AppError::Validation("Le nom est requis.".to_string()));
    }
    if prenom.is_empty() {
        return Err(AppError::Validation("Le prénom est requis.".to_string()));
    }
    if email.is_empty() {
        return Err(AppError::Validation("L'e-mail est requis.".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::Validation("L'e-mail doit contenir un @.".to_string()));
    }

    let permis: Option<i64> = input.permis.map(|b| if b { 1 } else { 0 });

    sqlx::query(
        "INSERT INTO users \
           (nom, prenom, email, telephone, ville, permis, description, linkedin, github, portfolio) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&nom)
    .bind(&prenom)
    .bind(&email)
    .bind(normalize(input.telephone))
    .bind(normalize(input.ville))
    .bind(permis)
    .bind(normalize(input.description))
    .bind(normalize(input.linkedin))
    .bind(normalize(input.github))
    .bind(normalize(input.portfolio))
    .execute(pool)
    .await?;

    fetch_first_user(pool).await
}

/// Met à jour la description libre du profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `description` - Nouvelle description (normalisée avant écriture).
///
/// # Returns
///
/// `UserView` mis à jour.
pub async fn update_description(
    pool: &SqlitePool,
    description: Option<String>,
) -> Result<UserView, AppError> {
    let value = normalize(description);
    sqlx::query(
        "UPDATE users SET description = ?, updated_at = CURRENT_TIMESTAMP \
         WHERE id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(&value)
    .execute(pool)
    .await?;
    fetch_first_user(pool).await
}

/// Met à jour les informations de contact du profil.
///
/// Les champs `None` conservent leur valeur existante (`COALESCE`), à l'exception
/// de `telephone` et `permis` qui sont toujours remplacés (y compris par `None`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Champs à mettre à jour.
///
/// # Returns
///
/// `UserView` mis à jour.
pub async fn update_user_info(
    pool: &SqlitePool,
    input: UpdateUserInfoInput,
) -> Result<UserView, AppError> {
    let email = normalize(input.email);
    if let Some(ref e) = email {
        if !e.contains('@') {
            return Err(AppError::Validation("L'e-mail doit contenir un @.".to_string()));
        }
    }

    let permis: Option<i64> = input.permis.map(|b| if b { 1 } else { 0 });

    sqlx::query(
        "UPDATE users SET \
           prenom    = COALESCE(?, prenom), \
           nom       = COALESCE(?, nom), \
           email     = COALESCE(?, email), \
           telephone = ?, \
           permis    = ?, \
           updated_at = CURRENT_TIMESTAMP \
         WHERE id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(normalize(input.prenom))
    .bind(normalize(input.nom))
    .bind(email)
    .bind(normalize(input.telephone))
    .bind(permis)
    .execute(pool)
    .await?;

    fetch_first_user(pool).await
}
