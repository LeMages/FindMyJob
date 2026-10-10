//! Commandes Tauri pour les projets personnels du profil.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{CreateProjetInput, ProjetView};

/// Ajoute un projet au profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données à créer.
///
/// # Returns
///
/// `ProjetView` fraîchement créé(e).
#[tauri::command]
pub async fn add_projet(ctx: tauri::State<'_, AppContext>, input: CreateProjetInput) -> Result<ProjetView, AppError> {
    db::projets::insert(&ctx.state()?.pool, input, &clock::now_utc()).await
}

/// Met à jour un projet existant(e).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de l'entrée à modifier.
/// * `input` - Nouvelles données.
///
/// # Returns
///
/// `ProjetView` mis(e) à jour.
#[tauri::command]
pub async fn update_projet(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    db::projets::update(&ctx.state()?.pool, id, input).await
}

/// Supprime un projet du profil.
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
pub async fn delete_projet(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    db::projets::delete(&ctx.state()?.pool, id).await
}
