//! Commandes Tauri pour les contacts recruteur (lecture seule : aucun
//! contact n'est créé par l'application à ce jour).

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::error::AppError;
use crate::models::ContactView;

/// Liste les contacts recruteur d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
///
/// # Returns
///
/// Contacts, du plus ancien au plus récent.
#[tauri::command]
pub async fn get_contacts(ctx: tauri::State<'_, AppContext>, candidature_id: i64) -> Result<Vec<ContactView>, AppError> {
    db::contacts::list_for_candidature(&ctx.state()?.pool, candidature_id).await
}
