//! Endpoints REST pour les candidatures : CRUD, cycle de vie (abandon,
//! postuler) et déclenchement du pipeline de génération de documents (CV +
//! lettre de motivation) à la création, quand les documents de référence de
//! la session sont disponibles.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::common::current_user_id;
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::{db, documents, pipeline};
use findmyjob_core::models::{
    CandidatureDetail, CandidatureView, CreateCandidatureInput, UpdateCandidatureInput,
};

/// Assemble les routes `/api/candidatures*` et `/api/sessions/{id}/candidatures`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}/candidatures", get(list_for_session))
        .route("/candidatures", post(create_candidature))
        .route("/candidatures/{id}", get(get_candidature).put(update_candidature))
        .route("/candidatures/{id}/abandon", post(abandon_candidature))
        .route("/candidatures/{id}/postuler", post(postuler_sans_message))
        .route("/candidatures/{id}/postuler/mail", post(postuler_par_mail))
        .route("/candidatures/{id}/postuler/message", post(postuler_par_message))
}

async fn list_for_session(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<Vec<CandidatureView>>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::candidatures::list_for_session(&state.pool, user_id, session_id).await?))
}

/// Indique si les deux documents de référence (CV et lettre de motivation)
/// d'une session ont déjà été analysés (`style_analysis` renseigné) — seule
/// condition sous laquelle `pipeline::generate_documents` peut réussir (voir
/// `pipeline::require_reference`).
async fn references_ready(pool: &sqlx::PgPool, session_id: i64) -> Result<bool, ApiError> {
    for doc_type in ["cv", "lettre_motivation"] {
        let ready = db::documents::get_by_session_and_type(pool, session_id, doc_type)
            .await?
            .is_some_and(|d| d.style_analysis.is_some());
        if !ready {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Crée une candidature puis, si la session cible dispose déjà de documents
/// de référence analysés, génère et attache immédiatement le CV et la lettre
/// de motivation adaptés à l'offre. Sans documents de référence, la
/// candidature est créée sans document attaché (comme `candidatures::insert`
/// seul) — la génération pourra être déclenchée plus tard par un autre appel.
async fn create_candidature(
    State(state): State<AppState>,
    Json(input): Json<CreateCandidatureInput>,
) -> Result<Json<CandidatureView>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    let description_offre = input.description_offre.clone();
    let mut view = db::candidatures::insert(&state.pool, user_id, &input).await?;

    if let Some(session_id) = view.session_id {
        if references_ready(&state.pool, session_id).await? {
            let description_offre = description_offre.unwrap_or_default();
            let generated =
                pipeline::generate_documents(&state.pool, &state.fonts_dir, session_id, &description_offre)
                    .await?;

            let cv_path = documents::save_generated_document(
                &state.data_dir,
                session_id,
                "cv",
                view.id,
                &generated.cv_typst,
                &generated.cv_pdf,
            )?;
            let lm_path = documents::save_generated_document(
                &state.data_dir,
                session_id,
                "lettre_motivation",
                view.id,
                &generated.lm_typst,
                &generated.lm_pdf,
            )?;

            db::candidatures::set_documents(&state.pool, view.id, &Some(cv_path.clone()), &Some(lm_path.clone()))
                .await?;
            view.cv_path = Some(cv_path);
            view.lettre_motivation_path = Some(lm_path);
        }
    }

    Ok(Json(view))
}

async fn get_candidature(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CandidatureDetail>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::candidatures::fetch_detail(&state.pool, user_id, id).await?))
}

async fn update_candidature(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateCandidatureInput>,
) -> Result<Json<CandidatureDetail>, ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    Ok(Json(db::candidatures::update(&state.pool, user_id, id, &input).await?))
}

async fn abandon_candidature(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    db::candidatures::abandon(&state.pool, user_id, id).await?;
    Ok(())
}

async fn postuler_sans_message(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    db::candidatures::mark_postule(&state.pool, user_id, id, None).await?;
    Ok(())
}

async fn postuler_par_mail(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    let detail = db::candidatures::fetch_detail(&state.pool, user_id, id).await?;
    let profile = db::profile::build_profile(&state.pool).await?;
    let model =
        db::settings::resolve_llm_model(&state.pool, findmyjob_core::llm::client::DEFAULT_MODEL).await?;

    let email = findmyjob_core::llm::generate_application_email(&profile, &detail, &model).await?;
    db::candidatures::mark_postule(&state.pool, user_id, id, Some(&email)).await?;
    Ok(())
}

async fn postuler_par_message(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    let user_id = current_user_id(&state.pool).await?;
    let detail = db::candidatures::fetch_detail(&state.pool, user_id, id).await?;
    let profile = db::profile::build_profile(&state.pool).await?;
    let model =
        db::settings::resolve_llm_model(&state.pool, findmyjob_core::llm::client::DEFAULT_MODEL).await?;

    let message = findmyjob_core::llm::generate_application_message(&profile, &detail, &model).await?;
    db::candidatures::mark_postule(&state.pool, user_id, id, Some(&message)).await?;
    Ok(())
}
