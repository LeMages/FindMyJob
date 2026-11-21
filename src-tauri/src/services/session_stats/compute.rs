//! Assemble les statistiques d'une session à partir de ses données brutes.
//! Logique pure : l'instant présent est un paramètre.

// Importation des crates externes
use chrono::{DateTime, NaiveDate, Utc};

// Importation des modules internes
use crate::models::{SessionStats, StatsInput};
use crate::services::session_stats::{activity, comparisons, durations, funnel, timeline};

/// En dessous de ce nombre de candidatures envoyées, l'onglet affiche un
/// état vide plutôt que des pourcentages sans signification.
pub const MIN_SENT_FOR_STATS: u32 = 3;

/// Calcule les statistiques d'une session.
///
/// # Arguments
///
/// * `input` - Candidatures, événements et entretiens de la session.
/// * `now` - Instant présent (entretiens passés).
/// * `today` - Date du jour (semaines).
///
/// # Returns
///
/// Statistiques de la session, calculées sur les candidatures envoyées.
pub fn compute(input: &StatsInput, now: DateTime<Utc>, today: NaiveDate) -> SessionStats {
    let sent = timeline::sent_candidatures(input);
    let counts = funnel::funnel_counts(&sent);
    SessionStats {
        enough_data: counts.sent >= MIN_SENT_FOR_STATS,
        min_sent: MIN_SENT_FOR_STATS,
        min_sent_per_row: comparisons::MIN_SENT_PER_ROW,
        sent: counts.sent,
        sent_this_week: activity::sent_this_week(&sent, today),
        replied: counts.replied,
        interviewed: counts.interviewed,
        offers: counts.offers,
        avg_first_reply_days: funnel::avg_first_reply_days(&sent),
        outcomes: funnel::outcomes(&sent),
        weeks: activity::weeks(&sent, &input.interviews, today),
        comparisons: comparisons::comparisons(&sent),
        interview_types: activity::interview_types(&input.interviews, now),
        median_days: durations::median_durations(&sent),
        follow_ups: durations::follow_ups(&sent),
    }
}
