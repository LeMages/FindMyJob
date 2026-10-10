//! Tests de `db::timeline_events`, `db::interviews` et `db::contacts` :
//! événements autorisés, entretiens, changement de statut et historique.

#[path = "shared/seed.rs"]
mod seed;

// Importation des crates externes
use chrono::{TimeZone, Utc};

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::models::{NewInterviewData, NewTimelineEvent};
use findmyjob_lib::db::{candidatures, contacts, interviews, status_history, timeline_events};

fn event(event_type: &str) -> NewTimelineEvent {
    NewTimelineEvent {
        event_type: event_type.into(),
        content: None,
        interview_data: None,
        interview_id: None,
        interview_issue: None,
    }
}

#[tokio::test]
async fn parcours_entretien_complet() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::mark_postule(&pool, cid, None, seed::NOW).await.unwrap();

    // 1. Planifier un entretien : statut « entretien », date stockée au format unique
    let mut planifie = event("entretien_planifie");
    planifie.interview_data = Some(NewInterviewData {
        type_entretien: "RH".into(),
        date_entretien: Some(Utc.with_ymd_and_hms(2026, 10, 20, 14, 0, 0).unwrap()),
        lieu: Some("Visio".into()),
        modalite: Some("Visio".into()),
        contact_id: None,
    });
    timeline_events::add_event(&pool, cid, planifie, "2026-10-11T08:00:00.000Z").await.unwrap();
    assert_eq!(candidatures::fetch_status(&pool, cid).await.unwrap(), "entretien");
    let stocke: String = sqlx::query_scalar("SELECT date_entretien FROM interviews").fetch_one(&pool).await.unwrap();
    assert_eq!(stocke, "2026-10-20T14:00:00.000Z");

    let en_attente = interviews::list_pending_for_candidature(&pool, cid).await.unwrap();
    assert_eq!(en_attente.len(), 1);
    assert_eq!(interviews::list_for_session(&pool, sid).await.unwrap().len(), 1);

    // 2. Événement « entretien passé » disponible tant qu'un entretien attend
    let dispo = timeline_events::available_for_candidature(&pool, cid).await.unwrap();
    assert!(dispo.iter().any(|e| e.event_type == "entretien_passe"));

    let mut passe = event("entretien_passe");
    passe.interview_id = Some(en_attente[0].id);
    passe.interview_issue = Some("Positif".into());
    timeline_events::add_event(&pool, cid, passe, "2026-10-21T08:00:00.000Z").await.unwrap();
    assert!(interviews::list_pending_for_candidature(&pool, cid).await.unwrap().is_empty());

    let historique = status_history::list_for_candidature(&pool, cid).await.unwrap();
    let types: Vec<&str> = historique.iter().map(|h| h.event_type.as_str()).collect();
    assert_eq!(types, ["creation", "candidature_envoyee", "entretien_planifie", "entretien_passe"]);
    assert_eq!(historique[3].interview_id, Some(en_attente[0].id));
}

#[tokio::test]
async fn evenements_refuses() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = seed::candidature(&pool, sid, seed::NOW).await;

    assert!(matches!(
        timeline_events::add_event(&pool, cid, event("relance"), seed::NOW).await,
        Err(AppError::Conflict(_))
    ));
    assert!(matches!(
        timeline_events::add_event(&pool, cid, event("note"), seed::NOW).await,
        Err(AppError::Validation(_))
    ));
    assert!(matches!(
        timeline_events::add_event(&pool, 999, event("note"), seed::NOW).await,
        Err(AppError::NotFound(_))
    ));
    assert_eq!(status_history::list_for_candidature(&pool, cid).await.unwrap().len(), 1, "rien d'écrit");

    let mut note = event("note");
    note.content = Some("  Relancer lundi ".into());
    timeline_events::add_event(&pool, cid, note, seed::NOW).await.unwrap();
    let historique = status_history::list_for_candidature(&pool, cid).await.unwrap();
    assert_eq!(historique[1].content.as_deref(), Some("Relancer lundi"));
    assert_eq!(candidatures::fetch_status(&pool, cid).await.unwrap(), "a_postuler");
}

#[tokio::test]
async fn contacts_vides_par_defaut() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = seed::candidature(&pool, sid, seed::NOW).await;
    assert!(contacts::list_for_candidature(&pool, cid).await.unwrap().is_empty());
}
