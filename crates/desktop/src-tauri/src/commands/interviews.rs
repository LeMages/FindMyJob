//! Commandes Tauri pour les entretiens rattachés à une candidature.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{InterviewView, PlanningInterviewView};

/// Liste les entretiens encore en attente d'une candidature — utilisé pour
/// proposer à l'utilisateur lequel marquer comme passé.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entretiens au statut `en_attente`, triés par date d'entretien croissante.
#[tauri::command]
pub async fn get_pending_interviews(
    api: tauri::State<'_, ApiClient>,
    candidature_id: i64,
) -> Result<Vec<InterviewView>, AppError> {
    api.get(&format!("/api/candidatures/{candidature_id}/interviews/pending"))
        .await
}

/// Liste tous les entretiens planifiés d'une session, enrichis de leur
/// candidature — utilisé par la vue planning.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Entretiens dont la date est renseignée, triés par date croissante.
#[tauri::command]
pub async fn get_all_interviews_for_session(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<Vec<PlanningInterviewView>, AppError> {
    api.get(&format!("/api/sessions/{session_id}/interviews")).await
}
