//! DTO de l'état de la clé API Anthropic (écran Paramètres). La clé
//! complète ne traverse jamais le pont IPC : seuls ses derniers caractères.

/// Emplacement d'où la clé API est lue, selon le mode d'exécution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
    /// Variable `ANTHROPIC_API_KEY` du `.env` (mode DEV).
    Env,
    /// Trousseau du système (mode PROD).
    Keyring,
}

/// État de la clé API Anthropic.
///
/// # Attributes
///
/// * `present` - Vrai si une clé est configurée.
/// * `source` - Emplacement d'où la clé est lue.
/// * `last4` - 4 derniers caractères de la clé, si elle est présente.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyStatus {
    pub present: bool,
    pub source: ApiKeySource,
    pub last4: Option<String>,
}
