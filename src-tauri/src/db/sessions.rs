//! Accès et logique de persistance pour les sessions de candidature.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::assert_owned_by_user;
use crate::error::AppError;
use crate::models::SessionView;

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
         RETURNING id, nom, created_at",
    )
    .bind(user_id)
    .bind(&nom)
    .fetch_one(pool)
    .await?;

    Ok(SessionView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
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
    let rows = sqlx::query(
        "SELECT s.id, s.nom, s.created_at, \
                (SELECT COUNT(*) FROM candidatures c WHERE c.session_id = s.id) AS candidature_count, \
                (SELECT CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END \
                 FROM candidatures c \
                 WHERE c.session_id = s.id AND c.status IN ('offre', 'refus', 'abandon')) AS has_accepted \
         FROM sessions s \
         WHERE s.user_id = ? \
         ORDER BY s.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<SessionView, sqlx::Error> {
            let has_accepted_int: i64 = r.try_get("has_accepted").unwrap_or(0);
            Ok(SessionView {
                id: r.try_get("id")?,
                nom: r.try_get("nom")?,
                candidature_count: r.try_get("candidature_count").unwrap_or(0),
                has_accepted: has_accepted_int != 0,
                created_at: r.try_get("created_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
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
