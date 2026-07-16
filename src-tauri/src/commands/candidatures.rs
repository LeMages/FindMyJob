//! Commandes Tauri pour les candidatures.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{candidatures, users};
use crate::documents::finalize_candidature_document;
use crate::error::AppError;
use crate::models::{CandidatureDetail, CandidatureView, CreateCandidatureInput, UpdateCandidatureInput};

/// Crée une candidature et finalise les documents (CV / lettre de motivation)
/// préalablement mis en scène par `upload_candidature_cv` / `select_candidature_cv`
/// (et leurs équivalents lettre de motivation).
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri (résolution des chemins de documents).
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données de la candidature à créer.
///
/// # Returns
///
/// `CandidatureView` créée, avec les chemins de documents finalisés.
#[tauri::command]
pub async fn create_candidature(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
    input: CreateCandidatureInput,
) -> Result<CandidatureView, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;

    let mut view = candidatures::insert(&state.pool, user.id, &input).await?;

    // Les documents joints appartiennent à la session : impossible de les
    // finaliser sans identifiant de session.
    let session_id = if input.cv_path.is_some() || input.lettre_motivation_path.is_some() {
        Some(input.session_id.ok_or_else(|| {
            AppError::Validation("Une session est requise pour joindre un document.".to_string())
        })?)
    } else {
        None
    };

    let cv_path = match session_id {
        Some(sid) => finalize_candidature_document(&app, sid, "cv", input.cv_path, view.id)?,
        None => None,
    };
    let lettre_motivation_path = match session_id {
        Some(sid) => finalize_candidature_document(
            &app,
            sid,
            "lettre_motivation",
            input.lettre_motivation_path,
            view.id,
        )?,
        None => None,
    };

    if cv_path.is_some() || lettre_motivation_path.is_some() {
        candidatures::set_documents(&state.pool, view.id, &cv_path, &lettre_motivation_path).await?;
    }

    view.cv_path = cv_path;
    view.lettre_motivation_path = lettre_motivation_path;

    Ok(view)
}

/// Liste les candidatures d'une session.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Candidatures de la session, les plus récentes en premier.
#[tauri::command]
pub async fn get_candidatures(
    state: tauri::State<'_, DbState>,
    session_id: i64,
) -> Result<Vec<CandidatureView>, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    candidatures::list_for_session(&state.pool, user.id, session_id).await
}

/// Récupère le détail complet d'une candidature, pour la page de détail.
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `CandidatureDetail` correspondante, ou `AppError::NotFound` si elle n'existe pas.
#[tauri::command]
pub async fn get_candidature(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<CandidatureDetail, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    candidatures::fetch_detail(&state.pool, user.id, id).await
}

/// Met à jour les champs éditables d'une candidature (détails du poste et
/// notes), refusé si son statut n'est plus « à postuler ».
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la candidature à modifier.
/// * `input` - Nouvelles valeurs des champs éditables.
///
/// # Returns
///
/// `CandidatureDetail` mise à jour.
#[tauri::command]
pub async fn update_candidature(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: UpdateCandidatureInput,
) -> Result<CandidatureDetail, AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    candidatures::update(&state.pool, user.id, id, &input).await
}
