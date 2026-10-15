//! Cible d'un appel à l'API Claude : clé API, modèle et exécutant de
//! l'appel, résolus par l'appelant et transmis à chaque fonction de `llm`.

// Importation des modules standard
use std::fmt;
use std::sync::Arc;

// Importation des modules internes
use crate::llm::api_key::ApiKey;
use crate::llm::caller::LlmCaller;

/// Clé, modèle et exécutant à utiliser pour un appel à l'API Claude.
///
/// # Attributes
///
/// * `api_key` - Clé API Anthropic.
/// * `model` - Identifiant du modèle Claude.
/// * `caller` - Exécutant de l'appel ; toujours
///   [`AnthropicCaller`](crate::llm::client::AnthropicCaller) dans
///   l'application, remplacé seulement par les tests.
#[derive(Clone)]
pub struct ClaudeTarget {
    pub api_key: ApiKey,
    pub model: String,
    pub caller: Arc<dyn LlmCaller>,
}

impl fmt::Debug for ClaudeTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaudeTarget")
            .field("api_key", &self.api_key)
            .field("model", &self.model)
            .field("caller", &self.caller)
            .finish()
    }
}
