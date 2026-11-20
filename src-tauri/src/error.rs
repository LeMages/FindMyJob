//! Type d'erreur applicatif commun à toutes les commandes Tauri.
//!
//! `AppError` uniformise les erreurs remontées au frontend (validation,
//! conflit, ressource introuvable, erreur base de données) et implémente
//! `serde::Serialize` afin de traverser le pont IPC de Tauri.

// Importation des modules internes
use crate::models::ReferenceIssue;
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
    /// Référence importée avant le contrat actuel (sans `contract.json` ou
    /// d'une autre version du contrat) : elle doit être réimportée. Le
    /// message est la phrase française affichée à l'utilisateur.
    OutdatedReference(String),
    /// Référence refusée à l'import : tous les problèmes relevés par la
    /// validation du contrat, codes typés traduits par l'interface
    /// (`{"kind":"InvalidReference","message":[{"code":…}]}`).
    InvalidReference(Vec<ReferenceIssue>),
    /// Modification refusée parce que la session est en pause ou terminée
    /// (lecture seule tant qu'elle n'est pas repassée « active »). Le
    /// message est la phrase française affichée à l'utilisateur.
    SessionLocked(String),
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
            | AppError::OutdatedReference(m)
            | AppError::SessionLocked(m) => write!(f, "{}", m),
            AppError::Scraping(failure) => write!(f, "Échec de l'extraction de l'offre ({failure})"),
            AppError::InvalidReference(issues) => {
                let listed: Vec<String> = issues.iter().map(ToString::to_string).collect();
                write!(f, "Référence refusée : {}", listed.join(" ; "))
            }
        }
    }
}

impl std::error::Error for AppError {}
