//! Commandes Tauri pour la timeline de suivi d'une candidature (événements
//! disponibles pour son statut courant, ajout d'un nouvel événement).

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{AvailableEvent, NewTimelineEvent};

/// Liste les événements proposables pour le statut courant d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
///
/// # Returns
///
/// Événements disponibles, dans l'ordre d'affichage.
#[tauri::command]
pub async fn get_available_events(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
) -> Result<Vec<AvailableEvent>, AppError> {
    db::timeline_events::available_for_candidature(&ctx.state()?.pool, candidature_id).await
}

/// Ajoute un événement à la timeline d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
/// * `event` - Événement saisi.
///
/// # Returns
///
/// `Ok(())` une fois l'événement enregistré.
#[tauri::command]
pub async fn add_timeline_event(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
    event: NewTimelineEvent,
) -> Result<(), AppError> {
    db::timeline_events::add_event(&ctx.state()?.pool, candidature_id, event, &clock::now_utc()).await
}
