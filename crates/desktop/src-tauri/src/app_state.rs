//! État partagé de l'application, construit au démarrage et géré par Tauri.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::progress::ProgressStore;

/// Ressources partagées par les commandes et les tâches de fond. Clonable à
/// coût constant (pool et store reposent sur des `Arc`), pour être déplacé
/// dans une tâche de génération.
///
/// # Attributes
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application (base, documents, polices).
/// * `fonts_dir` - Dossier des polices importées (`<data_dir>/polices`).
/// * `progress` - Progression en mémoire des générations de documents.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub data_dir: PathBuf,
    pub fonts_dir: PathBuf,
    pub progress: ProgressStore,
}
