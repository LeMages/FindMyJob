//! Type d'erreur applicatif commun à toutes les commandes Tauri.
//!
//! `AppError` uniformise les erreurs remontées au frontend (validation,
//! conflit, ressource introuvable, erreur base de données) et implémente
//! `serde::Serialize` afin de traverser le pont IPC de Tauri.

// Importation des modules internes
use crate::scraping::failure::ScrapingFailure;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    Validation(String),
    Conflict(String),
    NotFound(String),
    Config(String),
    Llm(String),
    /// Échec typé de la récupération ou de l'extraction d'une offre, sérialisé
    /// en code (`{"kind":"Scraping","message":"access_denied"}`).
    Scraping(ScrapingFailure),
    /// Référence templatisée par une version précédente, dont le template ne
    /// compile plus avec le schéma actuel : elle doit être réimportée. Le
    /// message est la phrase française affichée à l'utilisateur.
    OutdatedReference(String),
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
            | AppError::OutdatedReference(m) => write!(f, "{}", m),
            AppError::Scraping(failure) => write!(f, "Échec de l'extraction de l'offre ({failure})"),
        }
    }
}

impl std::error::Error for AppError {}
