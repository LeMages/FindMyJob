//! Accès et logique de persistance pour l'utilisateur unique de l'application
//! (ligne `users.id = 1`, imposée par la contrainte `CHECK (id = 1)`).

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::{map_db_err, normalize};
use crate::db::transaction::begin_write;
use crate::error::AppError;
use crate::models::{CreateUserInput, UpdateUserInfoInput, UserView};

/// Identifiant fixe de l'utilisateur unique.
const USER_ID: i64 = 1;

/// Colonnes lues pour reconstruire un `UserView`.
const USER_COLUMNS: &str = "id, nom, prenom, email, telephone, ville, zip_code, permis, \
     description, linkedin, github, portfolio, interets, langue_profil, created_at";

/// Construit un `UserView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne issue d'une requête `SELECT` sur `USER_COLUMNS`.
///
/// # Returns
///
/// `UserView` reconstruit, ou erreur si une colonne attendue est absente.
fn row_to_user(row: &SqliteRow) -> Result<UserView, sqlx::Error> {
    Ok(UserView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        prenom: row.try_get("prenom")?,
        email: row.try_get("email")?,
        telephone: row.try_get("telephone")?,
        ville: row.try_get("ville")?,
        zip_code: row.try_get("zip_code")?,
        permis: row.try_get("permis")?,
        description: row.try_get("description")?,
        linkedin: row.try_get("linkedin")?,
        github: row.try_get("github")?,
        portfolio: row.try_get("portfolio")?,
        interets: row.try_get("interets")?,
        langue_profil: row.try_get("langue_profil")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Récupère l'utilisateur unique du profil.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
///
/// # Returns
///
/// `UserView`, ou `AppError::NotFound` si le profil n'a pas encore été créé.
pub async fn fetch_user<'e, E: SqliteExecutor<'e>>(executor: E) -> Result<UserView, AppError> {
    let row = sqlx::query(&format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1"))
        .bind(USER_ID)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?;
    let row = row.ok_or_else(|| AppError::NotFound("Profil introuvable.".to_string()))?;
    row_to_user(&row).map_err(map_db_err)
}

/// Indique si l'utilisateur a déjà été créé (déclenche l'onboarding sinon).
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
///
/// # Returns
///
/// `true` si l'utilisateur existe.
pub async fn user_exists<'e, E: SqliteExecutor<'e>>(executor: E) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(executor)
        .await
        .map_err(map_db_err)
}

/// Valide les champs obligatoires de l'onboarding.
///
/// # Arguments
///
/// * `input` - Données saisies lors de l'onboarding.
///
/// # Returns
///
/// Nom, prénom et e-mail trimés, ou `AppError::Validation`.
fn validate_identity(input: &CreateUserInput) -> Result<(String, String, String), AppError> {
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
    Ok((nom, prenom, email))
}

/// Crée l'utilisateur unique de l'application.
///
/// Refuse la création si un utilisateur existe déjà (application
/// mono-utilisateur), et valide que nom, prénom et e-mail sont renseignés.
/// Contrôle et insertion partagent la même transaction immédiate.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données saisies lors de l'onboarding.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `UserView` fraîchement créé.
pub async fn insert_user(pool: &SqlitePool, input: CreateUserInput, now: &str) -> Result<UserView, AppError> {
    let mut tx = begin_write(pool).await?;

    if user_exists(&mut *tx).await? {
        return Err(AppError::Conflict("Un utilisateur existe déjà.".to_string()));
    }
    let (nom, prenom, email) = validate_identity(&input)?;

    sqlx::query(
        "INSERT INTO users \
           (id, nom, prenom, email, telephone, ville, zip_code, permis, description, linkedin, github, \
            portfolio, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(USER_ID)
    .bind(&nom)
    .bind(&prenom)
    .bind(&email)
    .bind(normalize(input.telephone))
    .bind(normalize(input.ville))
    .bind(normalize(input.zip_code))
    .bind(input.permis)
    .bind(normalize(input.description))
    .bind(normalize(input.linkedin))
    .bind(normalize(input.github))
    .bind(normalize(input.portfolio))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    let user = fetch_user(&mut *tx).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(user)
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
/// `UserView` mis à jour, ou `AppError::NotFound` si le profil n'existe pas.
pub async fn update_description(pool: &SqlitePool, description: Option<String>) -> Result<UserView, AppError> {
    let mut tx = begin_write(pool).await?;
    sqlx::query("UPDATE users SET description = $1 WHERE id = $2")
        .bind(normalize(description))
        .bind(USER_ID)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    let user = fetch_user(&mut *tx).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(user)
}

/// Met à jour la langue dans laquelle le profil est rédigé.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `lang` - Code de langue, déjà validé par l'appelant.
///
/// # Returns
///
/// `UserView` mis à jour, ou `AppError::NotFound` si le profil n'existe pas.
pub async fn update_profile_language(pool: &SqlitePool, lang: &str) -> Result<UserView, AppError> {
    let mut tx = begin_write(pool).await?;
    sqlx::query("UPDATE users SET langue_profil = $1 WHERE id = $2")
        .bind(lang)
        .bind(USER_ID)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    let user = fetch_user(&mut *tx).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(user)
}

/// Met à jour les informations de contact du profil.
///
/// Les champs `None` conservent leur valeur existante (`COALESCE`), à
/// l'exception de `telephone`, `ville`, `zip_code`, `interets` et `permis`
/// qui sont toujours remplacés (y compris par `None`). `ville` et `zip_code`
/// sont seulement nettoyés (trim, chaîne vide → NULL), jamais validés ni
/// formatés.
/// `linkedin`, `github` et `portfolio` suivent le même comportement
/// `COALESCE` que `prenom`/`nom`/`email` : un champ vidé côté client
/// (`None`) ne l'efface pas en base.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Champs à mettre à jour.
///
/// # Returns
///
/// `UserView` mis à jour, ou `AppError::NotFound` si le profil n'existe pas.
pub async fn update_user_info(pool: &SqlitePool, input: UpdateUserInfoInput) -> Result<UserView, AppError> {
    let email = normalize(input.email);
    if let Some(ref e) = email {
        if !e.contains('@') {
            return Err(AppError::Validation("L'e-mail doit contenir un @.".to_string()));
        }
    }

    let mut tx = begin_write(pool).await?;
    sqlx::query(
        "UPDATE users SET \
           prenom    = COALESCE($1, prenom), \
           nom       = COALESCE($2, nom), \
           email     = COALESCE($3, email), \
           telephone = $4, \
           permis    = $5, \
           linkedin  = COALESCE($6, linkedin), \
           github    = COALESCE($7, github), \
           portfolio = COALESCE($8, portfolio), \
           ville     = $9, \
           zip_code  = $10, \
           interets  = $11 \
         WHERE id = $12",
    )
    .bind(normalize(input.prenom))
    .bind(normalize(input.nom))
    .bind(email)
    .bind(normalize(input.telephone))
    .bind(input.permis)
    .bind(normalize(input.linkedin))
    .bind(normalize(input.github))
    .bind(normalize(input.portfolio))
    .bind(normalize(input.ville))
    .bind(normalize(input.zip_code))
    .bind(normalize(input.interets))
    .bind(USER_ID)
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    let user = fetch_user(&mut *tx).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(user)
}
