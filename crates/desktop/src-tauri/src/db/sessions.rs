//! Accès et logique de persistance pour les sessions de candidature.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::db::transaction::begin_write;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{SessionView, UpdateSessionInput};

/// Valeurs acceptées pour `sessions.statut` (miroir de la contrainte CHECK en base).
const VALID_STATUTS: [&str; 3] = ["active", "pause", "termine"];

/// Requête agrégée sessions + statistiques (candidatures, acceptation),
/// partagée par le listing et la mise à jour d'une session.
const SELECT_WITH_STATS: &str = "SELECT s.id, s.nom, s.statut, s.created_at, \
            (SELECT COUNT(*) FROM candidatures c \
             WHERE c.session_id = s.id \
               AND c.status NOT IN ('abandon', 'ghosting', 'refus', 'offre_refusee')) AS candidature_count, \
            EXISTS(SELECT 1 FROM candidatures c \
                   WHERE c.session_id = s.id AND c.status IN ('accepte', 'offre_refusee', 'refus')) AS has_accepted \
     FROM sessions s";

/// Convertit une ligne de `SELECT_WITH_STATS` en `SessionView`.
///
/// # Arguments
///
/// * `row` - Ligne retournée par une requête basée sur `SELECT_WITH_STATS`.
///
/// # Returns
///
/// `SessionView` correspondante, ou erreur si une colonne attendue est absente.
fn row_to_session_view(row: &SqliteRow) -> Result<SessionView, sqlx::Error> {
    Ok(SessionView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        statut: row.try_get("statut")?,
        candidature_count: row.try_get("candidature_count")?,
        has_accepted: row.try_get("has_accepted")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Valide le nom d'une session.
///
/// # Arguments
///
/// * `nom` - Nom saisi.
///
/// # Returns
///
/// Nom trimé, ou `AppError::Validation` s'il est vide.
fn validate_nom(nom: &str) -> Result<String, AppError> {
    let nom = nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la session est requis.".to_string()));
    }
    Ok(nom)
}

/// Lit une session avec ses statistiques.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant de la session.
///
/// # Returns
///
/// `SessionView`, ou `AppError::NotFound` si la session n'existe pas.
async fn fetch_view<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<SessionView, AppError> {
    let row = sqlx::query(&format!("{SELECT_WITH_STATS} WHERE s.id = $1"))
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Session introuvable.".to_string()))?;
    row_to_session_view(&row).map_err(map_db_err)
}

/// Crée une nouvelle session de candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `nom` - Nom de la session.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `SessionView` fraîchement créée (compteurs à zéro).
pub async fn insert(pool: &SqlitePool, nom: &str, now: &str) -> Result<SessionView, AppError> {
    let nom = validate_nom(nom)?;

    let row = sqlx::query(
        "INSERT INTO sessions (nom, created_at) VALUES ($1, $2) \
         RETURNING id, nom, statut, created_at",
    )
    .bind(&nom)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(SessionView {
        id: row.try_get("id").map_err(map_db_err)?,
        nom: row.try_get("nom").map_err(map_db_err)?,
        statut: row.try_get("statut").map_err(map_db_err)?,
        candidature_count: 0,
        has_accepted: false,
        created_at: row.try_get("created_at").map_err(map_db_err)?,
    })
}

/// Liste toutes les sessions avec leurs statistiques agrégées.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Sessions triées par date de création décroissante, avec le nombre de
/// candidatures et si la session comporte une candidature acceptée/refusée/abandonnée.
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<SessionView>, AppError> {
    let sql = format!("{SELECT_WITH_STATS} ORDER BY s.created_at DESC, s.id DESC");
    let rows = sqlx::query(&sql).fetch_all(pool).await.map_err(map_db_err)?;

    rows.iter().map(row_to_session_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Met à jour le nom et le statut d'une session existante.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session à modifier.
/// * `input` - Nouvelles valeurs des champs éditables de la session.
///
/// # Returns
///
/// `SessionView` mise à jour, ou `AppError::NotFound` si la session n'existe pas.
pub async fn update(pool: &SqlitePool, id: i64, input: &UpdateSessionInput) -> Result<SessionView, AppError> {
    let nom = validate_nom(&input.nom)?;
    if !VALID_STATUTS.contains(&input.statut.as_str()) {
        return Err(AppError::Validation("Statut de session invalide.".to_string()));
    }

    let mut tx = begin_write(pool).await?;
    let result = sqlx::query("UPDATE sessions SET nom = $1, statut = $2 WHERE id = $3")
        .bind(&nom)
        .bind(&input.statut)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session introuvable.".to_string()));
    }
    let view = fetch_view(&mut *tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Supprime une session et toutes les données qui en dépendent
/// (candidatures, historique, liaisons aux contacts, entretiens, versions et
/// références de documents ; les contacts restent dans le carnet), par les `ON DELETE CASCADE` du schéma. Les fichiers de la
/// session sont supprimés par l'appelant.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session à supprimer.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session introuvable.".to_string()));
    }
    Ok(())
}
