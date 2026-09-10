//! Récupération du HTML brut d'une page d'offre d'emploi.

// Importation des modules standard
use std::time::Duration;

// Importation des crates externes
use reqwest::Client;

// Importation des modules internes
use crate::error::AppError;

/// User-Agent envoyé avec chaque requête — certains sites d'offres bloquent
/// les clients HTTP sans en-tête `User-Agent` reconnaissable.
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// Délai maximal d'attente d'une requête de scraping.
const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

/// Récupère le HTML brut d'une URL d'offre d'emploi.
///
/// Suit les redirections (comportement par défaut de `reqwest`), avec un
/// timeout de 15 secondes et un User-Agent de navigateur réaliste.
///
/// # Arguments
///
/// * `url` - URL de l'offre à récupérer.
///
/// # Returns
///
/// Le corps HTML de la réponse, ou une erreur explicite si le site bloque
/// l'accès, ne répond pas dans le délai imparti, ou renvoie un contenu vide.
pub async fn fetch_html(url: &str) -> Result<String, AppError> {
    // ------------------------------------------------------------------
    // 1. Requête HTTP
    // ------------------------------------------------------------------
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| AppError::Scraping(format!("Impossible d'initialiser le client HTTP: {e}")))?;

    let response = client.get(url).send().await.map_err(|e| {
        if e.is_timeout() {
            AppError::Scraping(
                "Le site n'a pas répondu à temps. Essaie la saisie manuelle.".to_string(),
            )
        } else {
            AppError::Scraping(format!(
                "Impossible d'accéder à cette URL: {e}. Essaie la saisie manuelle."
            ))
        }
    })?;

    // ------------------------------------------------------------------
    // 2. Statut HTTP — certains sites (LinkedIn, Indeed...) bloquent le
    //    scraping côté serveur (403, 999, 429...) : message explicite plutôt
    //    que de tenter un contournement des protections.
    // ------------------------------------------------------------------
    if !response.status().is_success() {
        let status = response.status();
        return Err(AppError::Scraping(format!(
            "Ce site bloque l'extraction automatique (erreur {status}). Essaie la saisie manuelle."
        )));
    }

    // ------------------------------------------------------------------
    // 3. Corps de la réponse
    // ------------------------------------------------------------------
    let html = response
        .text()
        .await
        .map_err(|e| AppError::Scraping(format!("Réponse illisible: {e}")))?;

    if html.trim().is_empty() {
        return Err(AppError::Scraping(
            "La page est vide. Essaie la saisie manuelle.".to_string(),
        ));
    }

    Ok(html)
}
