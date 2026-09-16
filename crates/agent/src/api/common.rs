//! Aide partagée par les handlers `api::*` : résolution de l'utilisateur
//! unique de l'application (API mono-utilisateur, comme le reste du
//! backend — voir `db::common::assert_owned_by_user`).

// Importation des crates externes
use sqlx::PgPool;

// Importation des modules internes
use crate::api::error::ApiError;
use crate::db;

/// Résout l'identifiant de l'utilisateur unique de l'application.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// Identifiant de l'utilisateur, ou `ApiError` (404) si le profil n'a pas
/// encore été créé.
pub(crate) async fn current_user_id(pool: &PgPool) -> Result<i64, ApiError> {
    Ok(db::users::fetch_first_user(pool).await?.id)
}
