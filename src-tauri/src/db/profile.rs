//! Assemblage du profil complet de l'utilisateur (informations, expériences,
//! formations, projets, compétences, langues) — jointure logique entre les
//! tables du profil, partagée par la commande `get_profile` et le pipeline
//! de génération de documents (`crate::pipeline`).

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db::{experiences, formations, langues, projets, skills, users};
use crate::error::AppError;
use crate::models::ProfileDto;

/// Agrège le profil complet de l'utilisateur unique de l'application.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// `ProfileDto` regroupant les informations personnelles et l'ensemble des
/// entrées du profil, ou `AppError::NotFound` si le profil n'existe pas.
pub async fn build_profile(pool: &SqlitePool) -> Result<ProfileDto, AppError> {
    Ok(ProfileDto {
        user: users::fetch_user(pool).await?,
        experiences: experiences::list_all(pool).await?,
        formations: formations::list_all(pool).await?,
        projets: projets::list_all(pool).await?,
        skills: skills::list_all(pool).await?,
        langues: langues::list_all(pool).await?,
    })
}
