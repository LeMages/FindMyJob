//! Type d'événement de la timeline `status_history`.
//!
//! C'est la seule source de vérité applicative pour les valeurs autorisées
//! et les transitions de statut qu'elles impliquent — la colonne
//! `status_history.event_type` n'a volontairement pas de `CHECK` SQL
//! (comme `candidatures.type`, validé via `TYPES_CONTRAT_VALIDES`), la
//! validation vit ici.

// Importation des modules internes
use crate::error::AppError;

/// Type d'un événement de la timeline de suivi d'une candidature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Creation,
    CandidatureEnvoyee,
    Relance,
    EntretienPlanifie,
    EntretienPasse,
    OffreRecue,
    RefusRecu,
    Ghosting,
    Desistement,
    MessageEnvoye,
    MessageRecu,
    Appel,
    Note,
}

impl EventType {
    /// Représentation SQLite (`snake_case`) de l'événement, utilisée pour
    /// écrire et relire la colonne `status_history.event_type`.
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Creation => "creation",
            EventType::CandidatureEnvoyee => "candidature_envoyee",
            EventType::Relance => "relance",
            EventType::EntretienPlanifie => "entretien_planifie",
            EventType::EntretienPasse => "entretien_passe",
            EventType::OffreRecue => "offre_recue",
            EventType::RefusRecu => "refus_recu",
            EventType::Ghosting => "ghosting",
            EventType::Desistement => "desistement",
            EventType::MessageEnvoye => "message_envoye",
            EventType::MessageRecu => "message_recu",
            EventType::Appel => "appel",
            EventType::Note => "note",
        }
    }

    /// Statut cible qu'un événement fait prendre à la candidature.
    ///
    /// # Returns
    ///
    /// `Some(statut)` si l'événement change le statut de la candidature,
    /// `None` sinon (message, appel, note, entretien passé...).
    pub fn changes_status(&self) -> Option<&'static str> {
        match self {
            EventType::Relance => Some("relance"),
            EventType::EntretienPlanifie => Some("entretien"),
            EventType::OffreRecue => Some("offre"),
            EventType::RefusRecu => Some("refus"),
            EventType::Ghosting => Some("ghosting"),
            EventType::Desistement => Some("abandon"),
            EventType::Creation
            | EventType::CandidatureEnvoyee
            | EventType::EntretienPasse
            | EventType::MessageEnvoye
            | EventType::MessageRecu
            | EventType::Appel
            | EventType::Note => None,
        }
    }
}

impl TryFrom<&str> for EventType {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "creation" => Ok(EventType::Creation),
            "candidature_envoyee" => Ok(EventType::CandidatureEnvoyee),
            "relance" => Ok(EventType::Relance),
            "entretien_planifie" => Ok(EventType::EntretienPlanifie),
            "entretien_passe" => Ok(EventType::EntretienPasse),
            "offre_recue" => Ok(EventType::OffreRecue),
            "refus_recu" => Ok(EventType::RefusRecu),
            "ghosting" => Ok(EventType::Ghosting),
            "desistement" => Ok(EventType::Desistement),
            "message_envoye" => Ok(EventType::MessageEnvoye),
            "message_recu" => Ok(EventType::MessageRecu),
            "appel" => Ok(EventType::Appel),
            "note" => Ok(EventType::Note),
            other => Err(AppError::Validation(format!("Type d'événement inconnu : {other}."))),
        }
    }
}
