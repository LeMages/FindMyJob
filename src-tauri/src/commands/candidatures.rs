//! Commandes Tauri pour les candidatures.

// Importation des modules internes
use crate::db::init::DbState;
use crate::db::{candidatures, users};
use crate::documents::save_generated_document;
use crate::error::AppError;
use crate::llm::generate_documents;
use crate::models::{CandidatureDetail, CandidatureView, CreateCandidatureInput, UpdateCandidatureInput};

/// Crée une candidature en générant d'abord, via le pipeline IA, un CV et
/// une lettre de motivation adaptés à l'offre (à partir des documents de
/// référence de la session et du profil candidat).
///
/// La génération a lieu avant l'insertion en base : si le pipeline échoue
/// (documents de référence manquants, API indisponible, compilation Typst
/// impossible même après correction), la candidature n'est pas créée et
/// l'erreur remonte telle quelle au frontend.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri (résolution des chemins de documents).
/// * `state` - État partagé (pool de connexions).
/// * `input` - Données de la candidature à créer.
///
/// # Returns
///
/// `CandidatureView` créée, avec les chemins des documents générés.
#[tauri::command]
pub async fn create_candidature(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
    input: CreateCandidatureInput,
) -> Result<CandidatureView, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation des prérequis à la génération — une session (donc des
    //    documents de référence) et une offre sont indispensables.
    // ------------------------------------------------------------------
    let description_offre = input
        .description_offre
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("La description de l'offre est requise.".to_string()))?;
    let session_id = input.session_id.ok_or_else(|| {
        AppError::Validation("Une session est requise pour générer les documents.".to_string())
    })?;

    // ------------------------------------------------------------------
    // 2. Génération du CV et de la lettre de motivation — avant toute
    //    persistance : en cas d'échec, rien n'est créé.
    // ------------------------------------------------------------------
    let generated = generate_documents(&state.pool, session_id, description_offre).await?;

    // ------------------------------------------------------------------
    // 3. Insertion de la candidature, puis sauvegarde des documents générés
    //    sous leur emplacement définitif (artefact/<candidature_id>.*)
    // ------------------------------------------------------------------
    let user = users::fetch_first_user(&state.pool).await?;
    let mut view = candidatures::insert(&state.pool, user.id, &input).await?;

    let cv_path = save_generated_document(
        &app,
        session_id,
        "cv",
        view.id,
        &generated.cv_typst,
        &generated.cv_pdf,
    )?;
    let lettre_motivation_path = save_generated_document(
        &app,
        session_id,
        "lettre_motivation",
        view.id,
        &generated.lm_typst,
        &generated.lm_pdf,
    )?;

    candidatures::set_documents(
        &state.pool,
        view.id,
        &Some(cv_path.clone()),
        &Some(lettre_motivation_path.clone()),
    )
    .await?;

    view.cv_path = Some(cv_path);
    view.lettre_motivation_path = Some(lettre_motivation_path);

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

/// Marque une candidature comme abandonnée (statut `abandon`) — utilisé par
/// le bouton « Supprimer » de la page de détail, tant que la candidature est
/// encore au statut « à postuler ».
///
/// # Arguments
///
/// * `state` - État partagé (pool de connexions).
/// * `id` - Identifiant de la candidature à abandonner.
///
/// # Returns
///
/// `Ok(())` une fois le statut mis à jour.
#[tauri::command]
pub async fn abandon_candidature(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    let user = users::fetch_first_user(&state.pool).await?;
    candidatures::abandon(&state.pool, user.id, id).await
}
