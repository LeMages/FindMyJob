//! Endpoints REST pour les candidatures : CRUD, cycle de vie (abandon,
//! postuler) et déclenchement du pipeline de génération de documents (CV +
//! lettre de motivation) à la création, quand les documents de référence de
//! la session sont disponibles.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::progress::ProgressStore;
use crate::{db, documents, pipeline};
use findmyjob_core::error::AppError;
use findmyjob_core::models::{
    CandidatureDetail, CandidatureView, CreateCandidatureInput, GenerationProgress, UpdateCandidatureInput,
};

/// Assemble les routes `/api/candidatures*` et `/api/sessions/{id}/candidatures`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}/candidatures", get(list_for_session))
        .route("/candidatures", post(create_candidature))
        .route("/candidatures/{id}", get(get_candidature).put(update_candidature))
        .route("/candidatures/{id}/progress", get(get_generation_progress))
        .route("/candidatures/{id}/abandon", post(abandon_candidature))
        .route("/candidatures/{id}/postuler", post(postuler_sans_message))
        .route("/candidatures/{id}/postuler/mail", post(postuler_par_mail))
        .route("/candidatures/{id}/postuler/message", post(postuler_par_message))
}

async fn list_for_session(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<Vec<CandidatureView>>, ApiError> {
    Ok(Json(db::candidatures::list_for_session(&state.pool, session_id).await?))
}

/// Indique si les deux documents de référence (CV et lettre de motivation)
/// d'une session ont déjà été analysés (`style_analysis` renseigné) — seule
/// condition sous laquelle une candidature peut être créée pour cette
/// session (voir `create_candidature`) et sous laquelle
/// `pipeline::generate_documents` peut réussir (voir `pipeline::require_reference`).
async fn references_ready(pool: &sqlx::PgPool, session_id: i64) -> Result<bool, ApiError> {
    Ok(db::documents::status_for_session(pool, session_id).await?.all_ready)
}

/// Crée une candidature puis, si la session cible dispose déjà de documents
/// de référence analysés, lance en tâche de fond la génération du CV et de
/// la lettre de motivation adaptés à l'offre — la réponse HTTP ne les attend
/// pas, `cv_path`/`lettre_motivation_path` restent `None` à ce stade. La
/// progression est consultable via `GET /candidatures/{id}/progress` pendant
/// que la génération tourne (voir `crate::progress`). Si la génération
/// échoue, la candidature créée ici est supprimée (voir `fail_generation`) :
/// une candidature ne doit jamais rester en base sans documents. Si la
/// candidature cible une session dont les documents de référence ne sont pas
/// prêts, la création est refusée (`AppError::Validation`) plutôt que de
/// créer une candidature sans document.
async fn create_candidature(
    State(state): State<AppState>,
    Json(input): Json<CreateCandidatureInput>,
) -> Result<Json<CandidatureView>, ApiError> {
    if let Some(session_id) = input.session_id {
        if !references_ready(&state.pool, session_id).await? {
            return Err(AppError::Validation(
                "Les documents de référence (CV et lettre de motivation) doivent être uploadés \
                 et analysés avant de pouvoir postuler."
                    .to_string(),
            )
            .into());
        }
    }

    let description_offre = input.description_offre.clone();
    let view = db::candidatures::insert(&state.pool, &input).await?;

    if let Some(session_id) = view.session_id {
        if references_ready(&state.pool, session_id).await? {
            spawn_document_generation(state, view.id, session_id, description_offre.unwrap_or_default());
        }
    }

    Ok(Json(view))
}

/// Lance en tâche de fond (`tokio::spawn`) la génération des documents d'une
/// candidature : reporte la progression dans `state.progress` à chaque
/// étape, sauvegarde les fichiers générés puis persiste leurs chemins en
/// base une fois terminé. Si une étape échoue, la candidature — créée sans
/// documents en attendant ce résultat — est supprimée plutôt que laissée
/// orpheline en base ; seul le store de progression (éphémère) garde une
/// trace de l'échec pour le desktop. Cette tâche n'a pas d'appelant HTTP à
/// qui propager une erreur.
fn spawn_document_generation(state: AppState, candidature_id: i64, session_id: i64, description_offre: String) {
    tokio::spawn(async move {
        let progress = state.progress.clone();
        let result = pipeline::generate_documents(
            &state.pool,
            &state.fonts_dir,
            session_id,
            &description_offre,
            |step, label| progress.update(candidature_id, step, pipeline::GENERATION_STEPS_TOTAL, label),
        )
        .await;

        let generated = match result {
            Ok(generated) => generated,
            Err(e) => return fail_generation(&state, &progress, candidature_id, &e.to_string()).await,
        };

        let saved = documents::save_generated_document(
            &state.data_dir,
            session_id,
            "cv",
            candidature_id,
            &generated.cv_typst,
            &generated.cv_pdf,
        )
        .and_then(|cv_path| {
            documents::save_generated_document(
                &state.data_dir,
                session_id,
                "lettre_motivation",
                candidature_id,
                &generated.lm_typst,
                &generated.lm_pdf,
            )
            .map(|lm_path| (cv_path, lm_path))
        });

        let (cv_path, lm_path) = match saved {
            Ok(paths) => paths,
            Err(e) => return fail_generation(&state, &progress, candidature_id, &e.to_string()).await,
        };

        if let Err(e) =
            db::candidatures::set_documents(
                &state.pool,
                candidature_id,
                &Some(cv_path),
                &Some(lm_path),
                (generated.cv_reference_version_id, generated.lm_reference_version_id),
            )
            .await
        {
            return fail_generation(&state, &progress, candidature_id, &e.to_string()).await;
        }

        progress.mark_completed(candidature_id);
    });
}

/// Marque la génération comme échouée dans le store de progression, puis
/// supprime la candidature créée pour cette génération — elle n'a jamais eu
/// de documents ni pu recevoir d'interaction utilisateur (contact, entretien)
/// pendant la modale bloquante de progression côté desktop, donc rien d'autre
/// à nettoyer. Le message affiché au desktop reste celui de l'échec de
/// génération même si la suppression elle-même échoue (loggée sur stderr).
async fn fail_generation(state: &AppState, progress: &ProgressStore, candidature_id: i64, error: &str) {
    progress.mark_failed(candidature_id, error);
    if let Err(e) = db::candidatures::delete(&state.pool, candidature_id).await {
        eprintln!("Suppression de la candidature {candidature_id} après échec de génération impossible : {e}");
    }
}

/// Renvoie la progression courante de la génération des documents d'une
/// candidature, ou `404` si aucune génération n'est en cours ni récemment
/// terminée pour cet identifiant (voir `crate::progress::ProgressStore`).
async fn get_generation_progress(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<GenerationProgress>, ApiError> {
    state
        .progress
        .get(id)
        .map(Json)
        .ok_or_else(|| AppError::NotFound("Aucune génération en cours pour cette candidature.".to_string()).into())
}

async fn get_candidature(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CandidatureDetail>, ApiError> {
    Ok(Json(db::candidatures::fetch_detail(&state.pool, id).await?))
}

async fn update_candidature(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateCandidatureInput>,
) -> Result<Json<CandidatureDetail>, ApiError> {
    Ok(Json(db::candidatures::update(&state.pool, id, &input).await?))
}

async fn abandon_candidature(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::candidatures::abandon(&state.pool, id).await?;
    Ok(())
}

async fn postuler_sans_message(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), ApiError> {
    db::candidatures::mark_postule(&state.pool, id, None).await?;
    Ok(())
}

/// Fait rédiger le mail de candidature par l'IA, marque la candidature comme
/// postulée et renvoie le mail, que le desktop exporte en `.txt`.
async fn postuler_par_mail(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<String>, ApiError> {
    let detail = db::candidatures::fetch_detail(&state.pool, id).await?;
    let profile = db::profile::build_profile(&state.pool).await?;
    let model =
        db::settings::resolve_llm_model(&state.pool, findmyjob_core::llm::client::DEFAULT_MODEL).await?;

    let email = findmyjob_core::llm::generate_application_email(&profile, &detail, &model).await?;
    db::candidatures::mark_postule(&state.pool, id, Some(&email)).await?;
    Ok(Json(email))
}

/// Fait rédiger le message court de candidature par l'IA, marque la
/// candidature comme postulée et renvoie le message, que le desktop exporte en `.txt`.
async fn postuler_par_message(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<String>, ApiError> {
    let detail = db::candidatures::fetch_detail(&state.pool, id).await?;
    let profile = db::profile::build_profile(&state.pool).await?;
    let model =
        db::settings::resolve_llm_model(&state.pool, findmyjob_core::llm::client::DEFAULT_MODEL).await?;

    let message = findmyjob_core::llm::generate_application_message(&profile, &detail, &model).await?;
    db::candidatures::mark_postule(&state.pool, id, Some(&message)).await?;
    Ok(Json(message))
}
