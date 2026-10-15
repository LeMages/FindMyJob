//! Commandes Tauri pour les expériences professionnelles du profil.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{CreateExperienceInput, ExperienceView};

/// Ajoute une expérience au profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données à créer.
///
/// # Returns
///
/// `ExperienceView` fraîchement créé(e).
#[tauri::command]
pub async fn add_experience(ctx: tauri::State<'_, AppContext>, input: CreateExperienceInput) -> Result<ExperienceView, AppError> {
    db::experiences::insert(&ctx.state()?.pool, input, &clock::now_utc()).await
}

/// Met à jour une expérience existant(e).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de l'entrée à modifier.
/// * `input` - Nouvelles données.
///
/// # Returns
///
/// `ExperienceView` mis(e) à jour.
#[tauri::command]
pub async fn update_experience(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    db::experiences::update(&ctx.state()?.pool, id, input).await
}

/// Supprime une expérience du profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de l'entrée à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimé(e).
#[tauri::command]
pub async fn delete_experience(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    db::experiences::delete(&ctx.state()?.pool, id).await
}
