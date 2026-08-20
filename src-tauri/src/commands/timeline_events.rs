//! Commandes Tauri pour la timeline de suivi d'une candidature (événements
//! disponibles pour son statut courant, ajout d'un nouvel événement).

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{candidatures, interviews, timeline_events, users};
use crate::error::AppError;
use crate::models::{AvailableEvent, NewTimelineEvent};

/// Liste les événements que l'utilisateur peut ajouter à la timeline d'une
/// candidature, compte tenu de son statut courant.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Événements disponibles (voir `db::timeline_events::available_events`).
#[tauri::command]
pub async fn get_available_events(
    state: tauri::State<'_, DbState>,
    candidature_id: i64,
) -> Result<Vec<AvailableEvent>, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    let status = candidatures::fetch_status(&state.pool, user.id, candidature_id).await?;
    let has_pending_interviews = interviews::count_pending(&state.pool, candidature_id).await? > 0;
    Ok(timeline_events::available_events(&status, has_pending_interviews))
}

/// Ajoute un événement à la timeline d'une candidature.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `candidature_id` - Identifiant de la candidature.
/// * `event` - Événement soumis par le frontend.
///
/// # Returns
///
/// `Ok(())` une fois l'événement enregistré.
#[tauri::command]
pub async fn add_timeline_event(
    state: tauri::State<'_, DbState>,
    candidature_id: i64,
    event: NewTimelineEvent,
) -> Result<(), AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    timeline_events::add_event(&state.pool, user.id, candidature_id, event).await
}
