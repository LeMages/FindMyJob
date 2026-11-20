//! Application de test avec les références balisées (`references/tagged/`)
//! templatisées par le LLM factice, et lecture de ce qu'une génération
//! produit : `data.json` et rendu Typst réel avec le template de la version.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::shared::fake_llm::{read_reference, Reply, Step};
use crate::shared::seed;
use crate::shared::test_state::{self, artefact, candidature_input, wait_generation, TestApp};
use findmyjob_lib::db;
use findmyjob_lib::documents;
use findmyjob_lib::documents::typst::compile_template;
use findmyjob_lib::models::GenerationStatus;
use findmyjob_lib::services::candidatures;
use findmyjob_lib::storage_paths;

/// Application avec un profil et les deux références balisées.
///
/// # Arguments
///
/// * `name` - Nom unique du test (dossier temporaire).
///
/// # Returns
///
/// L'application et l'identifiant de sa session.
pub async fn app_with_tagged_references(name: &str) -> (TestApp, i64) {
    let app = test_state::app(name).await;
    test_state::seed_profile(&app.state.pool).await;
    let sid = seed::session(&app.state.pool).await;
    let llm = &app.llm;
    llm.respond_with(Step::ExampleCv, Reply::Files(&["references/tagged/cv_balise_example_data.json"]));
    llm.respond_with(Step::TemplateCv, Reply::Files(&["references/tagged/cv_balise_template.typ"]));
    llm.respond_with(Step::ExampleLm, Reply::Files(&["references/tagged/lm_balisee_example_data.json"]));
    llm.respond_with(Step::TemplateLm, Reply::Files(&["references/tagged/lm_balisee_template.typ"]));
    let cv = read_reference("tagged/cv_balise.typ");
    documents::save_reference(&app.state, sid, "cv", "cv.typ".into(), cv).await.expect("référence CV");
    let lm = read_reference("tagged/lm_balisee.typ");
    documents::save_reference(&app.state, sid, "lettre_motivation", "lettre.typ".into(), lm)
        .await
        .expect("référence lettre");
    (app, sid)
}

/// Lance une génération et attend sa fin.
///
/// # Arguments
///
/// * `app` - Application de test.
/// * `sid` - Session de la candidature créée.
///
/// # Returns
///
/// `(identifiant de la candidature, erreur éventuelle, génération terminée)`.
pub async fn generate(app: &TestApp, sid: i64) -> (i64, Option<String>, bool) {
    let view = candidatures::create(&app.state, &candidature_input(sid, "Data engineering intern.", None))
        .await
        .expect("création");
    let progress = wait_generation(&app.state, view.id).await;
    (view.id, progress.error, matches!(progress.status, GenerationStatus::Completed))
}

/// Lit le `data.json` généré d'un document.
///
/// # Arguments
///
/// * `app` - Application de test.
/// * `sid`, `doc_type`, `cid` - Session, type de document et candidature.
///
/// # Returns
///
/// Les données JSON.
pub fn data(app: &TestApp, sid: i64, doc_type: &str, cid: i64) -> Value {
    let raw = std::fs::read_to_string(artefact(&app.state, sid, doc_type, cid, "data.json")).expect("data.json");
    serde_json::from_str(&raw).expect("data.json valide")
}

/// Dossier de la version courante d'une référence.
///
/// # Arguments
///
/// * `app` - Application de test.
/// * `sid`, `doc_type` - Session et type de document.
///
/// # Returns
///
/// Chemin absolu du dossier de la version.
pub async fn version_dir(app: &TestApp, sid: i64, doc_type: &str) -> PathBuf {
    let doc = db::documents::get_current(&app.state.pool, sid, doc_type).await.expect("lecture").expect("référence");
    storage_paths::to_absolute(&app.state.data_dir, &storage_paths::version_dir(sid, doc_type, doc.current_version_id))
}

/// Texte rendu d'un document généré : son `data.json` compilé avec le
/// template de la version courante.
///
/// # Arguments
///
/// * `app` - Application de test.
/// * `sid`, `doc_type` - Session et type de document.
/// * `generated` - `data.json` du document.
///
/// # Returns
///
/// Le document compilé.
pub async fn render(app: &TestApp, sid: i64, doc_type: &str, generated: &Value) -> documents::typst::CompiledDocument {
    let dir = version_dir(app, sid, doc_type).await;
    let template = std::fs::read_to_string(dir.join("template.typ")).expect("template");
    let raw = serde_json::to_vec(generated).expect("data.json");
    compile_template(template, raw, dir, app.state.fonts_dir.clone()).expect("compilation")
}
