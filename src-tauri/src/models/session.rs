//! DTOs liés aux sessions de candidature.

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionView {
    pub id: i64,
    pub nom: String,
    pub statut: String,
    /// Ville de l'expéditeur (`None` tant qu'elle n'est pas renseignée).
    pub ville: Option<String>,
    /// Code postal de l'expéditeur (facultatif).
    pub zip_code: Option<String>,
    pub candidature_count: i64,
    pub has_accepted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSessionInput {
    pub nom: String,
    /// Ville de l'expéditeur, facultative à la création.
    #[serde(default)]
    pub ville: Option<String>,
    /// Code postal de l'expéditeur, facultatif.
    #[serde(default)]
    pub zip_code: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSessionInput {
    pub nom: String,
    pub statut: String,
}

/// Localisation de l'expéditeur d'une session (modifiée seule, sans toucher
/// au nom ni au statut).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSessionLocationInput {
    #[serde(default)]
    pub ville: Option<String>,
    #[serde(default)]
    pub zip_code: Option<String>,
}
