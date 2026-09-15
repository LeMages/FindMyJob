//! DTOs pour la timeline de suivi d'une candidature : liste des événements
//! proposables pour le statut courant, et entrée soumise pour en ajouter un.

use chrono::{DateTime, Utc};

/// Événement proposable à l'utilisateur pour le statut courant d'une
/// candidature (voir `db::timeline_events::available_events`).
#[derive(serde::Serialize)]
pub struct AvailableEvent {
    pub event_type: String,
    pub label: String,
    pub changes_status: bool,
    pub target_status: Option<String>,
}

/// Données de l'entretien à créer, fournies avec un événement
/// `entretien_planifie`.
#[derive(serde::Deserialize)]
pub struct NewInterviewData {
    #[serde(rename = "type")]
    pub type_entretien: String,
    pub date_entretien: Option<DateTime<Utc>>,
    pub lieu: Option<String>,
    pub modalite: Option<String>,
    pub contact_id: Option<i64>,
}

/// Entrée soumise par le frontend pour ajouter un événement à la timeline
/// d'une candidature. Les champs `interview_*` ne sont renseignés que pour
/// les événements `entretien_planifie` / `entretien_passe`.
#[derive(serde::Deserialize)]
pub struct NewTimelineEvent {
    pub event_type: String,
    pub content: Option<String>,
    pub interview_data: Option<NewInterviewData>,
    pub interview_id: Option<i64>,
    pub interview_issue: Option<String>,
}
