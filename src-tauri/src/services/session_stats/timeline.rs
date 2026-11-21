//! Faits datés de chaque candidature, déduits de son historique : envoi,
//! première réponse, offre, refus, ghosting, relances, entretiens. Base de
//! tous les calculs de statistiques. Logique pure.

// Importation des modules standard
use std::collections::{HashMap, HashSet};

// Importation des crates externes
use chrono::{DateTime, Utc};

// Importation des modules internes
use crate::models::{StatsCandidature, StatsEvent, StatsInput};

/// Événements comptés comme une réponse du recruteur. Le ghosting n'en est
/// pas une : c'est justement l'absence de réponse.
pub const RESPONSE_EVENTS: [&str; 5] = ["message_recu", "appel", "entretien_planifie", "offre_recue", "refus_recu"];

/// Statuts d'une candidature qui a reçu une offre (acceptée ou non).
const OFFER_STATUSES: [&str; 3] = ["offre", "accepte", "offre_refusee"];

/// Faits datés d'une candidature envoyée.
pub struct SentCandidature<'a> {
    pub candidature: &'a StatsCandidature,
    pub sent_at: DateTime<Utc>,
    /// Première réponse après l'envoi.
    pub first_reply_at: Option<DateTime<Utc>>,
    /// Dates des réponses après l'envoi, en ordre croissant.
    pub replies: Vec<DateTime<Utc>>,
    pub offer_at: Option<DateTime<Utc>>,
    pub rejection_at: Option<DateTime<Utc>>,
    pub ghosting_at: Option<DateTime<Utc>>,
    /// Relances après l'envoi, en ordre croissant.
    pub follow_ups: Vec<DateTime<Utc>>,
    pub has_offer: bool,
    pub interviewed: bool,
}

/// Construit les faits datés des candidatures envoyées. Une candidature
/// jamais envoyée (sans `candidature_envoyee`) est ignorée ; l'envoi retenu
/// est le plus récent, comme la colonne « Candidaté » du tableau.
///
/// # Arguments
///
/// * `input` - Données brutes de la session (événements triés par date).
///
/// # Returns
///
/// Une entrée par candidature envoyée, dans l'ordre des candidatures.
pub fn sent_candidatures(input: &StatsInput) -> Vec<SentCandidature<'_>> {
    let mut events_by_id: HashMap<i64, Vec<&StatsEvent>> = HashMap::new();
    for event in &input.events {
        events_by_id.entry(event.candidature_id).or_default().push(event);
    }
    let with_interview: HashSet<i64> = input.interviews.iter().map(|i| i.candidature_id).collect();

    input
        .candidatures
        .iter()
        .filter_map(|c| {
            let events = events_by_id.get(&c.id).map(Vec::as_slice).unwrap_or_default();
            build(c, events, with_interview.contains(&c.id))
        })
        .collect()
}

/// Faits datés d'une candidature, `None` si elle n'a pas été envoyée.
fn build<'a>(c: &'a StatsCandidature, events: &[&StatsEvent], has_interview: bool) -> Option<SentCandidature<'a>> {
    let sent_at = events.iter().filter(|e| e.event_type == "candidature_envoyee").map(|e| e.changed_at).max()?;
    // Seuls les événements postérieurs à l'envoi comptent : une note ou un
    // message reçu avant l'envoi n'est pas une réponse à la candidature.
    let after: Vec<&StatsEvent> = events.iter().copied().filter(|e| e.changed_at >= sent_at).collect();
    let first_of = |kind: &str| after.iter().find(|e| e.event_type == kind).map(|e| e.changed_at);
    let replies: Vec<DateTime<Utc>> = after
        .iter()
        .filter(|e| RESPONSE_EVENTS.contains(&e.event_type.as_str()))
        .map(|e| e.changed_at)
        .collect();
    let offer_at = first_of("offre_recue");

    Some(SentCandidature {
        candidature: c,
        sent_at,
        first_reply_at: replies.first().copied(),
        offer_at,
        rejection_at: first_of("refus_recu"),
        ghosting_at: first_of("ghosting"),
        follow_ups: after.iter().filter(|e| e.event_type == "relance").map(|e| e.changed_at).collect(),
        has_offer: offer_at.is_some() || OFFER_STATUSES.contains(&c.status.as_str()),
        interviewed: has_interview || first_of("entretien_planifie").is_some() || first_of("entretien_passe").is_some(),
        replies,
    })
}

/// Durée en jours (fractionnaire) entre deux instants.
///
/// # Arguments
///
/// * `from` - Instant de départ.
/// * `to` - Instant d'arrivée.
///
/// # Returns
///
/// Nombre de jours, négatif si `to` précède `from`.
pub fn days_between(from: DateTime<Utc>, to: DateTime<Utc>) -> f64 {
    (to - from).num_milliseconds() as f64 / 86_400_000.0
}
