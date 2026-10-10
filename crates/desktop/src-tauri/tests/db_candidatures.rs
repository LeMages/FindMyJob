//! Tests de `db::candidatures`, `db::status_history` et `db::joins` :
//! horodatage unique par transaction, ordre chronologique des chaînes,
//! règles de statut et générations interrompues.

#[path = "shared/seed.rs"]
mod seed;

// Importation des crates externes
use chrono::{TimeZone, Utc};

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_lib::clock;
use findmyjob_lib::db::{candidatures, joins, status_history};

#[tokio::test]
async fn creation_et_historique_partagent_le_meme_horodatage() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let now = clock::now_utc();
    let view = candidatures::insert(&pool, sid, &seed::fields(None), "Offre", &now).await.unwrap();
    assert_eq!(view.derniere_maj_statut, view.created_at);

    let historique = status_history::list_for_candidature(&pool, view.id).await.unwrap();
    assert_eq!(historique.len(), 1);
    assert_eq!(historique[0].event_type, "creation");
    assert_eq!(historique[0].changed_at, view.created_at, "un seul horodatage par transaction");

    let liste = candidatures::list_for_session(&pool, sid).await.unwrap();
    assert_eq!(liste[0].derniere_maj_statut, liste[0].created_at);
}

#[tokio::test]
async fn le_tri_des_chaines_est_le_tri_chronologique() {
    // Format fixe : la représentation d'une valeur reprise (microsecondes
    // tronquées) et celle d'une valeur sans fraction ont la même longueur.
    let reprise = Utc.with_ymd_and_hms(2026, 9, 21, 15, 25, 15).unwrap() + chrono::Duration::microseconds(400_366);
    let ronde = Utc.with_ymd_and_hms(2026, 9, 21, 15, 25, 16).unwrap();
    assert_eq!(clock::format_timestamp(reprise), "2026-09-21T15:25:15.400Z");
    assert_eq!(clock::format_timestamp(ronde), "2026-09-21T15:25:16.000Z");

    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let ancienne = seed::candidature(&pool, sid, &clock::format_timestamp(reprise)).await;
    let recente = seed::candidature(&pool, sid, &clock::format_timestamp(ronde)).await;
    candidatures::mark_postule(&pool, ancienne, Some("Bonjour"), "2026-10-01T09:00:00.000Z").await.unwrap();

    let liste = candidatures::list_for_session(&pool, sid).await.unwrap();
    assert_eq!(liste.iter().map(|c| c.id).collect::<Vec<_>>(), [recente, ancienne]);
    let maj = liste.iter().find(|c| c.id == ancienne).unwrap().derniere_maj_statut;
    assert_eq!(maj.to_rfc3339(), "2026-10-01T09:00:00+00:00", "MAX(changed_at) sur des chaînes");
}

#[tokio::test]
async fn regles_de_statut_a_postuler() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = seed::candidature(&pool, sid, seed::NOW).await;

    let mut champs = seed::fields(Some("https://exemple.fr/offre"));
    champs.notes = Some("note".into());
    let detail = candidatures::update(&pool, cid, &champs).await.unwrap();
    assert_eq!(detail.url_offre.as_deref(), Some("https://exemple.fr/offre"));
    assert_eq!(detail.description_offre.as_deref(), Some("Offre"));

    candidatures::mark_postule(&pool, cid, None, seed::NOW).await.unwrap();
    assert_eq!(candidatures::fetch_status(&pool, cid).await.unwrap(), "postule");
    assert!(matches!(candidatures::update(&pool, cid, &champs).await, Err(AppError::Conflict(_))));
    assert!(matches!(candidatures::mark_postule(&pool, cid, None, seed::NOW).await, Err(AppError::Conflict(_))));
    assert!(matches!(candidatures::delete_a_postuler(&pool, cid).await, Err(AppError::Conflict(_))));
    assert!(matches!(candidatures::delete_a_postuler(&pool, 999).await, Err(AppError::NotFound(_))));

    let autre = seed::candidature(&pool, sid, seed::NOW).await;
    assert_eq!(candidatures::delete_a_postuler(&pool, autre).await.unwrap(), sid);
}

#[tokio::test]
async fn generations_interrompues_et_suppression() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cv = seed::reference(&pool, sid, "cv").await;
    let lm = seed::reference(&pool, sid, "lettre_motivation").await;
    let interrompue = seed::candidature(&pool, sid, seed::NOW).await;
    let terminee = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::set_documents(&pool, terminee, ("a/cv.pdf", "a/lm.pdf"), (cv, lm)).await.unwrap();

    assert_eq!(candidatures::list_interrupted_generations(&pool).await.unwrap(), [(interrompue, sid)]);
    assert_eq!(candidatures::delete(&pool, interrompue).await.unwrap(), Some(sid));
    assert_eq!(candidatures::delete(&pool, interrompue).await.unwrap(), None);
    assert!(status_history::list_for_candidature(&pool, interrompue).await.unwrap().is_empty());
    assert!(matches!(
        candidatures::set_documents(&pool, interrompue, ("a", "b"), (cv, lm)).await,
        Err(AppError::NotFound(_))
    ));
}

#[tokio::test]
async fn la_base_refuse_un_event_type_inconnu() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = seed::candidature(&pool, sid, seed::NOW).await;
    let result = sqlx::query(
        "INSERT INTO status_history (candidature_id, event_type, changed_at) VALUES ($1, 'inconnu', $2)",
    )
    .bind(cid)
    .bind(seed::NOW)
    .execute(&pool)
    .await;
    assert!(result.is_err(), "CHECK sur event_type");
}

#[tokio::test]
async fn doublons_par_url_toutes_sessions() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    candidatures::insert(&pool, sid, &seed::fields(Some("https://a.fr/1")), "Offre", seed::NOW).await.unwrap();
    candidatures::insert(&pool, sid, &seed::fields(Some("   ")), "Offre", seed::NOW).await.unwrap();
    seed::candidature(&pool, sid, seed::NOW).await;

    let lignes = joins::list_candidatures_with_offer_url(&pool).await.unwrap();
    assert_eq!(lignes.len(), 1, "URL vide ou blanche ignorée");
    assert_eq!(lignes[0].0, "https://a.fr/1");
    assert_eq!(lignes[0].1.session_nom, "Stage M2");
}
