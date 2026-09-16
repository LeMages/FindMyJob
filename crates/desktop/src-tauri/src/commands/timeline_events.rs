//! Commandes Tauri pour la timeline de suivi d'une candidature (événements
//! disponibles pour son statut courant, ajout d'un nouvel événement).

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{AvailableEvent, NewTimelineEvent};

/// Liste les événements que l'utilisateur peut ajouter à la timeline d'une
/// candidature, compte tenu de son statut courant.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Événements disponibles pour le statut courant de la candidature.
#[tauri::command]
pub async fn get_available_events(
    api: tauri::State<'_, ApiClient>,
    candidature_id: i64,
) -> Result<Vec<AvailableEvent>, AppError> {
    api.get(&format!("/api/candidatures/{candidature_id}/events")).await
}

/// Ajoute un événement à la timeline d'une candidature.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `candidature_id` - Identifiant de la candidature.
/// * `event` - Événement soumis par le frontend.
///
/// # Returns
///
/// `Ok(())` une fois l'événement enregistré.
#[tauri::command]
pub async fn add_timeline_event(
    api: tauri::State<'_, ApiClient>,
    candidature_id: i64,
    event: NewTimelineEvent,
) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/candidatures/{candidature_id}/events"), &event)
        .await
}
