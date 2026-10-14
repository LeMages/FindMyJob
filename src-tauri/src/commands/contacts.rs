//! Commandes Tauri du carnet de contacts recruteur : lecture du carnet et
//! des contacts d'une candidature, création, modification, suppression,
//! liaison et retrait d'une candidature.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::clock;
use crate::db;
use crate::error::AppError;
use crate::models::{ContactInput, ContactView};

/// Liste tout le carnet de contacts.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// Contacts triés par nom.
#[tauri::command]
pub async fn list_contacts(ctx: tauri::State<'_, AppContext>) -> Result<Vec<ContactView>, AppError> {
    db::contacts::list_all(&ctx.state()?.pool).await
}

/// Liste les contacts liés à une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
///
/// # Returns
///
/// Contacts, dans l'ordre de leur liaison.
#[tauri::command]
pub async fn get_contacts(ctx: tauri::State<'_, AppContext>, candidature_id: i64) -> Result<Vec<ContactView>, AppError> {
    db::contacts::list_for_candidature(&ctx.state()?.pool, candidature_id).await
}

/// Crée un contact dans le carnet, lié à une candidature si elle est donnée.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Saisie du contact.
/// * `candidature_id` - Candidature à laquelle lier le contact, si renseignée.
///
/// # Returns
///
/// Le contact créé.
#[tauri::command]
pub async fn create_contact(
    ctx: tauri::State<'_, AppContext>,
    input: ContactInput,
    candidature_id: Option<i64>,
) -> Result<ContactView, AppError> {
    db::contacts::create(&ctx.state()?.pool, &input, candidature_id, &clock::now_utc()).await
}

/// Modifie un contact du carnet.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Contact à modifier.
/// * `input` - Nouvelle saisie.
///
/// # Returns
///
/// Le contact modifié.
#[tauri::command]
pub async fn update_contact(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: ContactInput,
) -> Result<ContactView, AppError> {
    db::contacts::update(&ctx.state()?.pool, id, &input).await
}

/// Supprime définitivement un contact du carnet (et toutes ses liaisons).
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Contact à supprimer.
///
/// # Returns
///
/// `Ok(())` une fois le contact supprimé.
#[tauri::command]
pub async fn delete_contact(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    db::contacts::delete(&ctx.state()?.pool, id).await
}

/// Lie un contact existant du carnet à une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
/// * `contact_id` - Contact à lier.
///
/// # Returns
///
/// `Ok(())` une fois la liaison créée.
#[tauri::command]
pub async fn link_contact(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
    contact_id: i64,
) -> Result<(), AppError> {
    db::contacts::link(&ctx.state()?.pool, candidature_id, contact_id, &clock::now_utc()).await
}

/// Retire un contact d'une candidature et de ses entretiens ; le contact
/// reste dans le carnet.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature concernée.
/// * `contact_id` - Contact à retirer.
///
/// # Returns
///
/// `Ok(())` une fois la liaison supprimée.
#[tauri::command]
pub async fn unlink_contact(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
    contact_id: i64,
) -> Result<(), AppError> {
    db::contacts::unlink(&ctx.state()?.pool, candidature_id, contact_id).await
}
