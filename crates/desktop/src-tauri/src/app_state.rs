//! État partagé de l'application, construit au démarrage et géré par Tauri.
//!
//! L'initialisation (dossiers, base, migrations) peut échouer : l'état géré
//! est alors un `AppContext` en échec, et chaque commande renvoie l'erreur
//! d'initialisation, que l'interface affiche sur son écran de démarrage.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::progress::ProgressStore;
use findmyjob_core::error::AppError;

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
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub data_dir: PathBuf,
    pub fonts_dir: PathBuf,
    pub capture_profile_dir: PathBuf,
    pub progress: ProgressStore,
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
