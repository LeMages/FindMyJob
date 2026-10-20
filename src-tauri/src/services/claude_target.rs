//! Résolution de la cible des appels à l'API Claude (clé et modèle) : le
//! point de passage de tout code qui appelle Claude.

// Importation des modules standard
use std::sync::Arc;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::services::api_key::ApiKeyAccess;
use crate::error::AppError;
use crate::llm::client::DEFAULT_MODEL;
use crate::llm::{ClaudeTarget, LlmCaller};

/// Ce qu'il faut pour résoudre la cible d'un appel à Claude. Permet de ne
/// résoudre la clé qu'au moment où un appel est réellement nécessaire.
///
/// # Attributes
///
/// * `pool` - Pool de connexions SQLite (modèle configuré).
/// * `api_key` - Accès à la clé API selon le mode d'exécution.
/// * `llm` - Exécutant des appels (l'API Anthropic dans l'application).
#[derive(Clone, Copy)]
pub struct ClaudeTargetSource<'a> {
    pub pool: &'a SqlitePool,
    pub api_key: &'a ApiKeyAccess,
    pub llm: &'a Arc<dyn LlmCaller>,
}

impl ClaudeTargetSource<'_> {
    /// Résout le modèle configuré et la clé API.
    ///
    /// # Returns
    ///
    /// `ClaudeTarget`, ou l'erreur de lecture du modèle ou de la clé.
    pub async fn resolve(&self) -> Result<ClaudeTarget, AppError> {
        let model = db::settings::resolve_llm_model(self.pool, DEFAULT_MODEL).await?;
        // Le trousseau du système est une API bloquante (D-Bus sous Linux).
        let access = self.api_key.clone();
        let api_key = tokio::task::spawn_blocking(move || access.resolve())
            .await
            .map_err(|e| AppError::Config(format!("Lecture de la clé API interrompue : {e}")))??;
        Ok(ClaudeTarget { api_key, model, caller: Arc::clone(self.llm), recorder: None })
    }
}
