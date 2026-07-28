//! Commandes Tauri pour les entretiens rattachés à une candidature.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::interviews;
use crate::error::AppError;
use crate::models::InterviewView;

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
