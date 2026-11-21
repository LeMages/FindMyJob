//! Durées médianes depuis l'envoi (offre, refus, ghosting) et effet des
//! relances. Logique pure.

// Importation des crates externes
use chrono::{DateTime, Utc};

// Importation des modules internes
use crate::models::{FollowUpStats, MedianDurations};
use crate::services::session_stats::funnel::mean;
use crate::services::session_stats::timeline::{days_between, SentCandidature};

/// Durées médianes entre l'envoi et chaque issue.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
///
/// # Returns
///
/// Médianes en jours, `None` pour une issue jamais atteinte.
pub fn median_durations(sent: &[SentCandidature]) -> MedianDurations {
    let median_to = |at: fn(&SentCandidature) -> Option<DateTime<Utc>>| {
        median(sent.iter().filter_map(|s| at(s).map(|t| days_between(s.sent_at, t))).collect())
    };
    MedianDurations {
        to_offer: median_to(|s| s.offer_at),
        to_rejection: median_to(|s| s.rejection_at),
        to_ghosting: median_to(|s| s.ghosting_at),
    }
}

/// Relances envoyées, relances suivies d'une réponse et délai moyen avant
/// la première relance.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
///
/// # Returns
///
/// Statistiques des relances.
pub fn follow_ups(sent: &[SentCandidature]) -> FollowUpStats {
    let total: usize = sent.iter().map(|s| s.follow_ups.len()).sum();
    let answered: usize = sent
        .iter()
        .map(|s| s.follow_ups.iter().filter(|f| s.replies.iter().any(|r| r > f)).count())
        .sum();
    FollowUpStats {
        sent: total as u32,
        answered: answered as u32,
        avg_days_before: mean(sent.iter().filter_map(|s| s.follow_ups.first().map(|f| days_between(s.sent_at, *f)))),
    }
}

/// Médiane d'une liste de valeurs.
///
/// # Arguments
///
/// * `values` - Valeurs, dans un ordre quelconque.
///
/// # Returns
///
/// Médiane (moyenne des deux valeurs centrales pour un nombre pair),
/// `None` pour une liste vide.
pub fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    Some(if values.len().is_multiple_of(2) { (values[mid - 1] + values[mid]) / 2.0 } else { values[mid] })
}
