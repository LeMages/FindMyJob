//! Journal des appels à Claude : modèle, tokens, durée, `stop_reason`,
//! identifiant de requête et prompts de chaque appel, pour diagnostiquer et
//! rejouer un échec.
//!
//! Chaque appel émet une ligne de synthèse dans `trace_log` (jamais les
//! prompts). Quand la cible porte un [`CallRecorder`], l'appel complet y est
//! aussi conservé ; l'appelant l'écrit ensuite dans son dossier de debug.

// Importation des modules standard
use std::sync::Mutex;

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::trace_log;

/// Tokens consommés par un appel, tels que rapportés par l'API.
///
/// # Attributes
///
/// * `input_tokens` - Tokens du prompt.
/// * `output_tokens` - Tokens générés.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Un appel à Claude, réussi ou non.
///
/// # Attributes
///
/// * `label` - Étape qui a fait l'appel (ex. `"templatisation:exemple"`).
/// * `model` - Modèle demandé.
/// * `usage` - Tokens consommés, absents en cas d'erreur.
/// * `duration_ms` - Durée de l'appel.
/// * `stop_reason` - `stop_reason` renvoyé par l'API.
/// * `request_id` - Identifiant de requête Anthropic (en-tête `request-id`).
/// * `system` - Prompt système envoyé.
/// * `user` - Message utilisateur envoyé.
/// * `response` - Texte reçu, ou message d'erreur préfixé de `ERREUR : `.
#[derive(Debug, Clone, Serialize)]
pub struct LlmCallRecord {
    pub label: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub duration_ms: u128,
    pub stop_reason: Option<String>,
    pub request_id: Option<String>,
    pub system: String,
    pub user: String,
    pub response: String,
}

impl LlmCallRecord {
    /// Ligne de synthèse de l'appel, sans prompt ni réponse.
    ///
    /// # Returns
    ///
    /// Une ligne lisible pour `trace_log`.
    pub fn summary(&self) -> String {
        let usage = self
            .usage
            .map(|u| format!("{} entrée / {} sortie", u.input_tokens, u.output_tokens))
            .unwrap_or_else(|| "tokens inconnus".to_string());
        let outcome = if self.response.starts_with(ERROR_PREFIX) { "échec" } else { "ok" };
        format!(
            "[llm] {} : {outcome}, modèle {}, {usage}, {} ms, stop_reason={:?}, request_id={:?}",
            self.label, self.model, self.duration_ms, self.stop_reason, self.request_id
        )
    }
}

/// Préfixe de [`LlmCallRecord::response`] pour un appel en erreur.
pub const ERROR_PREFIX: &str = "ERREUR : ";

/// Collecteur d'appels partagé par une opération (une templatisation, une
/// génération). Thread-safe : les appels parallèles y écrivent ensemble.
///
/// # Attributes
///
/// * `calls` - Appels enregistrés, dans l'ordre de fin.
#[derive(Debug, Default)]
pub struct CallRecorder {
    calls: Mutex<Vec<LlmCallRecord>>,
}

impl CallRecorder {
    /// Ajoute un appel au journal.
    ///
    /// # Arguments
    ///
    /// * `record` - Appel terminé.
    pub fn push(&self, record: LlmCallRecord) {
        // Un verrou empoisonné ne protège qu'une liste de traces : on continue.
        self.calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push(record);
    }

    /// Copie des appels enregistrés.
    ///
    /// # Returns
    ///
    /// Les appels, dans l'ordre de fin.
    pub fn snapshot(&self) -> Vec<LlmCallRecord> {
        self.calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    /// Retire et renvoie les appels enregistrés depuis le dernier `drain`.
    ///
    /// # Returns
    ///
    /// Les appels, dans l'ordre de fin ; le journal est vidé.
    pub fn drain(&self) -> Vec<LlmCallRecord> {
        std::mem::take(&mut *self.calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}

/// Trace la synthèse d'un appel puis l'ajoute au collecteur s'il y en a un.
///
/// # Arguments
///
/// * `record` - Appel terminé.
/// * `recorder` - Collecteur de l'opération en cours, s'il y en a un.
pub fn report(record: LlmCallRecord, recorder: Option<&CallRecorder>) {
    trace_log::log(&record.summary());
    if let Some(recorder) = recorder {
        recorder.push(record);
    }
}
