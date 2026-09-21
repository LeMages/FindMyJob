//! Accès et logique de persistance pour l'utilisateur unique de l'application.

// Importation des crates externes
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{map_db_err, normalize};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateUserInput, UpdateUserInfoInput, UserView};

/// Construit un `UserView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête `SELECT` sur la table `users`.
///
/// # Returns
///
/// `UserView` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_user(row: &PgRow) -> Result<UserView, sqlx::Error> {
    Ok(UserView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        prenom: row.try_get("prenom")?,
        email: row.try_get("email")?,
        telephone: row.try_get("telephone")?,
        ville: row.try_get("ville")?,
        permis: row.try_get("permis")?,
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
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// `UserView`, ou `AppError::NotFound` si le profil n'a pas encore été créé.
pub async fn fetch_first_user(pool: &PgPool) -> Result<UserView, AppError> {
    let row = sqlx::query(
        "SELECT id, nom, prenom, email, telephone, ville, permis, description, \
         linkedin, github, portfolio, created_at, updated_at \
         FROM profile.users LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?;
    let row = row.ok_or_else(|| AppError::NotFound("Profil introuvable.".to_string()))?;
    row_to_user(&row).map_err(map_db_err)
}

/// Indique si un utilisateur a déjà été créé (utilisé pour déclencher l'onboarding).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// `true` si un utilisateur existe.
pub async fn user_exists(pool: &PgPool) -> Result<bool, AppError> {
    let n: bool = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM profile.users)")
        .fetch_one(pool)
        .await
        .map_err(map_db_err)?;
    Ok(n)
}

/// Crée l'utilisateur unique de l'application.
///
/// Refuse la création si un utilisateur existe déjà (application mono-utilisateur),
/// et valide que nom, prénom et e-mail sont renseignés.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `input` - Données saisies lors de l'onboarding.
///
/// # Returns
///
/// `UserView` fraîchement créé.
pub async fn insert_user(pool: &PgPool, input: CreateUserInput) -> Result<UserView, AppError> {
    let exists: bool = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM profile.users)")
        .fetch_one(pool)
        .await
        .map_err(map_db_err)?;
    if exists {
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

    sqlx::query(
        "INSERT INTO profile.users \
           (nom, prenom, email, telephone, ville, permis, description, linkedin, github, portfolio) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&nom)
    .bind(&prenom)
    .bind(&email)
    .bind(normalize(input.telephone))
    .bind(normalize(input.ville))
    .bind(input.permis)
    .bind(normalize(input.description))
    .bind(normalize(input.linkedin))
    .bind(normalize(input.github))
    .bind(normalize(input.portfolio))
    .execute(pool)
    .await
    .map_err(map_db_err)?;

    fetch_first_user(pool).await
}

/// Met à jour la description libre du profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `description` - Nouvelle description (normalisée avant écriture).
///
/// # Returns
///
/// `UserView` mis à jour.
pub async fn update_description(
    pool: &PgPool,
    description: Option<String>,
) -> Result<UserView, AppError> {
    let value = normalize(description);
    sqlx::query(
        "UPDATE profile.users SET description = $1 \
         WHERE id = (SELECT id FROM profile.users LIMIT 1)",
    )
    .bind(&value)
    .execute(pool)
    .await
    .map_err(map_db_err)?;
    fetch_first_user(pool).await
}

/// Met à jour les informations de contact du profil.
///
/// Les champs `None` conservent leur valeur existante (`COALESCE`), à l'exception
/// de `telephone` et `permis` qui sont toujours remplacés (y compris par `None`).
/// `linkedin`, `github` et `portfolio` suivent le même comportement `COALESCE` que
/// `prenom`/`nom`/`email` : un champ vidé côté client (`None`) ne l'efface pas en base.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `input` - Champs à mettre à jour.
///
/// # Returns
///
/// `UserView` mis à jour.
pub async fn update_user_info(
    pool: &PgPool,
    input: UpdateUserInfoInput,
) -> Result<UserView, AppError> {
    let email = normalize(input.email);
    if let Some(ref e) = email {
        if !e.contains('@') {
            return Err(AppError::Validation("L'e-mail doit contenir un @.".to_string()));
        }
    }

    sqlx::query(
        "UPDATE profile.users SET \
           prenom    = COALESCE($1, prenom), \
           nom       = COALESCE($2, nom), \
           email     = COALESCE($3, email), \
           telephone = $4, \
           permis    = $5, \
           linkedin  = COALESCE($6, linkedin), \
           github    = COALESCE($7, github), \
           portfolio = COALESCE($8, portfolio) \
         WHERE id = (SELECT id FROM profile.users LIMIT 1)",
    )
    .bind(normalize(input.prenom))
    .bind(normalize(input.nom))
    .bind(email)
    .bind(normalize(input.telephone))
    .bind(input.permis)
    .bind(normalize(input.linkedin))
    .bind(normalize(input.github))
    .bind(normalize(input.portfolio))
    .execute(pool)
    .await
    .map_err(map_db_err)?;

    fetch_first_user(pool).await
}
