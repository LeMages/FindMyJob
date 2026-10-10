//! Accès et logique de persistance pour les langues du profil.
//!
//! Le nom d'une langue est unique en base : un doublon est détecté avant
//! l'écriture, dans la même transaction, pour renvoyer un message clair
//! plutôt que l'erreur brute de la contrainte SQLite.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqlitePool};

// Importation des modules internes
use crate::db::common::{delete_by_id, map_db_err, normalize};
use crate::db::transaction::begin_write;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateLangInput, LangView};

/// Valide le nom d'une langue.
///
/// # Arguments
///
/// * `input` - Données saisies.
///
/// # Returns
///
/// Nom trimé, ou `AppError::Validation` s'il est vide.
fn validate_nom(input: &CreateLangInput) -> Result<String, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }
    Ok(nom)
}

/// Construit un `LangView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne sélectionnant `id, nom, level`.
///
/// # Returns
///
/// `LangView` reconstruite, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &SqliteRow) -> Result<LangView, sqlx::Error> {
    Ok(LangView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        level: row.try_get("level")?,
    })
}

/// Refuse l'écriture si une autre langue porte déjà ce nom, comparé comme
/// la contrainte `UNIQUE` (exact).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `nom` - Nom trimé.
/// * `exclude_id` - Langue modifiée, exclue du contrôle (`None` à l'ajout).
///
/// # Returns
///
/// `Ok(())` si le nom est libre, `AppError::Conflict` sinon.
async fn ensure_unique(conn: &mut SqliteConnection, nom: &str, exclude_id: Option<i64>) -> Result<(), AppError> {
    let taken: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM langues WHERE nom = $1 AND id IS NOT $2)")
        .bind(nom)
        .bind(exclude_id)
        .fetch_one(conn)
        .await
        .map_err(map_db_err)?;

    if taken {
        return Err(AppError::Conflict(format!("La langue « {nom} » est déjà dans votre profil.")));
    }
    Ok(())
}

/// Liste les langues du profil, triées par nom.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Langues triées par nom, sans tenir compte de la casse.
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<LangView>, AppError> {
    let rows = sqlx::query("SELECT id, nom, level FROM langues ORDER BY nom COLLATE NOCASE, id")
        .fetch_all(pool)
        .await
        .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Ajoute une langue au profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données de la langue à créer.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `LangView` fraîchement créée, ou `AppError::Conflict` si elle existe déjà.
pub async fn insert(pool: &SqlitePool, input: CreateLangInput, now: &str) -> Result<LangView, AppError> {
    let nom = validate_nom(&input)?;

    let mut tx = begin_write(pool).await?;
    ensure_unique(&mut tx, &nom, None).await?;
    let row = sqlx::query(
        "INSERT INTO langues (nom, level, created_at) VALUES ($1, $2, $3) \
         RETURNING id, nom, level",
    )
    .bind(&nom)
    .bind(normalize(input.level))
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_err)?;
    let view = row_to_view(&row).map_err(map_db_err)?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
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
/// `LangView` mise à jour, `AppError::NotFound` si absente, ou
/// `AppError::Conflict` si une autre langue porte déjà ce nom.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateLangInput) -> Result<LangView, AppError> {
    let nom = validate_nom(&input)?;

    let mut tx = begin_write(pool).await?;
    ensure_unique(&mut tx, &nom, Some(id)).await?;
    let row = sqlx::query("UPDATE langues SET nom = $1, level = $2 WHERE id = $3 RETURNING id, nom, level")
        .bind(&nom)
        .bind(normalize(input.level))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err)?;
    let row = row.ok_or_else(|| AppError::NotFound("Langue introuvable.".to_string()))?;
    let view = row_to_view(&row).map_err(map_db_err)?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Supprime une langue.
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
    delete_by_id(pool, "langues", id, "Langue introuvable.").await
}
