//! Commandes Tauri pour les formations du profil.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{CreateFormationInput, FormationView};

/// Ajoute une formation au profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données à créer.
///
/// # Returns
///
/// `FormationView` fraîchement créé(e).
#[tauri::command]
pub async fn add_formation(ctx: tauri::State<'_, AppContext>, input: CreateFormationInput) -> Result<FormationView, AppError> {
    db::formations::insert(&ctx.state()?.pool, input, &clock::now_utc()).await
}

/// Met à jour une formation existant(e).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de l'entrée à modifier.
/// * `input` - Nouvelles données.
///
/// # Returns
///
/// `FormationView` mis(e) à jour.
#[tauri::command]
pub async fn update_formation(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    db::formations::update(&ctx.state()?.pool, id, input).await
}

/// Supprime une formation du profil.
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
pub async fn delete_formation(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    db::formations::delete(&ctx.state()?.pool, id).await
}
