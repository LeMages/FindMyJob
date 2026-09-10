//! Commandes Tauri pour les contacts recruteur.

// Importation des modules internes
use crate::db::contacts;
use crate::db::init::DbState;
use crate::error::AppError;
use crate::models::ContactView;

/// Liste les contacts recruteur d'une candidature.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Contacts triés par date de création croissante.
#[tauri::command]
pub async fn get_contacts(
    state: tauri::State<'_, DbState>,
    candidature_id: i64,
) -> Result<Vec<ContactView>, AppError> {
    contacts::list_for_candidature(&state.pool, candidature_id).await
}
