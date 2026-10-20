//! Point d'injection des appels à l'API Claude.
//!
//! Toutes les fonctions de `llm` passent par [`LlmCaller`], porté par le
//! [`ClaudeTarget`](crate::llm::target::ClaudeTarget) de l'appelant.
//! L'application utilise toujours l'implémentation réelle
//! ([`AnthropicCaller`](crate::llm::client::AnthropicCaller)) ; le trait
//! permet aux tests de substituer des réponses figées sans réseau, sans
//! rendre l'URL du fournisseur configurable.

// Importation des modules standard
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

// Importation des modules internes
use crate::error::AppError;
use crate::llm::api_key::ApiKey;
use crate::llm::call_record::TokenUsage;

/// Requête envoyée au modèle, déjà résolue (clé, modèle, prompts).
///
/// # Attributes
///
/// * `system` - Prompt système.
/// * `user_message` - Message utilisateur.
/// * `output_schema` - Schéma JSON des structured outputs, s'il y en a un.
/// * `max_tokens` - Plafond de tokens de sortie.
/// * `api_key` - Clé API Anthropic.
/// * `model` - Identifiant du modèle Claude.
pub struct LlmRequest<'a> {
    pub system: &'a str,
    pub user_message: &'a str,
    pub output_schema: Option<serde_json::Value>,
    pub max_tokens: u32,
    pub api_key: &'a ApiKey,
    pub model: &'a str,
}

/// Réponse textuelle du modèle.
///
/// # Attributes
///
/// * `text` - Texte du premier bloc `text` de la réponse.
/// * `stop_reason` - `stop_reason` brut renvoyé par l'API, pour le diagnostic.
/// * `usage` - Tokens consommés, si l'API les a rapportés.
/// * `request_id` - Identifiant de requête Anthropic (en-tête `request-id`).
pub struct LlmResponse {
    pub text: String,
    pub stop_reason: Option<String>,
    pub usage: Option<TokenUsage>,
    pub request_id: Option<String>,
}

/// Future renvoyé par [`LlmCaller::call`], boxé pour que le trait reste
/// utilisable en objet (`Arc<dyn LlmCaller>`) sans dépendance `async-trait`.
pub type LlmFuture<'a> = Pin<Box<dyn Future<Output = Result<LlmResponse, AppError>> + Send + 'a>>;

/// Exécute un appel au modèle de langage.
pub trait LlmCaller: Send + Sync + Debug {
    /// Envoie la requête et retourne la réponse textuelle.
    ///
    /// # Arguments
    ///
    /// * `request` - Requête complète (prompts, schéma, clé, modèle).
    ///
    /// # Returns
    ///
    /// Le texte de la réponse et son `stop_reason`, ou une erreur
    /// `AppError::Llm` / `AppError::Config`.
    fn call<'a>(&'a self, request: LlmRequest<'a>) -> LlmFuture<'a>;
}
