//! État partagé de l'application, construit au démarrage et géré par Tauri.
//!
//! L'initialisation (dossiers, base, migrations) peut échouer : l'état géré
//! est alors un `AppContext` en échec, et chaque commande renvoie l'erreur
//! d'initialisation, que l'interface affiche sur son écran de démarrage.

// Importation des modules standard
use std::path::PathBuf;
use std::sync::Arc;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::progress::ProgressStore;
use crate::services::api_key::ApiKeyAccess;
use crate::services::claude_target::ClaudeTargetSource;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::LlmCaller;

/// Ressources partagées par les commandes et les tâches de fond. Clonable à
/// coût constant (pool et store reposent sur des `Arc`), pour être déplacé
/// dans une tâche de génération.
///
/// # Attributes
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application (base, documents, polices).
/// * `fonts_dir` - Dossier des polices importées (`<data_dir>/polices`).
/// * `capture_profile_dir` - Profil WebView de la fenêtre de capture (selon le mode d'exécution).
/// * `progress` - Progression en mémoire des générations de documents.
/// * `api_key` - Accès à la clé API Anthropic selon le mode d'exécution.
/// * `llm` - Exécutant des appels à Claude (`AnthropicCaller` dans l'application).
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub data_dir: PathBuf,
    pub fonts_dir: PathBuf,
    pub capture_profile_dir: PathBuf,
    pub progress: ProgressStore,
    pub api_key: ApiKeyAccess,
    pub llm: Arc<dyn LlmCaller>,
}

impl AppState {
    /// Source de la cible des appels à Claude (modèle configuré et clé API).
    ///
    /// # Returns
    ///
    /// `ClaudeTargetSource` adossée au pool, à l'accès à la clé et à
    /// l'exécutant des appels de l'état.
    pub fn claude(&self) -> ClaudeTargetSource<'_> {
        ClaudeTargetSource { pool: &self.pool, api_key: &self.api_key, llm: &self.llm }
    }
}

/// État géré par Tauri : l'état de l'application, ou la cause de l'échec
/// de son initialisation.
pub struct AppContext(Result<AppState, String>);

impl AppContext {
    /// Contexte d'une initialisation réussie.
    ///
    /// # Arguments
    ///
    /// * `state` - État construit au démarrage.
    pub fn ready(state: AppState) -> Self {
        Self(Ok(state))
    }

    /// Contexte d'une initialisation échouée.
    ///
    /// # Arguments
    ///
    /// * `cause` - Message d'erreur, affiché par l'interface.
    pub fn failed(cause: String) -> Self {
        Self(Err(cause))
    }

    /// Accès à l'état de l'application.
    ///
    /// # Returns
    ///
    /// L'état, ou `AppError::Config` portant la cause de l'échec
    /// d'initialisation.
    pub fn state(&self) -> Result<&AppState, AppError> {
        self.0
            .as_ref()
            .map_err(|cause| AppError::Config(format!("Impossible d'initialiser la base locale : {cause}")))
    }
}
