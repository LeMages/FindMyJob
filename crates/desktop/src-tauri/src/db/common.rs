//! Fonctions partagées par les modules `db::*` : conversion des erreurs
//! sqlx, normalisation des saisies et suppression d'une ligne par identifiant.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Convertit une erreur sqlx en `AppError::Database`.
///
/// Nécessaire ici : `AppError` (défini dans `findmyjob-core`) et
/// `sqlx::Error` sont tous les deux étrangers à ce crate, la règle de
/// cohérence de Rust interdit d'implémenter `From<sqlx::Error> for AppError`.
/// Chaque appel sqlx passe donc explicitement par cette fonction.
///
/// # Arguments
///
/// * `e` - Erreur sqlx à convertir.
///
/// # Returns
///
/// `AppError::Database` portant le message de l'erreur d'origine.
pub fn map_db_err(e: sqlx::Error) -> AppError {
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

/// Supprime une ligne par identifiant (tables enfants du profil).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `table` - Nom de la table cible. Doit être un littéral interne au code,
///   jamais une valeur fournie par l'utilisateur (interpolé dans la requête).
/// * `id` - Identifiant de la ligne à supprimer.
/// * `not_found_message` - Message d'erreur renvoyé si la ligne n'existe pas.
///
/// # Returns
///
/// `Ok(())` si la ligne a été supprimée, `AppError::NotFound` sinon.
pub async fn delete_by_id(
    pool: &SqlitePool,
    table: &'static str,
    id: i64,
    not_found_message: &str,
) -> Result<(), AppError> {
    let sql = format!("DELETE FROM {table} WHERE id = $1");
    let result = sqlx::query(&sql).bind(id).execute(pool).await.map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(not_found_message.to_string()));
    }
    Ok(())
}
