//! Type d'erreur applicatif commun à toutes les commandes Tauri.
//!
//! `AppError` uniformise les erreurs remontées au frontend (validation,
//! conflit, ressource introuvable, erreur base de données) et implémente
//! `serde::Serialize` afin de traverser le pont IPC de Tauri.

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    Validation(String),
    Conflict(String),
    NotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(m)
            | AppError::Validation(m)
            | AppError::Conflict(m)
            | AppError::NotFound(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e.to_string())
    }
}
