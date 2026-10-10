//! Cible d'un appel à l'API Claude : clé API et modèle, résolus par
//! l'appelant et transmis à chaque fonction de `llm`.

// Importation des modules internes
use crate::llm::api_key::ApiKey;

/// Clé et modèle à utiliser pour un appel à l'API Claude.
///
/// # Attributes
///
/// * `api_key` - Clé API Anthropic.
/// * `model` - Identifiant du modèle Claude.
#[derive(Debug, Clone)]
pub struct ClaudeTarget {
    pub api_key: ApiKey,
    pub model: String,
}
