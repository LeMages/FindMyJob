//! Langue dans laquelle le profil est rédigé : validation puis écriture.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::error::AppError;
use crate::models::doc_lang::{is_supported, SUPPORTED_LANGS};
use crate::models::UserView;

/// Change la langue du profil.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `lang` - Code de langue choisi.
///
/// # Returns
///
/// `UserView` mis à jour, ou `AppError::Validation` pour une langue non
/// prise en charge.
pub async fn set_profile_language(pool: &SqlitePool, lang: &str) -> Result<UserView, AppError> {
    if !is_supported(lang) {
        return Err(AppError::Validation(format!(
            "Langue du profil « {lang} » non prise en charge (attendu : {}).",
            SUPPORTED_LANGS.join(" ou ")
        )));
    }
    db::users::update_profile_language(pool, lang).await
}
