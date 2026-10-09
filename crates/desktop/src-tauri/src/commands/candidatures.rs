//! Commandes Tauri pour les candidatures.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{
    CandidatureDetail, CandidatureView, CreateCandidatureInput, DuplicateCandidatureView, GenerationProgress,
    UpdateCandidatureInput,
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

/// Cherche les candidatures existantes pointant vers la même offre que
/// `url` (comparaison normalisée faite par l'agent), toutes sessions
/// confondues, pour un avertissement de doublon non bloquant.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `url` - URL de l'offre saisie.
///
/// # Returns
///
/// Les candidatures correspondantes (liste vide si aucune).
#[tauri::command]
pub async fn find_candidatures_by_url(
    api: tauri::State<'_, ApiClient>,
    url: String,
) -> Result<Vec<DuplicateCandidatureView>, AppError> {
    api.get_query("/api/candidatures/by-url", &[("url", url.as_str())]).await
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

/// Supprime définitivement une candidature — utilisé par le bouton
/// « Supprimer » de la page de détail. L'agent refuse la suppression si la
/// candidature n'est plus au statut « à postuler ».
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature à supprimer.
///
/// # Returns
///
/// `Ok(())` une fois la candidature supprimée.
#[tauri::command]
pub async fn delete_candidature(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.delete(&format!("/api/candidatures/{id}")).await
}

/// Traite le bouton « Postuler sans message » : passe la candidature au
/// statut `postule`. Le téléchargement du CV et de la lettre est fait ensuite
/// par le frontend.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` une fois le statut mis à jour.
#[tauri::command]
pub async fn postuler_sans_message(api: tauri::State<'_, ApiClient>, id: i64) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/candidatures/{id}/postuler"), &()).await
}

/// Traite le bouton « Postuler par mail » : fait rédiger un mail de
/// candidature par l'IA et passe la candidature au statut `postule`. Le
/// frontend exporte ensuite CV, lettre de motivation et mail (`.txt`).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// Texte du mail rédigé par l'IA.
#[tauri::command]
pub async fn postuler_par_mail(api: tauri::State<'_, ApiClient>, id: i64) -> Result<String, AppError> {
    api.post(&format!("/api/candidatures/{id}/postuler/mail"), &())
        .await
}

/// Traite le bouton « Postuler par message » : fait rédiger un message court
/// (LinkedIn ou équivalent) par l'IA et passe la candidature au statut
/// `postule`. Le frontend exporte ensuite CV, lettre de motivation et
/// message (`.txt`).
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// Texte du message rédigé par l'IA.
#[tauri::command]
pub async fn postuler_par_message(api: tauri::State<'_, ApiClient>, id: i64) -> Result<String, AppError> {
    api.post(&format!("/api/candidatures/{id}/postuler/message"), &())
        .await
}
