//! Assemblage du profil complet de l'utilisateur (informations, expériences,
//! formations, projets, compétences, langues) — jointure logique entre les
//! tables du profil, partagée par la commande `get_profile` et le pipeline
//! de génération de documents (`llm::pipeline`).

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
/// entrées de profil de l'utilisateur.
pub async fn build_profile(pool: &SqlitePool) -> Result<ProfileDto, AppError> {
    let user = users::fetch_first_user(pool).await?;
    let uid = user.id;

    let profile_experiences = experiences::list_for_user(pool, uid).await?;
    let profile_formations = formations::list_for_user(pool, uid).await?;
    let profile_projets = projets::list_for_user(pool, uid).await?;
    let profile_skills = skills::list_for_user(pool, uid).await?;
    let profile_langues = langues::list_for_user(pool, uid).await?;

    Ok(ProfileDto {
        user,
        experiences: profile_experiences,
        formations: profile_formations,
        projets: profile_projets,
        skills: profile_skills,
        langues: profile_langues,
    })
}
