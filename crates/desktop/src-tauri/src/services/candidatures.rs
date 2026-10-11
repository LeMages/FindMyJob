//! Règles métier des candidatures qui dépassent un accès base : création
//! conditionnée aux références et suivie de la génération des documents,
//! détection de doublons par lien, suppression avec les fichiers, et
//! « postuler » avec rédaction du message par l'IA.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::app_state::AppState;
use crate::clock;
use crate::db;
use crate::documents;
use crate::services::candidature_validation::validate_create;
use crate::services::generation;
use crate::url_normalize::normalize_offer_url;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CandidatureView, CreateCandidatureInput, DuplicateCandidatureView};
use findmyjob_core::trace_log;

/// Indique si les deux références (CV et lettre) d'une session sont
/// templatisées — seule condition sous laquelle une candidature peut être
/// créée et sa génération réussir.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Session cible.
///
/// # Returns
///
/// `true` si les deux références sont prêtes.
async fn references_ready(pool: &SqlitePool, session_id: i64) -> Result<bool, AppError> {
    Ok(db::documents::status_for_session(pool, session_id).await?.all_ready)
}

/// Crée une candidature puis lance en tâche de fond la génération de son CV
/// et de sa lettre de motivation. La commande n'attend pas la génération :
/// `cv_path` et `lettre_motivation_path` restent `None` à ce stade, la
/// progression est consultable dès le retour (voir `services::generation`).
/// Si les références de la session ne sont pas prêtes, la création est
/// refusée plutôt que de créer une candidature sans document.
///
/// # Arguments
///
/// * `state` - État de l'application.
/// * `input` - Données saisies.
///
/// # Returns
///
/// `CandidatureView` créée, ou `AppError::Validation`.
pub async fn create(state: &AppState, input: &CreateCandidatureInput) -> Result<CandidatureView, AppError> {
    if let Some(session_id) = input.session_id {
        if !references_ready(&state.pool, session_id).await? {
            return Err(AppError::Validation(
                "Les documents de référence (CV et lettre de motivation) doivent être uploadés \
                 et convertis en template avant de pouvoir postuler."
                    .to_string(),
            ));
        }
    }

    let new = validate_create(input)?;
    let view = db::candidatures::insert(
        &state.pool,
        new.session_id,
        &new.fields,
        &new.description_offre,
        &new.contacts,
        &clock::now_utc(),
    )
    .await?;

    generation::start(state, view.id, new.session_id, new.description_offre);
    Ok(view)
}

/// Cherche les candidatures existantes dont le lien d'offre correspond,
/// après normalisation (schéma, `www.`, query, fragment, slash final,
/// casse), au lien donné, toutes sessions confondues. Sert à un
/// avertissement non bloquant : liste vide si rien ne correspond.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `url` - Lien saisi.
///
/// # Returns
///
/// Candidatures existantes pour cette offre.
pub async fn find_by_offer_url(pool: &SqlitePool, url: &str) -> Result<Vec<DuplicateCandidatureView>, AppError> {
    let Some(target) = normalize_offer_url(url) else {
        return Ok(Vec::new());
    };
    let candidatures = db::joins::list_candidatures_with_offer_url(pool).await?;
    Ok(candidatures
        .into_iter()
        .filter(|(url, _)| normalize_offer_url(url).as_deref() == Some(target.as_str()))
        .map(|(_, candidature)| candidature)
        .collect())
}

/// Supprime définitivement une candidature au statut « à postuler », puis
/// ses documents générés. Le nettoyage disque est tracé en cas d'échec sans
/// renvoyer d'erreur : la base est déjà cohérente.
///
/// # Arguments
///
/// * `state` - État de l'application.
/// * `id` - Candidature à supprimer.
///
/// # Returns
///
/// `Ok(())`, `AppError::NotFound` ou `AppError::Conflict` (statut).
pub async fn delete(state: &AppState, id: i64) -> Result<(), AppError> {
    let session_id = db::candidatures::delete_a_postuler(&state.pool, id).await?;
    if let Err(e) = documents::delete_generated_documents(&state.data_dir, session_id, id) {
        trace_log::log(&format!("Suppression des documents de la candidature {id} impossible : {e}"));
    }
    Ok(())
}

/// Marque une candidature comme postulée sans message.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Candidature concernée.
///
/// # Returns
///
/// `Ok(())`, `AppError::NotFound` ou `AppError::Conflict`.
pub async fn postuler_sans_message(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    db::candidatures::mark_postule(pool, id, None, &clock::now_utc()).await
}

/// Type de message de candidature rédigé par l'IA.
#[derive(Clone, Copy)]
pub enum ApplicationMessage {
    /// Mail de candidature complet.
    Mail,
    /// Message court (plateforme, réseau social).
    Message,
}

/// Fait rédiger le message de candidature par l'IA, puis marque la
/// candidature comme postulée avec ce message. L'appel LLM a lieu hors
/// transaction ; seul le changement de statut en ouvre une.
///
/// # Arguments
///
/// * `state` - État de l'application (base, clé API).
/// * `id` - Candidature concernée.
/// * `kind` - Mail ou message court.
///
/// # Returns
///
/// Le texte rédigé, que l'interface exporte en `.txt`.
pub async fn postuler_avec_message(state: &AppState, id: i64, kind: ApplicationMessage) -> Result<String, AppError> {
    let pool = &state.pool;
    let detail = db::candidatures::fetch_detail(pool, id).await?;
    let profile = db::profile::build_profile(pool).await?;
    // Le message est adressé au premier contact lié (ordre de liaison).
    let contact = db::contacts::list_for_candidature(pool, id).await?.into_iter().next();
    let target = state.claude().resolve().await?;

    let text = match kind {
        ApplicationMessage::Mail => {
            findmyjob_core::llm::generate_application_email(&profile, &detail, contact.as_ref(), &target).await?
        }
        ApplicationMessage::Message => {
            findmyjob_core::llm::generate_application_message(&profile, &detail, contact.as_ref(), &target).await?
        }
    };
    db::candidatures::mark_postule(pool, id, Some(&text), &clock::now_utc()).await?;
    Ok(text)
}
