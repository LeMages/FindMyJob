//! Commandes Tauri pour les entretiens rattachés à une candidature.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{interviews, users};
use crate::error::AppError;
use crate::models::{InterviewView, PlanningInterviewView};

/// Liste les entretiens encore en attente d'une candidature — utilisé pour
/// proposer à l'utilisateur lequel marquer comme passé.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entretiens au statut `en_attente`, triés par date d'entretien croissante.
#[tauri::command]
pub async fn get_pending_interviews(
    state: tauri::State<'_, DbState>,
    candidature_id: i64,
) -> Result<Vec<InterviewView>, AppError> {
    interviews::list_pending_for_candidature(&state.pool, candidature_id).await
}

/// Liste tous les entretiens planifiés d'une session, enrichis de leur
/// candidature — utilisé par la vue planning.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Entretiens dont la date est renseignée, triés par date croissante.
#[tauri::command]
pub async fn get_all_interviews_for_session(
    state: tauri::State<'_, DbState>,
    session_id: i64,
) -> Result<Vec<PlanningInterviewView>, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    interviews::list_for_session(&state.pool, user.id, session_id).await
}
