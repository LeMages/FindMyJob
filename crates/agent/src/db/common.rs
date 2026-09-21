//! Helpers partagés par les modules `db::*` : normalisation des chaînes
//! utilisateur, suppression générique d'une ligne appartenant à
//! l'utilisateur unique de l'application, et conversion des erreurs sqlx.

// Importation des crates externes
use sqlx::PgPool;

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Convertit une erreur sqlx en `AppError::Database`.
///
/// Nécessaire ici : `AppError` (défini dans `findmyjob-core`) et
/// `sqlx::Error` sont tous les deux étrangers à ce crate, la règle de
/// cohérence de Rust interdit d'implémenter `From<sqlx::Error> for AppError`
/// depuis `findmyjob-agent`. Chaque appel sqlx dans `db::*` doit donc passer
/// explicitement par cette fonction plutôt que par une conversion implicite.
///
/// # Arguments
///
/// * `e` - Erreur sqlx à convertir.
///
/// # Returns
///
/// `AppError::Database` portant le message de l'erreur d'origine.
pub(crate) fn map_db_err(e: sqlx::Error) -> AppError {
    AppError::Database(e.to_string())
}

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
/// * `pool` - Pool de connexions PostgreSQL.
/// * `table` - Nom de la table cible. Doit être un littéral interne au code,
///   jamais une valeur fournie par l'utilisateur (construit par `format!` dans la requête).
/// * `id` - Identifiant de la ligne à vérifier.
/// * `not_found_message` - Message d'erreur renvoyé si la ligne n'existe pas.
///
/// # Returns
///
/// `Ok(())` si la ligne appartient à l'utilisateur, `AppError::NotFound` sinon.
pub async fn assert_owned_by_user(
    pool: &PgPool,
    table: &str,
    id: i64,
    not_found_message: &str,
) -> Result<(), AppError> {
    let sql = format!(
        "SELECT COUNT(*) FROM {table} WHERE id = $1 AND user_id = (SELECT id FROM profile.users LIMIT 1)"
    );
    let n: i64 = sqlx::query_scalar::<_, i64>(&sql)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(map_db_err)?;

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
/// * `pool` - Pool de connexions PostgreSQL.
/// * `table` - Nom de la table cible (littéral interne, voir `assert_owned_by_user`).
/// * `id` - Identifiant de la ligne à supprimer.
/// * `not_found_message` - Message d'erreur renvoyé si la ligne n'existe pas.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi, `AppError::NotFound` si la ligne n'existe pas.
pub async fn delete_owned_row(
    pool: &PgPool,
    table: &str,
    id: i64,
    not_found_message: &str,
) -> Result<(), AppError> {
    assert_owned_by_user(pool, table, id, not_found_message).await?;

    let sql = format!("DELETE FROM {table} WHERE id = $1");
    sqlx::query(&sql).bind(id).execute(pool).await.map_err(map_db_err)?;

    Ok(())
}
