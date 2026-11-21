//! Tests des statistiques d'une session : calcul pur sur un historique
//! construit à la main (entonnoir, issues, semaines, comparaisons,
//! entretiens, durées, relances), seuils « trop peu » et état vide, puis
//! lecture en base des versions de référence.

// Importation des crates externes
use chrono::{DateTime, NaiveDate, Utc};

// Importation des modules internes
use crate::shared::seed;
use findmyjob_lib::db::candidatures;
use findmyjob_lib::models::{
    ComparisonRow, FollowUpStats, InterviewTypeCount, MedianDurations, OutcomeCount, SessionStats, StatsCandidature,
    StatsEvent, StatsInput, StatsInterview,
};
use findmyjob_lib::services::session_stats::compute::compute;
use findmyjob_lib::services::session_stats::load::session_stats;

const PLATFORM: &str = "Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)";
const COMPANY_SITE: &str = "Site carrière de l'entreprise (formulaire sur le site)";
const EMAIL: &str = "Mail (candidature directe par email)";

fn at(day: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&format!("{day}T09:00:00Z")).expect("date de test").with_timezone(&Utc)
}

fn cand(id: i64, status: &str, source: &str, cv: Option<i64>, lm: Option<i64>, contact: bool) -> StatsCandidature {
    StatsCandidature {
        id,
        status: status.into(),
        source: source.into(),
        cv_version: cv,
        lm_version: lm,
        has_contact: contact,
    }
}

fn ev(candidature_id: i64, event_type: &str, day: &str) -> StatsEvent {
    StatsEvent { candidature_id, event_type: event_type.into(), changed_at: at(day) }
}

fn itv(candidature_id: i64, kind: Option<&str>, day: &str, issue: &str) -> StatsInterview {
    StatsInterview {
        candidature_id,
        type_entretien: kind.map(str::to_string),
        date_entretien: Some(at(day)),
        issue: issue.into(),
    }
}

/// Session type : 6 candidatures dont 5 envoyées (la 5e jamais envoyée,
/// la 6e reçoit un message avant son envoi, qui n'est pas une réponse).
fn scenario() -> StatsInput {
    let mut events = vec![
        ev(1, "candidature_envoyee", "2026-10-01"),
        ev(1, "message_recu", "2026-10-04"),
        ev(2, "candidature_envoyee", "2026-10-01"),
        ev(2, "refus_recu", "2026-10-11"),
        ev(3, "candidature_envoyee", "2026-10-02"),
        ev(3, "relance", "2026-10-09"),
        ev(3, "appel", "2026-10-10"),
        ev(3, "entretien_planifie", "2026-10-12"),
        ev(3, "offre_recue", "2026-10-20"),
        ev(4, "candidature_envoyee", "2026-10-05"),
        ev(4, "relance", "2026-10-12"),
        ev(4, "ghosting", "2026-10-30"),
        ev(6, "message_recu", "2026-10-05"),
        ev(6, "candidature_envoyee", "2026-10-06"),
    ];
    events.sort_by_key(|e| e.changed_at);
    StatsInput {
        candidatures: vec![
            cand(1, "relance", PLATFORM, Some(1), Some(1), true),
            cand(2, "refus", PLATFORM, Some(2), None, false),
            cand(3, "offre", EMAIL, Some(2), None, true),
            cand(4, "ghosting", PLATFORM, None, None, false),
            cand(5, "a_postuler", PLATFORM, None, None, false),
            cand(6, "postule", COMPANY_SITE, None, None, false),
        ],
        events,
        interviews: vec![
            itv(3, Some("rh"), "2026-10-15", "positif"),
            // Entretien à venir : ni « passé », ni dans les semaines affichées.
            itv(3, Some("technique"), "2026-12-01", "en_attente"),
        ],
    }
}

fn stats_of(input: &StatsInput) -> SessionStats {
    compute(input, at("2026-11-19"), NaiveDate::from_ymd_opt(2026, 11, 19).expect("date"))
}

fn row(key: &str, sent: u32, replied: u32, interviewed: u32) -> ComparisonRow {
    ComparisonRow { key: key.into(), sent, replied, interviewed, low_sample: sent < 5, best: false }
}

#[test]
fn entonnoir_et_premiere_reponse() {
    let s = stats_of(&scenario());
    assert!(s.enough_data);
    assert_eq!((s.sent, s.replied, s.interviewed, s.offers), (5, 3, 1, 1));
    // Premières réponses : 3 j, 10 j et 8 j après l'envoi.
    assert_eq!(s.avg_first_reply_days, Some(7.0));
    assert_eq!(s.sent_this_week, 0);
}

#[test]
fn issues_des_candidatures_envoyees() {
    let s = stats_of(&scenario());
    let outcome = |o: &str, count| OutcomeCount { outcome: o.into(), count };
    assert_eq!(
        s.outcomes,
        [outcome("in_progress", 2), outcome("offer", 1), outcome("refus", 1), outcome("ghosting", 1)]
    );
}

#[test]
fn activite_des_neuf_dernieres_semaines() {
    let s = stats_of(&scenario());
    assert_eq!(s.weeks.len(), 9);
    assert_eq!(s.weeks[0].week_start, NaiveDate::from_ymd_opt(2026, 9, 21).expect("date"));
    assert_eq!(s.weeks[8].week_start, NaiveDate::from_ymd_opt(2026, 11, 16).expect("date"));
    let counts: Vec<(u32, u32)> = s.weeks.iter().map(|w| (w.sent, w.interviews)).collect();
    assert_eq!(counts, [(0, 0), (3, 0), (2, 0), (0, 1), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)]);
}

#[test]
fn comparaisons_par_modele_source_et_contact() {
    let s = stats_of(&scenario());
    let rows = |criterion: &str| &s.comparisons.iter().find(|c| c.criterion == criterion).expect(criterion).rows;
    assert_eq!(rows("cv_template"), &[row("1", 1, 1, 0), row("2", 2, 2, 1), row("unknown", 2, 0, 0)]);
    assert_eq!(rows("lm_template"), &[row("1", 1, 1, 0), row("unknown", 4, 2, 1)]);
    assert_eq!(rows("source"), &[row(PLATFORM, 3, 2, 0), row(EMAIL, 1, 1, 1), row(COMPANY_SITE, 1, 0, 0)]);
    assert_eq!(rows("contact"), &[row("with", 2, 2, 1), row("without", 3, 1, 0)]);
}

#[test]
fn meilleure_ligne_parmi_celles_assez_fournies() {
    // 6 envois par la plateforme (3 réponses), 5 par mail (4 réponses),
    // 1 par le site (1 réponse) : 100 % mais trop peu pour compter.
    let mut input = StatsInput { candidatures: vec![], events: vec![], interviews: vec![] };
    for id in 1..=12 {
        let source = match id { 1..=6 => PLATFORM, 7..=11 => EMAIL, _ => COMPANY_SITE };
        input.candidatures.push(cand(id, "postule", source, None, None, false));
        input.events.push(ev(id, "candidature_envoyee", "2026-10-01"));
        if matches!(id, 1..=3 | 7..=10 | 12) {
            input.events.push(ev(id, "message_recu", "2026-10-03"));
        }
    }
    let s = stats_of(&input);
    let source = &s.comparisons.iter().find(|c| c.criterion == "source").expect("source").rows;
    let flags: Vec<(&str, bool, bool)> = source.iter().map(|r| (r.key.as_str(), r.low_sample, r.best)).collect();
    assert_eq!(flags, [(PLATFORM, false, false), (EMAIL, false, true), (COMPANY_SITE, true, false)]);
}

#[test]
fn entretiens_durees_et_relances() {
    let s = stats_of(&scenario());
    assert_eq!(s.interview_types, [InterviewTypeCount { interview_type: "rh".into(), passed: 1, positive: 1 }]);
    assert_eq!(
        s.median_days,
        MedianDurations { to_offer: Some(18.0), to_rejection: Some(10.0), to_ghosting: Some(25.0) }
    );
    // La relance de la candidature 3 est suivie d'un appel, pas celle de la 4.
    assert_eq!(s.follow_ups, FollowUpStats { sent: 2, answered: 1, avg_days_before: Some(7.0) });
}

#[test]
fn etat_vide_sous_trois_envois() {
    let mut input = scenario();
    input.events.retain(|e| e.candidature_id <= 2);
    let s = stats_of(&input);
    assert_eq!(s.sent, 2);
    assert!(!s.enough_data);
}

#[tokio::test]
async fn lecture_en_base_des_versions_et_des_envois() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let version = seed::reference(&pool, sid, "cv").await;
    let envoyee = seed::candidature(&pool, sid, seed::NOW).await;
    seed::candidature(&pool, sid, seed::NOW).await;
    sqlx::query("UPDATE candidatures SET cv_reference_version_id = $1 WHERE id = $2")
        .bind(version)
        .bind(envoyee)
        .execute(&pool)
        .await
        .expect("version de référence");
    candidatures::mark_postule(&pool, envoyee, None, "2026-10-11T08:00:00.000Z").await.expect("envoi");

    let s = session_stats(&pool, sid).await.expect("statistiques");
    assert_eq!(s.sent, 1);
    assert!(!s.enough_data);
    let cv = &s.comparisons.iter().find(|c| c.criterion == "cv_template").expect("cv").rows;
    assert_eq!(cv.iter().map(|r| (r.key.as_str(), r.sent)).collect::<Vec<_>>(), [("1", 1)]);
}
