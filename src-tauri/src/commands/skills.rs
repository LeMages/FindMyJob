//! Commandes Tauri pour les compétences du profil.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{CreateSkillInput, SkillView};

/// Ajoute une compétence au profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données à créer.
///
/// # Returns
///
/// `SkillView` fraîchement créé(e).
#[tauri::command]
pub async fn add_skill(ctx: tauri::State<'_, AppContext>, input: CreateSkillInput) -> Result<SkillView, AppError> {
    db::skills::insert(&ctx.state()?.pool, input, &clock::now_utc()).await
}

/// Met à jour une compétence existant(e).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de l'entrée à modifier.
/// * `input` - Nouvelles données.
///
/// # Returns
///
/// `SkillView` mis(e) à jour.
#[tauri::command]
pub async fn update_skill(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    db::skills::update(&ctx.state()?.pool, id, input).await
}

/// Supprime une compétence du profil.
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
pub async fn delete_skill(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    db::skills::delete(&ctx.state()?.pool, id).await
}
