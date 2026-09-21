//! Commandes Tauri pour les requêtes de recherche d'une session.

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::{CreateSearchQueryInput, SearchQueryView};

/// Liste les requêtes de recherche d'une session.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Requêtes triées par date de création croissante.
#[tauri::command]
pub async fn get_search_queries(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
) -> Result<Vec<SearchQueryView>, AppError> {
    api.get(&format!("/api/sessions/{session_id}/search-queries")).await
}

/// Ajoute une requête de recherche à une session.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
/// * `input` - Mots-clés et critères de la requête.
///
/// # Returns
///
/// `SearchQueryView` fraîchement créée.
#[tauri::command]
pub async fn create_search_query(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    input: CreateSearchQueryInput,
) -> Result<SearchQueryView, AppError> {
    api.post(&format!("/api/sessions/{session_id}/search-queries"), &input).await
}

/// Supprime une requête de recherche d'une session.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
/// * `id` - Identifiant de la requête à supprimer.
///
/// # Returns
///
/// `Ok(())` si supprimée.
#[tauri::command]
pub async fn delete_search_query(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    id: i64,
) -> Result<(), AppError> {
    api.delete(&format!("/api/sessions/{session_id}/search-queries/{id}")).await
}

/// Déclenche une recherche complète (scraping + filtre + scoring) pour une session.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// `Ok(())` une fois la recherche terminée.
#[tauri::command]
pub async fn run_session_search(api: tauri::State<'_, ApiClient>, session_id: i64) -> Result<(), AppError> {
    api.post_no_content(&format!("/api/sessions/{session_id}/search"), &()).await
}
