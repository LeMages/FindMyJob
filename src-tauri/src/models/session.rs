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
    /// Symbole des listes des descriptions dans les CV générés : `puce` (•)
    /// ou `tiret` (-).
    pub format_liste: String,
    /// Candidatures non clôturées (tout sauf refus, offre déclinée, abandon, ghosting).
    pub candidature_count: i64,
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

/// Format des listes des CV d'une session (modifié seul, sans toucher au
/// reste de la session).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSessionListFormatInput {
    /// `puce` ou `tiret`.
    pub format_liste: String,
}
