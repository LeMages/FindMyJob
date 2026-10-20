//! Cible d'un appel à l'API Claude : clé API, modèle et exécutant de
//! l'appel, résolus par l'appelant et transmis à chaque fonction de `llm`.

// Importation des modules standard
use std::fmt;
use std::sync::Arc;

// Importation des modules internes
use crate::llm::api_key::ApiKey;
use crate::llm::call_record::CallRecorder;
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
/// * `recorder` - Journal des appels de l'opération en cours, s'il y en a
///   un (prompts et réponses complets, pour le dossier de debug).
#[derive(Clone)]
pub struct ClaudeTarget {
    pub api_key: ApiKey,
    pub model: String,
    pub caller: Arc<dyn LlmCaller>,
    pub recorder: Option<Arc<CallRecorder>>,
}

impl ClaudeTarget {
    /// Même cible, avec un autre modèle (étape au modèle imposé).
    ///
    /// # Arguments
    ///
    /// * `model` - Identifiant du modèle à utiliser.
    ///
    /// # Returns
    ///
    /// Une copie de la cible avec ce modèle.
    pub fn with_model(&self, model: &str) -> Self {
        Self { model: model.to_string(), ..self.clone() }
    }

    /// Même cible, dont les appels sont aussi enregistrés dans `recorder`.
    ///
    /// # Arguments
    ///
    /// * `recorder` - Journal de l'opération en cours.
    ///
    /// # Returns
    ///
    /// Une copie de la cible portant ce journal.
    pub fn with_recorder(&self, recorder: Arc<CallRecorder>) -> Self {
        Self { recorder: Some(recorder), ..self.clone() }
    }
}

impl fmt::Debug for ClaudeTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaudeTarget")
            .field("api_key", &self.api_key)
            .field("model", &self.model)
            .field("caller", &self.caller)
            .field("recorder", &self.recorder.is_some())
            .finish()
    }
}
