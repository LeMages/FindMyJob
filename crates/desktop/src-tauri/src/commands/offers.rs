//! Commandes Tauri pour les offres scorées d'une session (onglet Marché).

// Importation des modules internes
use crate::api_client::ApiClient;
use crate::error::AppError;
use crate::models::OfferView;

/// Seuil de score appliqué quand le frontend n'en fournit pas.
const DEFAULT_MIN_SCORE: f32 = 0.5;

/// Liste les offres d'une session dont le score dépasse un seuil.
///
/// # Arguments
///
/// * `api` - Client HTTP vers l'agent.
/// * `session_id` - Identifiant de la session.
/// * `min_score` - Seuil exclusif de score (défaut : 0.5).
///
/// # Returns
///
/// Offres triées par score décroissant.
#[tauri::command]
pub async fn get_session_offers(
    api: tauri::State<'_, ApiClient>,
    session_id: i64,
    min_score: Option<f32>,
) -> Result<Vec<OfferView>, AppError> {
    let score = min_score.unwrap_or(DEFAULT_MIN_SCORE);
    api.get(&format!("/api/sessions/{session_id}/offers?min_score={score}")).await
}
