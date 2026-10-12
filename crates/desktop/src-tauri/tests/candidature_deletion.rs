//! Suppression d'une candidature au niveau des services appelés par les
//! commandes : la ligne, son historique, ses entretiens, ses liaisons aux
//! contacts et ses fichiers disparaissent ; les contacts du carnet restent.
//!
//! L'utilisateur ne supprime une candidature qu'« à postuler »
//! (`services::candidatures::delete`) ; une fois postulée, elle ne disparaît
//! qu'avec sa session (`services::sessions::delete`), seul chemin qui peut
//! emporter des entretiens.

#[path = "shared/fake_llm.rs"]
mod fake_llm;
#[path = "shared/fake_secret_store.rs"]
mod fake_secret_store;
#[path = "shared/seed.rs"]
mod seed;
#[path = "shared/test_state.rs"]
mod test_state;

// Importation des modules standard
use std::fs;
use std::path::PathBuf;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::models::{ContactInput, NewInterviewData, NewTimelineEvent};
use findmyjob_lib::app_state::AppState;
use findmyjob_lib::db::{candidatures, contacts, timeline_events};
use findmyjob_lib::services;
use findmyjob_lib::storage_paths;
use test_state::{artefact, count};

/// Types de documents générés pour une candidature.
const DOC_TYPES: [&str; 2] = ["cv", "lettre_motivation"];
/// Extensions des artefacts générés.
const EXTENSIONS: [&str; 3] = ["pdf", "data.json", "typ"];

/// Pose les fichiers générés d'une candidature (artefacts et trace) ;
/// retourne leurs chemins.
fn write_generated_files(state: &AppState, sid: i64, cid: i64) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = DOC_TYPES
        .iter()
        .flat_map(|doc_type| EXTENSIONS.iter().map(move |ext| artefact(state, sid, doc_type, cid, ext)))
        .collect();
    let debug = storage_paths::file_in(&storage_paths::debug_dir(sid, cid), "offer_analysis.json");
    files.push(storage_paths::to_absolute(&state.data_dir, &debug));
    for file in &files {
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"contenu").unwrap();
    }
    files
}

/// Contact du carnet.
fn contact(nom: &str) -> ContactInput {
    ContactInput { nom: nom.into(), role: Some("RH".into()), email: None, telephone: None, notes: None }
}

/// Planifie un entretien avec des interlocuteurs.
async fn plan_interview(pool: &SqlitePool, cid: i64, contact_ids: Vec<i64>) {
    let event = NewTimelineEvent {
        event_type: "entretien_planifie".into(),
        content: None,
        interview_data: Some(NewInterviewData {
            type_entretien: "rh".into(),
            date_entretien: Some("2026-10-20T09:00:00Z".parse().unwrap()),
            lieu: Some("Visio".into()),
            modalite: None,
            contact_ids,
        }),
        interview_id: None,
        interview_issue: None,
    };
    timeline_events::add_event(pool, cid, event, seed::NOW).await.expect("entretien planifié");
}

#[tokio::test]
async fn supprimer_une_candidature_a_postuler() {
    let app = test_state::app("suppression-a-postuler").await;
    let pool = &app.state.pool;
    let sid = seed::session(pool).await;
    let cid = seed::candidature(pool, sid, seed::NOW).await;
    let autre = seed::candidature(pool, sid, seed::NOW).await;

    // Un contact créé depuis la candidature, un autre repris du carnet et
    // partagé avec une autre candidature.
    contacts::create(pool, &contact("Claire Dupont"), Some(cid), seed::NOW).await.unwrap();
    let partage = contacts::create(pool, &contact("Paul Durand"), Some(autre), seed::NOW).await.unwrap().id;
    contacts::link(pool, cid, partage, seed::NOW).await.unwrap();
    let files = write_generated_files(&app.state, sid, cid);
    let autres_files = write_generated_files(&app.state, sid, autre);

    services::candidatures::delete(&app.state, cid).await.expect("suppression");

    assert!(matches!(candidatures::fetch_detail(pool, cid).await, Err(AppError::NotFound(_))));
    let history: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM status_history WHERE candidature_id = $1")
        .bind(cid)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(history, 0);
    assert!(contacts::list_for_candidature(pool, cid).await.unwrap().is_empty());
    // Les contacts restent dans le carnet ; l'autre candidature garde le sien.
    assert_eq!(contacts::list_all(pool).await.unwrap().len(), 2);
    assert_eq!(contacts::list_for_candidature(pool, autre).await.unwrap().len(), 1);
    for file in &files {
        assert!(!file.exists(), "fichier restant : {}", file.display());
    }
    assert!(autres_files.iter().all(|f| f.exists()), "les fichiers d'une autre candidature sont touchés");

    assert!(matches!(services::candidatures::delete(&app.state, cid).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn candidature_postulee_supprimee_avec_ses_entretiens() {
    let app = test_state::app("suppression-entretiens").await;
    let pool = &app.state.pool;
    let sid = seed::session(pool).await;
    let cid = seed::candidature(pool, sid, seed::NOW).await;
    let recruteuse = contacts::create(pool, &contact("Claire Dupont"), Some(cid), seed::NOW).await.unwrap().id;
    candidatures::mark_postule(pool, cid, None, seed::NOW).await.unwrap();
    plan_interview(pool, cid, vec![recruteuse]).await;
    let files = write_generated_files(&app.state, sid, cid);
    assert_eq!(count(pool, "interviews").await, 1);
    assert_eq!(count(pool, "interview_contacts").await, 1);

    // ------------------------------------------------------------------
    // 1. Une candidature postulée ne se supprime pas seule
    // ------------------------------------------------------------------
    assert!(matches!(services::candidatures::delete(&app.state, cid).await, Err(AppError::Conflict(_))));
    assert_eq!(count(pool, "interviews").await, 1);
    assert!(files.iter().all(|f| f.exists()));

    // ------------------------------------------------------------------
    // 2. Elle disparaît avec sa session, entretiens et liaisons compris
    // ------------------------------------------------------------------
    services::sessions::delete(&app.state, sid).await.expect("suppression de la session");

    for table in ["candidatures", "status_history", "interviews", "interview_contacts", "candidature_contacts"] {
        assert_eq!(count(pool, table).await, 0, "table {table} non vidée");
    }
    assert_eq!(contacts::list_all(pool).await.unwrap().len(), 1, "le contact reste dans le carnet");
    for file in &files {
        assert!(!file.exists(), "fichier restant : {}", file.display());
    }
    let session_dir = storage_paths::to_absolute(&app.state.data_dir, &storage_paths::session_dir(sid));
    assert!(!session_dir.exists());
}
