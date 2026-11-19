//! Activité hebdomadaire (envois et entretiens) et entretiens passés par
//! type. Logique pure.

// Importation des crates externes
use chrono::{DateTime, Datelike, Days, NaiveDate, Utc};

// Importation des modules internes
use crate::models::{InterviewTypeCount, StatsInterview, WeekActivity};
use crate::services::session_stats::timeline::SentCandidature;

/// Nombre de semaines affichées, semaine en cours comprise.
pub const WEEK_COUNT: u64 = 9;

/// Types d'entretien dans l'ordre d'affichage.
const INTERVIEW_TYPES: [&str; 5] = ["rh", "technique", "manager", "final", "autre"];

/// Lundi de la semaine d'une date.
///
/// # Arguments
///
/// * `date` - Date quelconque.
///
/// # Returns
///
/// Lundi de la même semaine (la date elle-même si c'est un lundi).
pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Days::new(u64::from(date.weekday().num_days_from_monday()))
}

/// Compte les envois et les entretiens des `WEEK_COUNT` dernières semaines.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
/// * `interviews` - Entretiens de la session.
/// * `today` - Date du jour.
///
/// # Returns
///
/// Une entrée par semaine, de la plus ancienne à la semaine en cours.
pub fn weeks(sent: &[SentCandidature], interviews: &[StatsInterview], today: NaiveDate) -> Vec<WeekActivity> {
    let current = week_start(today);
    let in_week = |start: NaiveDate, at: DateTime<Utc>| week_start(at.date_naive()) == start;
    (0..WEEK_COUNT)
        .rev()
        .map(|back| {
            let start = current - Days::new(back * 7);
            WeekActivity {
                week_start: start,
                sent: sent.iter().filter(|s| in_week(start, s.sent_at)).count() as u32,
                interviews: interviews.iter().filter(|i| i.date_entretien.is_some_and(|d| in_week(start, d))).count() as u32,
            }
        })
        .collect()
}

/// Compte les envois de la semaine en cours.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
/// * `today` - Date du jour.
///
/// # Returns
///
/// Nombre de candidatures envoyées depuis lundi.
pub fn sent_this_week(sent: &[SentCandidature], today: NaiveDate) -> u32 {
    let current = week_start(today);
    sent.iter().filter(|s| s.sent_at.date_naive() >= current).count() as u32
}

/// Compte les entretiens passés par type et leurs issues positives. Un
/// entretien est passé quand sa date est atteinte ; sans type, il compte
/// comme « autre ».
///
/// # Arguments
///
/// * `interviews` - Entretiens de la session.
/// * `now` - Instant présent.
///
/// # Returns
///
/// Une entrée par type non vide, dans l'ordre de `INTERVIEW_TYPES`.
pub fn interview_types(interviews: &[StatsInterview], now: DateTime<Utc>) -> Vec<InterviewTypeCount> {
    let passed: Vec<&StatsInterview> = interviews.iter().filter(|i| i.date_entretien.is_some_and(|d| d <= now)).collect();
    INTERVIEW_TYPES
        .iter()
        .map(|kind| {
            let of_kind: Vec<&&StatsInterview> =
                passed.iter().filter(|i| i.type_entretien.as_deref().unwrap_or("autre") == *kind).collect();
            InterviewTypeCount {
                interview_type: (*kind).to_string(),
                passed: of_kind.len() as u32,
                positive: of_kind.iter().filter(|i| i.issue == "positif").count() as u32,
            }
        })
        .filter(|c| c.passed > 0)
        .collect()
}
