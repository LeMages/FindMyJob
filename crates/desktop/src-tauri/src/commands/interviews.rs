//! Commandes Tauri pour les entretiens rattachés à une candidature.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::error::AppError;
use crate::models::{InterviewView, PlanningInterviewView};

/// Liste les entretiens en attente d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
///
/// # Returns
///
/// Entretiens `en_attente`, par date croissante.
#[tauri::command]
pub async fn get_pending_interviews(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
) -> Result<Vec<InterviewView>, AppError> {
    db::interviews::list_pending_for_candidature(&ctx.state()?.pool, candidature_id).await
}

/// Liste les entretiens datés d'une session (onglet Planning).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session affichée.
///
/// # Returns
///
/// Entretiens par date croissante.
#[tauri::command]
pub async fn get_all_interviews_for_session(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
) -> Result<Vec<PlanningInterviewView>, AppError> {
    db::interviews::list_for_session(&ctx.state()?.pool, session_id).await
}
