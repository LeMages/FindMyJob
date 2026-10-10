//! Commandes Tauri pour les candidatures.

// Importation des modules internes
use crate::app_state::AppContext;
use crate::db;
use crate::error::AppError;
use crate::models::{
    CandidatureDetail, CandidatureView, CreateCandidatureInput, DuplicateCandidatureView, GenerationProgress,
    UpdateCandidatureInput,
};
use crate::services;
use crate::services::candidature_validation::validate_update;
use crate::services::candidatures::ApplicationMessage;

/// Crée une candidature et lance la génération de ses documents en tâche de fond.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `input` - Données saisies.
///
/// # Returns
///
/// `CandidatureView` créée (documents en cours de génération).
#[tauri::command]
pub async fn create_candidature(
    ctx: tauri::State<'_, AppContext>,
    input: CreateCandidatureInput,
) -> Result<CandidatureView, AppError> {
    services::candidatures::create(ctx.state()?, &input).await
}

/// Cherche les candidatures existantes pour le même lien d'offre.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `url` - Lien saisi.
///
/// # Returns
///
/// Candidatures existantes, toutes sessions confondues.
#[tauri::command]
pub async fn find_candidatures_by_url(
    ctx: tauri::State<'_, AppContext>,
    url: String,
) -> Result<Vec<DuplicateCandidatureView>, AppError> {
    services::candidatures::find_by_offer_url(&ctx.state()?.pool, &url).await
}

/// Renvoie la progression de la génération des documents d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `candidature_id` - Candidature suivie.
///
/// # Returns
///
/// `GenerationProgress`, ou `AppError::NotFound` si aucune génération n'est
/// en cours ni récemment terminée.
#[tauri::command]
pub async fn get_generation_progress(
    ctx: tauri::State<'_, AppContext>,
    candidature_id: i64,
) -> Result<GenerationProgress, AppError> {
    ctx.state()?
        .progress
        .get(candidature_id)
        .ok_or_else(|| AppError::NotFound("Aucune génération en cours pour cette candidature.".to_string()))
}

/// Liste les candidatures d'une session.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `session_id` - Session affichée.
///
/// # Returns
///
/// Candidatures, de la plus récente à la plus ancienne.
#[tauri::command]
pub async fn get_candidatures(
    ctx: tauri::State<'_, AppContext>,
    session_id: i64,
) -> Result<Vec<CandidatureView>, AppError> {
    db::candidatures::list_for_session(&ctx.state()?.pool, session_id).await
}

/// Récupère le détail d'une candidature.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `CandidatureDetail`.
#[tauri::command]
pub async fn get_candidature(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<CandidatureDetail, AppError> {
    db::candidatures::fetch_detail(&ctx.state()?.pool, id).await
}

/// Modifie une candidature encore « à postuler ».
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la candidature.
/// * `input` - Nouvelles valeurs.
///
/// # Returns
///
/// `CandidatureDetail` mise à jour.
#[tauri::command]
pub async fn update_candidature(
    ctx: tauri::State<'_, AppContext>,
    id: i64,
    input: UpdateCandidatureInput,
) -> Result<CandidatureDetail, AppError> {
    let fields = validate_update(&input)?;
    db::candidatures::update(&ctx.state()?.pool, id, &fields).await
}

/// Supprime une candidature « à postuler » et ses documents.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_candidature(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    services::candidatures::delete(ctx.state()?, id).await
}

/// Marque une candidature comme postulée, sans message.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` une fois le statut changé.
#[tauri::command]
pub async fn postuler_sans_message(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    services::candidatures::postuler_sans_message(&ctx.state()?.pool, id).await
}

/// Fait rédiger le mail de candidature par l'IA et marque la candidature postulée.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// Le mail rédigé.
#[tauri::command]
pub async fn postuler_par_mail(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<String, AppError> {
    services::candidatures::postuler_avec_message(&ctx.state()?.pool, id, ApplicationMessage::Mail).await
}

/// Fait rédiger le message court de candidature par l'IA et marque la
/// candidature postulée.
///
/// # Arguments
///
/// * `ctx` - Contexte de l'application.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// Le message rédigé.
#[tauri::command]
pub async fn postuler_par_message(ctx: tauri::State<'_, AppContext>, id: i64) -> Result<String, AppError> {
    services::candidatures::postuler_avec_message(&ctx.state()?.pool, id, ApplicationMessage::Message).await
}
