//! Commandes Tauri pour les langues du profil.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{CreateLangInput, LangView};

/// Ajoute une langue au profil.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données à créer.
///
/// # Returns
///
/// `LangView` fraîchement créé(e).
#[tauri::command]
pub async fn add_lang(ctx: tauri::State<'_, AppContext>, input: CreateLangInput) -> Result<LangView, AppError> {
    db::langues::insert(&ctx.state()?.pool, input, &clock::now_utc()).await
}

/// Met à jour une langue existant(e).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de l'entrée à modifier.
/// * `input` - Nouvelles données.
///
/// # Returns
///
/// `LangView` mis(e) à jour.
#[tauri::command]
pub async fn update_lang(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    db::langues::update(&ctx.state()?.pool, id, input).await
}

/// Supprime une langue du profil.
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
pub async fn delete_lang(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    db::langues::delete(&ctx.state()?.pool, id).await
}
