//! Helpers partagés par les modules `db::*` : normalisation des chaînes
//! utilisateur et suppression générique d'une ligne appartenant à
//! l'utilisateur unique de l'application.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::error::AppError;

/// Normalise une entrée utilisateur optionnelle : trim, puis `None` si vide.
///
/// # Arguments
///
/// * `s` - Chaîne optionnelle telle que reçue du frontend.
///
/// # Returns
///
/// `None` si absente ou vide après trim, sinon la chaîne trimée.
pub fn normalize(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    })
}

/// Vérifie qu'une ligne appartient à l'utilisateur unique de l'application.
///
/// Toutes les tables enfants du profil (experiences, formations, projets,
/// skills, lang, sessions) partagent le même schéma de propriété : une
/// colonne `user_id` sur la table elle-même.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `table` - Nom de la table cible. Doit être un littéral interne au code,
///   jamais une valeur fournie par l'utilisateur (construit par `format!` dans la requête).
/// * `id` - Identifiant de la ligne à vérifier.
/// * `not_found_message` - Message d'erreur renvoyé si la ligne n'existe pas.
///
/// # Returns
///
/// `Ok(())` si la ligne appartient à l'utilisateur, `AppError::NotFound` sinon.
pub async fn assert_owned_by_user(
    pool: &SqlitePool,
    table: &str,
    id: i64,
    not_found_message: &str,
) -> Result<(), AppError> {
    let sql = format!(
        "SELECT COUNT(*) FROM {table} WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)"
    );
    let n: i64 = sqlx::query_scalar::<_, i64>(&sql)
        .bind(id)
        .fetch_one(pool)
        .await?;

    if n == 0 {
        return Err(AppError::NotFound(not_found_message.to_string()));
    }
    Ok(())
}

/// Supprime une ligne après avoir vérifié qu'elle appartient à l'utilisateur.
///
/// Factorise le triptyque « vérifier la propriété → erreur si absente → supprimer »
/// répété à l'identique pour experiences, formations, projets, skills et lang.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `table` - Nom de la table cible (littéral interne, voir `assert_owned_by_user`).
/// * `id` - Identifiant de la ligne à supprimer.
/// * `not_found_message` - Message d'erreur renvoyé si la ligne n'existe pas.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi, `AppError::NotFound` si la ligne n'existe pas.
pub async fn delete_owned_row(
    pool: &SqlitePool,
    table: &str,
    id: i64,
    not_found_message: &str,
) -> Result<(), AppError> {
    assert_owned_by_user(pool, table, id, not_found_message).await?;

    let sql = format!("DELETE FROM {table} WHERE id = ?");
    sqlx::query(&sql).bind(id).execute(pool).await?;

    Ok(())
}
