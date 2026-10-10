//! Clé API Anthropic : d'où elle vient selon le mode d'exécution, et sa
//! gestion depuis les Paramètres.
//!
//! Règle : en mode DEV, la clé est la variable `ANTHROPIC_API_KEY` du
//! `.env` et le trousseau n'est jamais consulté. En mode PROD, elle est lue
//! dans le trousseau du système et une clé présente dans un `.env` est
//! ignorée. La valeur complète ne sort d'ici que vers le client Claude.

// Importation des modules standard
use std::sync::Arc;

// Importation des modules internes
use crate::secrets::store::SecretStore;
use crate::startup::run_mode::RunMode;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::ApiKey;
use findmyjob_core::models::{ApiKeySource, ApiKeyStatus};

/// Nom de la variable d'environnement lue en mode DEV.
pub const ENV_VAR: &str = "ANTHROPIC_API_KEY";

/// Compte de l'entrée du trousseau (le service est l'identifiant de l'application).
pub const KEYRING_ACCOUNT: &str = "anthropic_api_key";

/// Message de clé absente, complété selon le mode.
const MISSING_KEY: &str = "Aucune clé API Anthropic configurée.";

/// Accès à la clé API pour le mode d'exécution courant. Clonable à coût
/// constant, partagé par les commandes et les tâches de fond.
///
/// # Attributes
///
/// * `mode` - Mode d'exécution décidé au démarrage.
/// * `store` - Trousseau du système (ou faux en test), consulté en PROD seulement.
/// * `read_env` - Lecture de `ANTHROPIC_API_KEY`, appelée en DEV seulement.
#[derive(Clone)]
pub struct ApiKeyAccess {
    mode: RunMode,
    store: Arc<dyn SecretStore>,
    read_env: fn() -> Option<String>,
}

impl ApiKeyAccess {
    /// Construit l'accès à la clé.
    ///
    /// # Arguments
    ///
    /// * `mode` - Mode d'exécution décidé au démarrage.
    /// * `store` - Trousseau du système.
    /// * `read_env` - Lecture de la variable d'environnement de la clé.
    pub fn new(mode: RunMode, store: Arc<dyn SecretStore>, read_env: fn() -> Option<String>) -> Self {
        Self { mode, store, read_env }
    }

    /// Source de la clé pour le mode courant.
    pub fn source(&self) -> ApiKeySource {
        match self.mode {
            RunMode::Dev => ApiKeySource::Env,
            RunMode::Prod => ApiKeySource::Keyring,
        }
    }

    /// Lit la clé à sa source — la seule fonction qui décide d'où elle vient.
    ///
    /// # Returns
    ///
    /// La clé, `None` si elle est absente, ou `AppError::Config` si le
    /// trousseau est inaccessible.
    fn lookup(&self) -> Result<Option<ApiKey>, AppError> {
        let raw = match self.mode {
            RunMode::Dev => (self.read_env)(),
            RunMode::Prod => self.store.read()?,
        };
        Ok(raw.as_deref().and_then(ApiKey::parse))
    }

    /// Clé à utiliser pour un appel à l'API Claude.
    ///
    /// # Returns
    ///
    /// La clé, ou `AppError::Config` si elle est absente (avec l'endroit où
    /// la renseigner) ou si le trousseau est inaccessible.
    pub fn resolve(&self) -> Result<ApiKey, AppError> {
        self.lookup()?.ok_or_else(|| AppError::Config(self.missing_message()))
    }

    /// État de la clé pour l'écran Paramètres, sans la valeur complète.
    ///
    /// # Returns
    ///
    /// `ApiKeyStatus`, ou `AppError::Config` si le trousseau est inaccessible.
    pub fn status(&self) -> Result<ApiKeyStatus, AppError> {
        let key = self.lookup()?;
        Ok(ApiKeyStatus {
            present: key.is_some(),
            source: self.source(),
            last4: key.map(|k| k.last4()),
        })
    }

    /// Enregistre la clé dans le trousseau (mode PROD uniquement).
    ///
    /// # Arguments
    ///
    /// * `raw` - Clé saisie ; les espaces autour sont retirés.
    ///
    /// # Returns
    ///
    /// L'état de la clé enregistrée, `AppError::Validation` en mode DEV ou
    /// pour une clé vide, ou `AppError::Config` si le trousseau est inaccessible.
    pub fn save(&self, raw: &str) -> Result<ApiKeyStatus, AppError> {
        self.ensure_editable()?;
        let key = ApiKey::parse(raw)
            .ok_or_else(|| AppError::Validation("La clé API Anthropic ne peut pas être vide.".to_string()))?;
        self.store.write(key.expose())?;
        Ok(ApiKeyStatus { present: true, source: self.source(), last4: Some(key.last4()) })
    }

    /// Supprime la clé du trousseau (mode PROD uniquement).
    ///
    /// # Returns
    ///
    /// L'état de la clé (absente), `AppError::Validation` en mode DEV, ou
    /// `AppError::Config` si le trousseau est inaccessible.
    pub fn delete(&self) -> Result<ApiKeyStatus, AppError> {
        self.ensure_editable()?;
        self.store.delete()?;
        Ok(ApiKeyStatus { present: false, source: self.source(), last4: None })
    }

    /// Refuse toute modification en mode DEV, où la clé vient du `.env`.
    fn ensure_editable(&self) -> Result<(), AppError> {
        match self.mode {
            RunMode::Prod => Ok(()),
            RunMode::Dev => Err(AppError::Validation(
                "En mode DEV, la clé API Anthropic se règle dans le fichier .env (ANTHROPIC_API_KEY).".to_string(),
            )),
        }
    }

    /// Message de clé absente, avec l'endroit où la renseigner.
    fn missing_message(&self) -> String {
        match self.mode {
            RunMode::Prod => format!("{MISSING_KEY} Saisis-la dans les Paramètres."),
            RunMode::Dev => format!("{MISSING_KEY} Renseigne ANTHROPIC_API_KEY dans le fichier .env."),
        }
    }
}
