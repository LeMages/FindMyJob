//! DTO représentant une entrée de la timeline de suivi d'une candidature.

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatusHistoryView {
    pub id: i64,
    pub candidature_id: i64,
    /// Type d'événement (voir `EventType`) — détermine l'icône et le libellé
    /// affichés côté frontend.
    pub event_type: String,
    pub ancien_status: Option<String>,
    /// `None` si l'événement ne change pas le statut (message, appel, note...).
    pub nouveau_status: Option<String>,
    /// Message (mail ou message de candidature) ayant accompagné le
    /// changement, `None` s'il n'y en a pas.
    pub content: Option<String>,
    /// Entretien concerné, uniquement pour `entretien_planifie` /
    /// `entretien_passe`.
    pub interview_id: Option<i64>,
    pub changed_at: DateTime<Utc>,
}
