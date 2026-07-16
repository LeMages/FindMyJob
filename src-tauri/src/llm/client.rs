//! Client HTTP générique vers l'API Messages d'Anthropic (Claude).
//!
//! Pas de SDK officiel Rust pour l'API Anthropic : les appels passent par
//! `reqwest` en HTTP brut. Ce module ne connaît que le transport (auth,
//! requête, désérialisation minimale) — la construction du prompt et la
//! logique métier vivent dans les modules appelants (ex. `llm::style`).

// Importation des crates externes
use reqwest::Client;
use serde::Deserialize;

// Importation des modules internes
use crate::error::AppError;

/// Endpoint de l'API Messages d'Anthropic.
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
/// Version d'API figée, requise par Anthropic sur chaque requête.
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Modèle utilisé pour les appels applicatifs (analyse de style, etc.).
const MODEL: &str = "claude-sonnet-5";
/// Plafond de tokens de sortie — largement suffisant pour un JSON d'analyse.
const MAX_TOKENS: u32 = 4096;

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

/// Envoie un message à l'API Claude et retourne le texte de la réponse.
///
/// # Arguments
///
/// * `system` - Prompt système (instructions, contexte de la tâche).
/// * `user_message` - Message utilisateur (contenu à traiter).
/// * `output_schema` - Schéma JSON optionnel : si fourni, la requête active
///   les structured outputs (`output_config.format`) pour garantir une
///   réponse JSON strictement conforme.
///
/// # Returns
///
/// Le texte du premier bloc de type `text` dans la réponse.
pub async fn call_claude(
    system: &str,
    user_message: &str,
    output_schema: Option<serde_json::Value>,
) -> Result<String, AppError> {
    // ------------------------------------------------------------------
    // 1. Clé API — lue à la volée depuis l'environnement (.env chargé au
    //    démarrage de l'app via dotenvy, voir lib.rs)
    // ------------------------------------------------------------------
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| AppError::Config("ANTHROPIC_API_KEY non définie".to_string()))?;

    // ------------------------------------------------------------------
    // 2. Construction du corps de la requête
    // ------------------------------------------------------------------
    let mut body = serde_json::json!({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        "system": system,
        "messages": [{"role": "user", "content": user_message}],
    });
    if let Some(schema) = output_schema {
        body["output_config"] = serde_json::json!({
            "format": { "type": "json_schema", "schema": schema }
        });
    }

    // ------------------------------------------------------------------
    // 3. Appel réseau
    // ------------------------------------------------------------------
    let client = Client::new();
    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", &api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Llm(format!("Erreur réseau API Claude: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(AppError::Llm(format!(
            "API Claude erreur {status}: {body_text}"
        )));
    }

    // ------------------------------------------------------------------
    // 4. Désérialisation et extraction du texte
    // ------------------------------------------------------------------
    let api_response: ApiResponse = response
        .json()
        .await
        .map_err(|e| AppError::Llm(format!("Réponse API Claude invalide: {e}")))?;

    // On vérifie stop_reason avant de lire content : un refus (classifieur
    // de sécurité) renvoie un contenu vide ou partiel qu'il ne faut pas
    // interpréter comme une réponse exploitable.
    if api_response.stop_reason.as_deref() == Some("refusal") {
        return Err(AppError::Llm(
            "L'API Claude a refusé de traiter cette requête.".to_string(),
        ));
    }

    api_response
        .content
        .iter()
        .find(|b| b.block_type == "text")
        .and_then(|b| b.text.clone())
        .ok_or_else(|| AppError::Llm("Pas de contenu texte dans la réponse API".to_string()))
}
