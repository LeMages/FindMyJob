//! Accès et logique de persistance pour les compétences du profil.
//!
//! Le couple (domaine, nom) est unique en base : un doublon est détecté
//! avant l'écriture, dans la même transaction, pour renvoyer un message
//! clair plutôt que l'erreur brute de la contrainte SQLite.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqlitePool};

// Importation des modules internes
use crate::db::common::{delete_by_id, map_db_err};
use crate::db::transaction::begin_write;
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

/// Construit un `SkillView` à partir d'une ligne SQL brute.
///
/// # Arguments
///
/// * `row` - Ligne sélectionnant `id, domaine, nom`.
///
/// # Returns
///
/// `SkillView` reconstruite, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &SqliteRow) -> Result<SkillView, sqlx::Error> {
    Ok(SkillView {
        id: row.try_get("id")?,
        domaine: row.try_get("domaine")?,
        nom: row.try_get("nom")?,
    })
}

/// Refuse l'écriture si une autre compétence porte déjà ce couple
/// (domaine, nom), comparé comme la contrainte `UNIQUE` (exact).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `domaine` - Domaine trimé.
/// * `nom` - Nom trimé.
/// * `exclude_id` - Compétence modifiée, exclue du contrôle (`None` à l'ajout).
///
/// # Returns
///
/// `Ok(())` si le couple est libre, `AppError::Conflict` sinon.
async fn ensure_unique(
    conn: &mut SqliteConnection,
    domaine: &str,
    nom: &str,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    let taken: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM skills WHERE domaine = $1 AND nom = $2 AND id IS NOT $3)",
    )
    .bind(domaine)
    .bind(nom)
    .bind(exclude_id)
    .fetch_one(conn)
    .await
    .map_err(map_db_err)?;

    if taken {
        return Err(AppError::Conflict(format!(
            "La compétence « {nom} » existe déjà dans le domaine « {domaine} »."
        )));
    }
    Ok(())
}

/// Liste les compétences du profil, groupées par domaine puis triées par nom.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Compétences triées par domaine puis nom, sans tenir compte de la casse.
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<SkillView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, domaine, nom FROM skills \
         ORDER BY domaine COLLATE NOCASE, nom COLLATE NOCASE, id",
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Ajoute une compétence au profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Données de la compétence à créer.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `SkillView` fraîchement créée, ou `AppError::Conflict` si elle existe déjà.
pub async fn insert(pool: &SqlitePool, input: CreateSkillInput, now: &str) -> Result<SkillView, AppError> {
    let (nom, domaine) = validate(&input)?;

    let mut tx = begin_write(pool).await?;
    ensure_unique(&mut tx, &domaine, &nom, None).await?;
    let row = sqlx::query(
        "INSERT INTO skills (nom, domaine, created_at) VALUES ($1, $2, $3) \
         RETURNING id, domaine, nom",
    )
    .bind(&nom)
    .bind(&domaine)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_err)?;
    let view = row_to_view(&row).map_err(map_db_err)?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
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
/// `SkillView` mise à jour, `AppError::NotFound` si absente, ou
/// `AppError::Conflict` si une autre compétence porte déjà ce couple.
pub async fn update(pool: &SqlitePool, id: i64, input: CreateSkillInput) -> Result<SkillView, AppError> {
    let (nom, domaine) = validate(&input)?;

    let mut tx = begin_write(pool).await?;
    ensure_unique(&mut tx, &domaine, &nom, Some(id)).await?;
    let row = sqlx::query(
        "UPDATE skills SET nom = $1, domaine = $2 WHERE id = $3 \
         RETURNING id, domaine, nom",
    )
    .bind(&nom)
    .bind(&domaine)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_err)?;
    let row = row.ok_or_else(|| AppError::NotFound("Compétence introuvable.".to_string()))?;
    let view = row_to_view(&row).map_err(map_db_err)?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Supprime une compétence.
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
    delete_by_id(pool, "skills", id, "Compétence introuvable.").await
}
