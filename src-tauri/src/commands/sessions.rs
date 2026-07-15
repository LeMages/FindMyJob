//! Commandes Tauri pour les sessions de candidature.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{sessions, users};
use crate::error::AppError;
use crate::models::{CreateSessionInput, SessionView, UpdateSessionInput};

/// Crée une nouvelle session de candidature.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `input` - Nom de la session à créer.
///
/// # Returns
///
/// `SessionView` fraîchement créée.
#[tauri::command]
pub async fn create_session(
    state: tauri::State<'_, DbState>,
    input: CreateSessionInput,
) -> Result<SessionView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    sessions::insert(&state.pool, user.id, &input.nom).await
}

/// Liste les sessions de l'utilisateur avec leurs statistiques.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
///
/// # Returns
///
/// Sessions triées par date de création décroissante.
#[tauri::command]
pub async fn get_sessions(state: tauri::State<'_, DbState>) -> Result<Vec<SessionView>, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    sessions::list_for_user(&state.pool, user.id).await
}

/// Met à jour le nom, le contexte, le statut et la zone cible d'une session existante.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la session à modifier.
/// * `input` - Nouvelles valeurs des champs éditables de la session.
///
/// # Returns
///
/// `SessionView` mise à jour.
#[tauri::command]
pub async fn update_session(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: UpdateSessionInput,
) -> Result<SessionView, AppError> {
    sessions::update(&state.pool, id, &input).await
}

/// Supprime une session et toutes les candidatures qui en dépendent.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la session à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_session(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    sessions::delete(&state.pool, id).await
}
