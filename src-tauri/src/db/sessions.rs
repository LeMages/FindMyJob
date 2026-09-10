//! Accès et logique de persistance pour les sessions de candidature.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::{assert_owned_by_user, normalize};
use crate::error::AppError;
use crate::models::{SessionView, UpdateSessionInput};

/// Valeurs acceptées pour `sessions.statut` (miroir de la contrainte CHECK en base).
const VALID_STATUTS: [&str; 3] = ["active", "pause", "termine"];

/// Requête agrégée sessions + statistiques (candidatures, acceptation),
/// partagée par le listing et la mise à jour d'une session.
const SELECT_WITH_STATS: &str =
    "SELECT s.id, s.nom, s.contexte, s.statut, s.zone, s.created_at, \
            (SELECT COUNT(*) FROM candidatures c \
             WHERE c.session_id = s.id \
               AND c.status NOT IN ('abandon', 'ghosting', 'refus', 'offre_refusee')) AS candidature_count, \
            (SELECT CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END \
             FROM candidatures c \
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
fn row_to_session_view(row: &SqliteRow) -> Result<SessionView, AppError> {
    let has_accepted_int: i64 = row.try_get("has_accepted").unwrap_or(0);
    Ok(SessionView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        contexte: row.try_get("contexte").map_err(AppError::from)?,
        statut: row.try_get("statut").map_err(AppError::from)?,
        zone: row.try_get("zone").map_err(AppError::from)?,
        candidature_count: row.try_get("candidature_count").unwrap_or(0),
        has_accepted: has_accepted_int != 0,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

/// Crée une nouvelle session de candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `nom` - Nom de la session.
///
/// # Returns
///
/// `SessionView` fraîchement créée (compteurs à zéro).
pub async fn insert(pool: &SqlitePool, user_id: i64, nom: &str) -> Result<SessionView, AppError> {
    let nom = nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la session est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO sessions (user_id, nom) VALUES (?, ?) \
         RETURNING id, nom, contexte, statut, zone, created_at",
    )
    .bind(user_id)
    .bind(&nom)
    .fetch_one(pool)
    .await?;

    Ok(SessionView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        contexte: row.try_get("contexte").map_err(AppError::from)?,
        statut: row.try_get("statut").map_err(AppError::from)?,
        zone: row.try_get("zone").map_err(AppError::from)?,
        candidature_count: 0,
        has_accepted: false,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

/// Liste les sessions d'un utilisateur avec leurs statistiques agrégées.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur.
///
/// # Returns
///
/// Sessions triées par date de création décroissante, avec le nombre de
/// candidatures et si la session comporte une candidature acceptée/refusée/abandonnée.
pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<SessionView>, AppError> {
    let sql = format!("{SELECT_WITH_STATS} WHERE s.user_id = ? ORDER BY s.created_at DESC");
    let rows = sqlx::query(&sql).bind(user_id).fetch_all(pool).await?;

    rows.iter().map(row_to_session_view).collect()
}

/// Met à jour le nom, le contexte, le statut et la zone cible d'une session existante.
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
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: &UpdateSessionInput,
) -> Result<SessionView, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la session est requis.".to_string()));
    }
    if !VALID_STATUTS.contains(&input.statut.as_str()) {
        return Err(AppError::Validation("Statut de session invalide.".to_string()));
    }
    let contexte = normalize(input.contexte.clone());
    let zone = normalize(input.zone.clone());

    assert_owned_by_user(pool, "sessions", id, "Session introuvable.").await?;

    sqlx::query("UPDATE sessions SET nom = ?, contexte = ?, statut = ?, zone = ? WHERE id = ?")
        .bind(&nom)
        .bind(&contexte)
        .bind(&input.statut)
        .bind(&zone)
        .bind(id)
        .execute(pool)
        .await?;

    let sql = format!("{SELECT_WITH_STATS} WHERE s.id = ?");
    let row = sqlx::query(&sql).bind(id).fetch_one(pool).await?;
    row_to_session_view(&row)
}

/// Supprime une session et toutes les données qui en dépendent (candidatures,
/// emails, historique de statut, contacts, entretiens).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session à supprimer.
///
/// # Returns
///
/// `Ok(())` si la suppression (transactionnelle) a réussi, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    // ------------------------------------------------------------------
    // 1. Vérification de propriété
    // ------------------------------------------------------------------
    assert_owned_by_user(pool, "sessions", id, "Session introuvable.").await?;

    let mut tx = pool.begin().await?;

    // ------------------------------------------------------------------
    // 2. Suppression des dépendances des candidatures de la session
    //    (ordre imposé par les contraintes de clé étrangère)
    // ------------------------------------------------------------------
    sqlx::query(
        "DELETE FROM emails \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM status_history \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM contacts \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM interviews \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // ------------------------------------------------------------------
    // 3. Suppression des candidatures puis de la session elle-même
    // ------------------------------------------------------------------
    sqlx::query("DELETE FROM candidatures WHERE session_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
