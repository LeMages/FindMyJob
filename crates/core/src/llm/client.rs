//! Client HTTP générique vers l'API Messages d'Anthropic (Claude).
//!
//! Pas de SDK officiel Rust pour l'API Anthropic : les appels passent par
//! `reqwest` en HTTP brut. Ce module ne connaît que le transport (auth,
//! requête, désérialisation minimale) — la construction du prompt et la
//! logique métier vivent dans les modules appelants (ex. `llm::style`).

// Importation des modules standard
use std::sync::LazyLock;
use std::time::Duration;

// Importation des crates externes
use reqwest::Client;
use serde::Deserialize;

// Importation des modules internes
use crate::error::AppError;

/// Endpoint de l'API Messages d'Anthropic.
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
/// Version d'API figée, requise par Anthropic sur chaque requête.
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Modèle utilisé par défaut si aucun n'est configuré en base (voir
/// `db::settings::resolve_llm_model`).
pub const DEFAULT_MODEL: &str = "claude-sonnet-4-6";
/// Modèles proposés dans le sélecteur des paramètres.
pub const AVAILABLE_MODELS: [&str; 2] = ["claude-sonnet-4-6", "claude-haiku-4-5-20251001"];
/// Plafond de tokens de sortie — largement suffisant pour un JSON d'analyse.
/// Visibilité `pub(crate)` pour permettre aux points d'appel utilisant
/// [`call_claude_with_tokens_meta`] de réutiliser le même plafond par défaut.
pub(crate) const MAX_TOKENS: u32 = 8192;
/// Délai maximal d'un appel complet. Large : les générations Typst/prose
/// produisent des milliers de tokens et peuvent durer plusieurs minutes.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
/// Délai maximal d'établissement de la connexion à l'API.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Client HTTP partagé par tous les appels (réutilise les connexions TLS).
/// L'échec de construction est conservé en texte pour être remonté à chaque
/// appel plutôt que de faire paniquer le processus.
static CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())
});

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
/// Utilise le plafond de tokens par défaut ([`MAX_TOKENS`]), adapté aux
/// réponses JSON courtes (analyse de style, d'offre, de matching...). Pour
/// les réponses volumineuses (ex. assemblage Typst), voir
/// [`call_claude_with_tokens`].
///
/// # Arguments
///
/// * `system` - Prompt système (instructions, contexte de la tâche).
/// * `user_message` - Message utilisateur (contenu à traiter).
/// * `output_schema` - Schéma JSON optionnel : si fourni, la requête active
///   les structured outputs (`output_config.format`) pour garantir une
///   réponse JSON strictement conforme.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// Le texte du premier bloc de type `text` dans la réponse.
pub async fn call_claude(
    system: &str,
    user_message: &str,
    output_schema: Option<serde_json::Value>,
    model: &str,
) -> Result<String, AppError> {
    call_claude_with_tokens(system, user_message, output_schema, MAX_TOKENS, model).await
}

/// Variante de [`call_claude`] avec un plafond de tokens de sortie explicite.
///
/// Nécessaire pour les étapes produisant du code Typst complet (CV/lettre),
/// dont la longueur dépasse largement le plafond par défaut d'une réponse JSON.
///
/// # Arguments
///
/// * `system` - Prompt système (instructions, contexte de la tâche).
/// * `user_message` - Message utilisateur (contenu à traiter).
/// * `output_schema` - Schéma JSON optionnel : si fourni, la requête active
///   les structured outputs (`output_config.format`) pour garantir une
///   réponse JSON strictement conforme.
/// * `max_tokens` - Plafond de tokens de sortie pour cette requête.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// Le texte du premier bloc de type `text` dans la réponse.
pub async fn call_claude_with_tokens(
    system: &str,
    user_message: &str,
    output_schema: Option<serde_json::Value>,
    max_tokens: u32,
    model: &str,
) -> Result<String, AppError> {
    call_claude_with_tokens_meta(system, user_message, output_schema, max_tokens, model)
        .await
        .map(|(text, _stop_reason)| text)
}

/// Variante de [`call_claude_with_tokens`] qui expose aussi le `stop_reason`
/// brut de la réponse, en plus du texte. Utilisée par les points d'appel qui
/// ont besoin de logger le `stop_reason` d'un appel réussi (diagnostic du
/// pipeline LM) sans dupliquer la logique d'appel réseau.
///
/// # Returns
///
/// Le texte du premier bloc `text` de la réponse, et le `stop_reason` associé.
pub async fn call_claude_with_tokens_meta(
    system: &str,
    user_message: &str,
    output_schema: Option<serde_json::Value>,
    max_tokens: u32,
    model: &str,
) -> Result<(String, Option<String>), AppError> {
    // ------------------------------------------------------------------
    // 1. Clé API — lue directement depuis l'environnement du processus.
    //    `core` ignore d'où vient cette variable (.env en développement,
    //    trousseau OS en production) : cette résolution vit dans le
    //    `security` de l'appelant (ex. `findmyjob-agent`).
    // ------------------------------------------------------------------
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| AppError::Config("Aucune clé API Anthropic configurée.".to_string()))?;

    // ------------------------------------------------------------------
    // 2. Construction du corps de la requête
    // ------------------------------------------------------------------
    // On désactive le thinking : ce pipeline est une génération structurée en
    // un seul tour (JSON ou Typst), le thinking n'apporte rien et partage le
    // même budget de tokens que la sortie visible — l'activer par défaut
    // tronquait les réponses avant même d'atteindre le contenu utile.
    let mut body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": [{"role": "user", "content": user_message}],
        "thinking": { "type": "disabled" },
    });
    if let Some(schema) = output_schema {
        body["output_config"] = serde_json::json!({
            "format": { "type": "json_schema", "schema": schema }
        });
    }

    // ------------------------------------------------------------------
    // 3. Appel réseau
    // ------------------------------------------------------------------
    let client = CLIENT
        .as_ref()
        .map_err(|e| AppError::Config(format!("Impossible d'initialiser le client HTTP Claude : {e}")))?;
    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", &api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::Llm("L'API Claude n'a pas répondu dans le délai imparti.".to_string())
            } else {
                AppError::Llm(format!("Erreur réseau API Claude: {e}"))
            }
        })?;

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
        eprintln!(
            "[llm::client] Réponse refusée par l'API Claude (stop_reason={:?})",
            api_response.stop_reason
        );
        return Err(AppError::Llm(
            "L'API Claude a refusé de traiter cette requête.".to_string(),
        ));
    }

    // On vérifie ensuite le troncage : max_tokens atteint signifie que le
    // contenu (JSON ou Typst) est incomplet et ne pourra pas être parsé
    // correctement — mieux vaut échouer explicitement ici que de laisser
    // l'appelant se heurter à une erreur de parsing cryptique en aval.
    if api_response.stop_reason.as_deref() == Some("max_tokens") {
        eprintln!(
            "[llm::client] Réponse tronquée par l'API Claude (stop_reason={:?})",
            api_response.stop_reason
        );
        return Err(AppError::Llm(
            "Réponse tronquée par le modèle (max_tokens atteint). Le contenu généré est incomplet."
                .to_string(),
        ));
    }

    let stop_reason = api_response.stop_reason.clone();

    let text = api_response
        .content
        .iter()
        .find(|b| b.block_type == "text")
        .and_then(|b| b.text.clone())
        .ok_or_else(|| {
            eprintln!(
                "[llm::client] Pas de contenu texte dans la réponse API (stop_reason={:?})",
                api_response.stop_reason
            );
            AppError::Llm("Pas de contenu texte dans la réponse API".to_string())
        })?;

    Ok((text, stop_reason))
}
