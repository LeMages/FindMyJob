//! Suppression d'une session avec ses fichiers.

// Importation des modules internes
use crate::app_state::AppState;
use crate::db;
use crate::documents;
use crate::error::AppError;
use crate::trace_log;

/// Supprime une session, toutes ses données en base (par cascade) puis son
/// dossier de documents (références, versions, documents générés, traces).
/// Le nettoyage disque est tracé en cas d'échec sans renvoyer d'erreur : la
/// base est déjà cohérente.
///
/// # Arguments
///
/// * `state` - État de l'application.
/// * `id` - Session à supprimer.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::NotFound` si la session n'existe pas.
pub async fn delete(state: &AppState, id: i64) -> Result<(), AppError> {
    db::sessions::delete(&state.pool, id).await?;
    if let Err(e) = documents::delete_session_documents(&state.data_dir, id) {
        trace_log::log(&format!("Suppression des documents de la session {id} impossible : {e}"));
    }
    Ok(())
}
