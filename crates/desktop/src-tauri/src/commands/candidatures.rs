//! Commandes Tauri pour les candidatures.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{
    CandidatureDetail, CandidatureView, CreateCandidatureInput, GenerationProgress, UpdateCandidatureInput,
};

/// Crée une candidature. Si la session cible a ses documents de référence
/// prêts, l'agent lance la génération du CV et de la lettre de motivation en
/// tâche de fond : cette commande répond dès l'insertion en base, sans
/// attendre la génération — `cv_path`/`lettre_motivation_path` valent `None`
/// dans la réponse. Utiliser `get_generation_progress` pour suivre l'avancement.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `input` - Données de la candidature à créer.
///
/// # Returns
///
/// `CandidatureView` créée (documents pas encore générés).
#[tauri::command]
pub async fn create_candidature(
    api: tauri::State<'_, ApiClient>,
    input: CreateCandidatureInput,
) -> Result<CandidatureView, AppError> {
    api.post("/api/candidatures", &input).await
}

/// Récupère la progression de la génération des documents d'une candidature,
/// pour affichage d'une barre de progression pendant `create_candidature`.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `candidature_id` - Identifiant de la candidature suivie.
///
/// # Returns
///
/// `GenerationProgress` courante, ou `AppError::NotFound` si aucune
/// génération n'est en cours ni récemment terminée pour cet identifiant.
#[tauri::command]
pub async fn get_generation_progress(
    api: tauri::State<'_, ApiClient>,
    candidature_id: i64,
) -> Result<GenerationProgress, AppError> {
    api.get(&format!("/api/candidatures/{candidature_id}/progress")).await
}

/// Liste les candidatures d'une session.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Candidatures de la session, les plus récentes en premier.
#[tauri::command]
pub async fn get_candidatures(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<Vec<CandidatureView>, AppError> {
    api.get(&format!("/api/sessions/{session_id}/candidatures")).await
}

/// Récupère le détail complet d'une candidature, pour la page de détail.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `CandidatureDetail` correspondante, ou `AppError::NotFound` si elle n'existe pas.
#[tauri::command]
pub async fn get_candidature(
    api: tauri::State<'_, ApiClient>,
    id: i64,
) -> Result<CandidatureDetail, AppError> {
    api.get(&format!("/api/candidatures/{id}")).await
}

/// Met à jour les champs éditables d'une candidature (détails du poste et
/// notes), refusé si son statut n'est plus « à postuler ».
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature à modifier.
/// * `input` - Nouvelles valeurs des champs éditables.
///
/// # Returns
///
/// `CandidatureDetail` mise à jour.
#[tauri::command]
pub async fn update_candidature(
    api: tauri::State<'_, ApiClient>,
    id: i64,
    input: UpdateCandidatureInput,
) -> Result<CandidatureDetail, AppError> {
    api.put(&format!("/api/candidatures/{id}"), &input).await
}

/// Marque une candidature comme abandonnée (statut `abandon`) — utilisé par
/// le bouton « Supprimer » de la page de détail, tant que la candidature est
/// encore au statut « à postuler ».
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature à abandonner.
///
/// # Returns
///
/// `Ok(())` une fois le statut mis à jour.
#[tauri::command]
pub async fn abandon_candidature(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/candidatures/{id}/abandon"), &()).await
}

/// Traite le bouton « Postuler sans message » : exporte le CV et la lettre de
/// motivation vers le dossier de téléchargements de l'utilisateur, puis passe
/// la candidature au statut `postule`.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` une fois les documents exportés et le statut mis à jour.
#[tauri::command]
pub async fn postuler_sans_message(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/candidatures/{id}/postuler"), &()).await
}

/// Traite le bouton « Postuler par mail » : fait rédiger un mail de
/// candidature par l'IA, exporte CV, lettre de motivation et mail vers le
/// dossier de téléchargements, puis passe la candidature au statut `postule`.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` une fois le mail rédigé, les documents exportés et le statut mis à jour.
#[tauri::command]
pub async fn postuler_par_mail(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/candidatures/{id}/postuler/mail"), &())
        .await
}

/// Traite le bouton « Postuler par message » : fait rédiger un message court
/// (LinkedIn ou équivalent) par l'IA, exporte CV, lettre de motivation et
/// message vers le dossier de téléchargements, puis passe la candidature au
/// statut `postule`.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` une fois le message rédigé, les documents exportés et le statut mis à jour.
#[tauri::command]
pub async fn postuler_par_message(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/candidatures/{id}/postuler/message"), &())
        .await
}
