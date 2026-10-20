//! Tests de `db::sessions`, `db::document_versions` et `db::documents` :
//! statistiques, cascades à la suppression, pointeur de référence courante,
//! identifiants jamais réattribués et liaison des paramètres réutilisés.

// Importation des modules internes
use crate::shared::seed;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::UpdateSessionInput;
use findmyjob_lib::db::transaction::begin_write;
use findmyjob_lib::db::{candidatures, document_versions, documents, sessions, settings};

async fn count(pool: &sqlx::SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}")).fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn sessions_statistiques_et_mise_a_jour() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    seed::candidature(&pool, sid, seed::NOW).await;
    sqlx::query("UPDATE candidatures SET status = 'refus' WHERE id = $1").bind(c1).execute(&pool).await.unwrap();

    let liste = sessions::list_all(&pool).await.unwrap();
    assert_eq!(liste[0].candidature_count, 1, "les candidatures refusées ne comptent pas");
    assert!(liste[0].has_accepted, "un refus compte comme issue");

    let input = UpdateSessionInput { nom: " Alternance ".into(), statut: "pause".into() };
    let maj = sessions::update(&pool, sid, &input).await.unwrap();
    assert_eq!((maj.nom.as_str(), maj.statut.as_str(), maj.candidature_count), ("Alternance", "pause", 1));

    let invalide = UpdateSessionInput { nom: "X".into(), statut: "archivee".into() };
    assert!(matches!(sessions::update(&pool, sid, &invalide).await, Err(AppError::Validation(_))));
    assert!(matches!(sessions::update(&pool, 999, &input).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn supprimer_une_session_supprime_tout_en_cascade() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    seed::reference(&pool, sid, "cv").await;
    seed::candidature(&pool, sid, seed::NOW).await;

    sessions::delete(&pool, sid).await.unwrap();
    for table in ["candidatures", "status_history", "document_versions", "documents"] {
        assert_eq!(count(&pool, table).await, 0, "table {table} vidée par cascade");
    }
    assert!(matches!(sessions::delete(&pool, sid).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn le_pointeur_suit_la_version_courante() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    assert!(documents::get_current(&pool, sid, "cv").await.unwrap().is_none());
    assert!(!documents::status_for_session(&pool, sid).await.unwrap().all_ready);

    let v1 = seed::reference(&pool, sid, "cv").await;
    let v2 = seed::reference(&pool, sid, "cv").await;
    seed::reference(&pool, sid, "lettre_motivation").await;

    let courant = documents::get_current(&pool, sid, "cv").await.unwrap().unwrap();
    assert_eq!(courant.current_version_id, v2);
    assert_eq!(courant.file_path, "documents/x/reference.typ");
    assert!(courant.template().is_some());
    assert!(documents::status_for_session(&pool, sid).await.unwrap().all_ready);

    let versions = document_versions::list_versions(&pool, sid, "cv").await.unwrap();
    let numeros: Vec<(i32, bool)> = versions.iter().map(|v| (v.version, v.is_current)).collect();
    assert_eq!(numeros, [(2, true), (1, false)], "numéros calculés par $2/$3 réutilisés");

    // Restauration : le pointeur revient sur la version 1.
    let mut tx = begin_write(&pool).await.unwrap();
    let record = document_versions::get_version_record(&mut tx, sid, "cv", 1).await.unwrap().unwrap();
    assert_eq!(record.id, v1);
    documents::set_current(&mut tx, sid, "cv", record.id, "2026-10-10T09:00:00.000Z").await.unwrap();
    tx.commit().await.unwrap();
    let courant = documents::get_current(&pool, sid, "cv").await.unwrap().unwrap();
    assert_eq!(courant.current_version_id, v1);
    assert_eq!(courant.updated_at.to_rfc3339(), "2026-10-10T09:00:00+00:00");
}

#[tokio::test]
async fn un_identifiant_de_version_n_est_jamais_reattribue() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let v1 = seed::reference(&pool, sid, "cv").await;
    let v2 = seed::reference(&pool, sid, "cv").await;
    sqlx::query("DELETE FROM document_versions WHERE id = $1").bind(v2).execute(&pool).await.unwrap();
    assert!(documents::get_current(&pool, sid, "cv").await.unwrap().is_none(), "pointeur supprimé en cascade");

    let v3 = seed::reference(&pool, sid, "cv").await;
    assert!(v3 > v2 && v2 > v1, "AUTOINCREMENT : {v1} < {v2} < {v3}");
}

#[tokio::test]
async fn supprimer_une_version_detache_les_candidatures() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cv = seed::reference(&pool, sid, "cv").await;
    let lm = seed::reference(&pool, sid, "lettre_motivation").await;
    let cid = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::set_documents(&pool, cid, ("a/cv.pdf", "a/lm.pdf"), (cv, lm), &[]).await.unwrap();
    assert_eq!(candidatures::fetch_detail(&pool, cid).await.unwrap().cv_reference_version, Some(1));

    sqlx::query("DELETE FROM document_versions WHERE id = $1").bind(cv).execute(&pool).await.unwrap();
    let detail = candidatures::fetch_detail(&pool, cid).await.unwrap();
    assert_eq!(detail.cv_reference_version, None, "ON DELETE SET NULL");
    assert_eq!(detail.lm_reference_version, Some(1));
}

#[tokio::test]
async fn reglages_upsert_par_cle() {
    let pool = seed::pool().await;
    assert_eq!(settings::resolve_theme(&pool).await.unwrap(), settings::DEFAULT_THEME);
    settings::set(&pool, settings::KEY_THEME, "sombre").await.unwrap();
    settings::set(&pool, settings::KEY_THEME, "clair").await.unwrap();
    assert_eq!(settings::resolve_theme(&pool).await.unwrap(), "clair");
    assert_eq!(settings::resolve_llm_model(&pool, "modele-defaut").await.unwrap(), "modele-defaut");
}
