//! Type d'erreur applicatif commun à toutes les commandes Tauri.
//!
//! `AppError` uniformise les erreurs remontées au frontend (validation,
//! conflit, ressource introuvable, erreur base de données) et implémente
//! `serde::Serialize` afin de traverser le pont IPC de Tauri.

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    Validation(String),
    Conflict(String),
    NotFound(String),
    Config(String),
    Llm(String),
    Scraping(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(m)
            | AppError::Validation(m)
            | AppError::Conflict(m)
            | AppError::NotFound(m)
            | AppError::Config(m)
            | AppError::Llm(m)
            | AppError::Scraping(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for AppError {}
