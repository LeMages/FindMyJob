//! Génération en tâche de fond des documents d'une candidature (CV + lettre
//! de motivation) : lancement dans le runtime de Tauri, suivi de
//! progression, sauvegarde des fichiers et des chemins, et suppression de la
//! candidature si la génération échoue ou a été interrompue.
//!
//! Une candidature ne reste jamais en base sans documents : en cas d'échec
//! elle est supprimée avec ses fichiers partiels, et une génération coupée
//! par la fermeture de l'application est nettoyée au démarrage suivant.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::app_state::AppState;
use crate::db;
use crate::documents;
use crate::pipeline::{self, GeneratedDocuments, GenerationRequest, GENERATION_STEPS_TOTAL};
use crate::error::AppError;
use crate::services::user_errors;
use crate::trace_log;

/// Libellé publié avant la première étape du pipeline.
const PREPARATION_LABEL: &str = "Préparation…";

/// Publie la progression initiale puis lance la génération en tâche de
/// fond. La progression existe donc dès le retour de la commande de
/// création : le premier polling de l'interface ne peut plus tomber sur une
/// génération inconnue.
///
/// # Arguments
///
/// * `state` - État de l'application (cloné dans la tâche).
/// * `candidature_id` - Candidature fraîchement créée, sans documents.
/// * `session_id` - Session de la candidature (fournit les références).
/// * `description_offre` - Texte de l'offre.
pub fn start(state: &AppState, candidature_id: i64, session_id: i64, description_offre: String) {
    state.progress.update(candidature_id, 0, GENERATION_STEPS_TOTAL, PREPARATION_LABEL);
    let state = state.clone();
    tauri::async_runtime::spawn(async move {
        match run(&state, candidature_id, session_id, &description_offre).await {
            Ok(()) => state.progress.mark_completed(candidature_id),
            Err(e) => {
                let error = user_errors::readable(e, "La génération des documents");
                state.progress.mark_failed(candidature_id, &error.to_string());
                discard_candidature(&state.pool, &state.data_dir, candidature_id).await;
            }
        }
    });
}

/// Déroule le pipeline puis persiste son résultat.
///
/// # Arguments
///
/// * `state` - État de l'application.
/// * `candidature_id` - Candidature concernée.
/// * `session_id` - Session de la candidature.
/// * `description_offre` - Texte de l'offre.
///
/// # Returns
///
/// `Ok(())` une fois les documents sauvegardés et rattachés à la candidature.
async fn run(state: &AppState, candidature_id: i64, session_id: i64, description_offre: &str) -> Result<(), AppError> {
    let request = GenerationRequest {
        data_dir: &state.data_dir,
        session_id,
        candidature_id,
        description_offre,
        api_key: &state.api_key,
        llm: &state.llm,
    };
    let progress = state.progress.clone();
    let generated = pipeline::generate_documents(&state.pool, &state.fonts_dir, request, |step, label| {
        progress.update(candidature_id, step, GENERATION_STEPS_TOTAL, label)
    })
    .await?;

    save_trace(&state.data_dir, session_id, candidature_id, &generated);
    let paths = save_documents(&state.data_dir, session_id, candidature_id, &generated)?;
    let versions = (generated.cv.reference_version_id, generated.lm.reference_version_id);
    db::candidatures::set_documents(&state.pool, candidature_id, (&paths.0, &paths.1), versions, &generated.warnings).await
}

/// Écrit la trace de la génération. Elle sert au diagnostic : son échec est
/// tracé sans faire échouer une génération réussie.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Session de la candidature.
/// * `candidature_id` - Candidature concernée.
/// * `generated` - Résultat du pipeline.
fn save_trace(data_dir: &Path, session_id: i64, candidature_id: i64, generated: &GeneratedDocuments) {
    let trace = generated
        .trace
        .files()
        .and_then(|files| documents::save_generation_trace(data_dir, session_id, candidature_id, &files));
    if let Err(e) = trace {
        trace_log::log(&format!("Trace de génération de la candidature {candidature_id} non écrite : {e}"));
    }
}

/// Sauvegarde le CV et la lettre générés.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Session de la candidature.
/// * `candidature_id` - Candidature concernée.
/// * `generated` - Résultat du pipeline.
///
/// # Returns
///
/// Chemins relatifs `(cv, lettre)` des PDF.
fn save_documents(
    data_dir: &Path,
    session_id: i64,
    candidature_id: i64,
    generated: &GeneratedDocuments,
) -> Result<(String, String), AppError> {
    let (cv, lm) = (&generated.cv, &generated.lm);
    let cv_path = documents::save_generated_document(data_dir, session_id, "cv", candidature_id, &cv.pdf, &cv.data_json)?;
    let lm_path = documents::save_generated_document(
        data_dir,
        session_id,
        "lettre_motivation",
        candidature_id,
        &lm.pdf,
        &lm.data_json,
    )?;
    Ok((cv_path, lm_path))
}

/// Supprime une candidature dont la génération a échoué ou a été
/// interrompue, avec ses fichiers partiels. Elle n'a jamais eu de documents
/// ni reçu d'interaction utilisateur (modale bloquante pendant la
/// génération) : rien d'autre à nettoyer. Les échecs sont tracés, jamais
/// propagés — l'erreur montrée à l'utilisateur reste celle de la génération.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `candidature_id` - Candidature à supprimer.
pub async fn discard_candidature(pool: &SqlitePool, data_dir: &Path, candidature_id: i64) {
    let session_id = match db::candidatures::delete(pool, candidature_id).await {
        Ok(Some(session_id)) => session_id,
        Ok(None) => return,
        Err(e) => {
            trace_log::log(&format!("Suppression de la candidature {candidature_id} sans documents impossible : {e}"));
            return;
        }
    };
    if let Err(e) = documents::delete_generated_documents(data_dir, session_id, candidature_id) {
        trace_log::log(&format!("Suppression des fichiers de la candidature {candidature_id} impossible : {e}"));
    }
}

/// Nettoie au démarrage les générations interrompues par la fermeture de
/// l'application : toute candidature « à postuler » sans aucun document est
/// supprimée avec ses fichiers partiels, comme après un échec de génération.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
///
/// # Returns
///
/// Nombre de candidatures supprimées.
pub async fn clean_interrupted(pool: &SqlitePool, data_dir: &Path) -> Result<usize, AppError> {
    let interrupted = db::candidatures::list_interrupted_generations(pool).await?;
    for (candidature_id, _) in &interrupted {
        discard_candidature(pool, data_dir, *candidature_id).await;
    }
    Ok(interrupted.len())
}
