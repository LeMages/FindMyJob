//! Commandes Tauri pour les sessions de candidature.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{
    CreateSessionInput, SessionView, UpdateSessionInput, UpdateSessionListFormatInput, UpdateSessionLocationInput,
};
use crate::services;

/// Crée une session de candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Nom et localisation facultative de la session.
///
/// # Returns
///
/// `SessionView` fraîchement créée.
#[tauri::command]
pub async fn create_session(
    ctx: tauri::State<'_, AppContext>,
    input: CreateSessionInput,
) -> Result<SessionView, AppError> {
    db::sessions::insert(&ctx.state()?.pool, &input, &clock::now_utc()).await
}

/// Liste les sessions avec leurs statistiques.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// Sessions, de la plus récente à la plus ancienne.
#[tauri::command]
pub async fn get_sessions(ctx: tauri::State<'_, AppContext>) -> Result<Vec<SessionView>, AppError> {
    db::sessions::list_all(&ctx.state()?.pool).await
}

/// Renomme une session ou change son statut.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la session.
/// * `input` - Nouveau nom et nouveau statut.
///
/// # Returns
///
/// `SessionView` mise à jour.
#[tauri::command]
pub async fn update_session(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: UpdateSessionInput,
) -> Result<SessionView, AppError> {
    db::sessions::update(&ctx.state()?.pool, id, &input).await
}

/// Modifie la localisation de l'expéditeur d'une session (ni nom ni statut).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la session.
/// * `input` - Ville et code postal.
///
/// # Returns
///
/// `SessionView` mise à jour.
#[tauri::command]
pub async fn update_session_location(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: UpdateSessionLocationInput,
) -> Result<SessionView, AppError> {
    db::sessions::update_location(&ctx.state()?.pool, id, &input).await
}

/// Modifie le format des listes des CV générés dans une session.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la session.
/// * `input` - Nouveau format (`puce` ou `tiret`).
///
/// # Returns
///
/// `SessionView` mise à jour.
#[tauri::command]
pub async fn update_session_list_format(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: UpdateSessionListFormatInput,
) -> Result<SessionView, AppError> {
    db::sessions::update_list_format(&ctx.state()?.pool, id, &input).await
}

/// Supprime une session, ses données et son dossier de documents.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la session.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_session(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    services::sessions::delete(ctx.state()?, id).await
}
