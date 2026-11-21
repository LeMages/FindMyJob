//! Tests : fichiers d'une version de référence importée au contrat, et refus
//! des versions importées avant lui. Une version sans `contract.json` (ou
//! d'une autre version du contrat) demande un réimport ; un `contract.json`
//! illisible est signalé comme corrompu, jamais comme obsolète.

// Importation des modules internes
use crate::shared::seed;
use crate::shared::test_state::{self, TestApp};
use findmyjob_lib::db;
use findmyjob_lib::documents::reference_template::{load_reference, ReferenceTemplate};
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{CvData, CONTRACT_FILE};
use findmyjob_lib::storage_paths;

/// Application avec une session et ses références importées.
async fn app_with_references(name: &str) -> (TestApp, i64) {
    let app = test_state::app(name).await;
    let sid = seed::session(&app.state.pool).await;
    test_state::seed_references(&app.state, sid).await;
    (app, sid)
}

/// Charge la référence CV courante d'une session.
async fn load_cv(app: &TestApp, sid: i64) -> Result<ReferenceTemplate<CvData>, AppError> {
    load_reference(&app.state.pool, &app.state.data_dir, sid, "cv", "CV").await
}

/// Dossier de la version courante du CV.
async fn cv_version_dir(app: &TestApp, sid: i64) -> std::path::PathBuf {
    let doc = db::documents::get_current(&app.state.pool, sid, "cv").await.expect("lecture").expect("référence");
    storage_paths::to_absolute(&app.state.data_dir, &storage_paths::version_dir(sid, "cv", doc.current_version_id))
}

#[tokio::test]
async fn import_ecrit_la_reference_son_apercu_et_son_contrat() {
    let (app, sid) = app_with_references("reference-fichiers").await;

    let dir = cv_version_dir(&app, sid).await;
    for file in ["reference.typ", "reference.pdf", CONTRACT_FILE] {
        assert!(dir.join(file).is_file(), "{file} absent de {}", dir.display());
    }
    let reference = load_cv(&app, sid).await.expect("référence au contrat");
    assert_eq!(reference.page_count, 1);
    assert_eq!(reference.contract.example.identity.prenom, "Camille");
    assert!(reference.contract.period.is_some(), "format des périodes du CV");
    for lang in ["fr", "en"] {
        assert!(reference.contract.example_text.get(lang).is_some_and(|t| !t.is_empty()), "rendu {lang}");
    }
    assert_eq!(app.llm.total_calls(), 0, "import sans appel LLM");
}

#[tokio::test]
async fn version_d_avant_le_contrat_a_reimporter() {
    let (app, sid) = app_with_references("reference-avant-contrat").await;
    // Fichiers d'une version templatisée par l'ancien import.
    sqlx::query("UPDATE document_versions SET template_path = 'documents/x/template.typ', example_data_path = 'documents/x/example_data.json'")
        .execute(&app.state.pool)
        .await
        .expect("ancienne version");

    let error = load_cv(&app, sid).await.err().expect("refus attendu");
    assert!(matches!(&error, AppError::OutdatedReference(m) if m.contains("Réimportez-le")), "{error}");
}

#[tokio::test]
async fn autre_version_du_contrat_a_reimporter() {
    let (app, sid) = app_with_references("reference-autre-contrat").await;
    let path = cv_version_dir(&app, sid).await.join(CONTRACT_FILE);
    let mut contract: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("contrat")).expect("JSON");
    contract["contract_version"] = 99.into();
    std::fs::write(&path, contract.to_string()).expect("écriture");

    assert!(matches!(load_cv(&app, sid).await, Err(AppError::OutdatedReference(_))));
}

#[tokio::test]
async fn contrat_illisible_corrompu_pas_obsolete() {
    let (app, sid) = app_with_references("reference-contrat-corrompu").await;
    let path = cv_version_dir(&app, sid).await.join(CONTRACT_FILE);
    std::fs::write(&path, "{ pas du json").expect("écriture");

    let error = load_cv(&app, sid).await.err().expect("refus attendu");
    assert!(matches!(&error, AppError::Validation(m) if m.contains("corrompu")), "{error}");
}
