//! Génération du CV et de la lettre au niveau des services appelés par les
//! commandes (`documents::save_reference`, `services::candidatures::create`) :
//! profil en base, références de test templatisées, réponses LLM figées,
//! compilation Typst réelle dans un dossier temporaire.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::shared::{fake_llm, seed, test_state};
use fake_llm::{Reply, Step};
use findmyjob_lib::docgen::lint::lint_prose;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{GenerationStatus, ProseField};
use findmyjob_lib::app_state::AppState;
use findmyjob_lib::db;
use findmyjob_lib::documents::typst::compile_template;
use findmyjob_lib::services::candidatures;
use findmyjob_lib::storage_paths;
use test_state::{artefact, candidature_input, count, wait_generation, TestApp};

/// Texte d'offre transmis à la génération.
const OFFRE: &str = "Stage data engineer : construire des pipelines SQL et Python pour nos équipes santé.";

/// Application avec profil, session et références prêtes ; retourne l'id de session.
async fn ready_app(name: &str) -> (TestApp, i64) {
    let app = test_state::app(name).await;
    test_state::seed_profile(&app.state.pool).await;
    let sid = seed::session(&app.state.pool).await;
    test_state::seed_references(&app.state, sid).await;
    (app, sid)
}

/// Lit un `data.json` généré.
fn read_data(state: &AppState, sid: i64, doc_type: &str, cid: i64) -> (String, Value) {
    let raw = std::fs::read_to_string(artefact(state, sid, doc_type, cid, "data.json")).expect("data.json généré");
    let value = serde_json::from_str(&raw).expect("data.json valide");
    (raw, value)
}

/// Champs de prose d'un `data.json` (titre, résumé, bullets, objet, paragraphes).
fn prose_fields(data: &Value) -> Vec<ProseField> {
    let mut fields = Vec::new();
    let mut push = |path: String, value: &Value| {
        if let Some(text) = value.as_str() {
            fields.push(ProseField { path, text: text.to_string() });
        }
    };
    push("identity.title".to_string(), &data["identity"]["title"]);
    for key in ["summary", "subject"] {
        push(key.to_string(), &data[key]);
    }
    for (i, p) in data["paragraphs"].as_array().into_iter().flatten().enumerate() {
        push(format!("paragraphs[{i}]"), p);
    }
    for section in ["experiences", "projects"] {
        for (i, entry) in data[section].as_array().into_iter().flatten().enumerate() {
            for (j, b) in entry["bullets"].as_array().into_iter().flatten().enumerate() {
                push(format!("{section}[{i}].bullets[{j}]"), b);
            }
        }
    }
    fields
}

/// Vérifie qu'aucun fichier généré ne reste pour une candidature.
fn assert_no_generated_files(state: &AppState, sid: i64, cid: i64) {
    for doc_type in ["cv", "lettre_motivation"] {
        for extension in ["pdf", "data.json", "typ"] {
            let path = artefact(state, sid, doc_type, cid, extension);
            assert!(!path.exists(), "fichier partiel restant : {}", path.display());
        }
    }
    let debug = storage_paths::to_absolute(&state.data_dir, &storage_paths::debug_dir(sid, cid));
    assert!(!debug.exists(), "trace restante : {}", debug.display());
}

#[tokio::test]
async fn generation_remplit_data_json_depuis_le_profil() {
    let (app, sid) = ready_app("generation-ok").await;
    // Les références de test passent la validation dès le premier essai.
    assert_eq!(app.llm.calls_for(Step::TemplatizeCv), 1);
    assert_eq!(app.llm.calls_for(Step::TemplatizeLm), 1);

    let view = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await.expect("création");
    let progress = wait_generation(&app.state, view.id).await;
    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);

    // ------------------------------------------------------------------
    // 1. data.json : faits du profil ; ce que le profil n'a pas (ville et
    //    code postal, vidés par `seed_profile`) vient du document
    // ------------------------------------------------------------------
    let (cv_raw, cv) = read_data(&app.state, sid, "cv", view.id);
    let (_, lm) = read_data(&app.state, sid, "lettre_motivation", view.id);
    for data in [&cv, &lm] {
        assert_eq!(data["identity"]["ville"], "Exempleville", "profil vide : ville du document");
        assert_eq!(data["identity"]["zip_code"], "00000", "profil vide : code postal du document");
    }
    assert_eq!(cv["identity"]["prenom"], "Alex");
    assert_eq!(cv["identity"]["nom"], "Martin");
    assert_eq!(cv["identity"]["email"], "alex@example.com");
    assert_eq!(cv["experiences"][0]["poste"], "Alternant data engineer");
    assert_eq!(cv["experiences"][0]["entreprise"], "Lumen Data");
    assert_eq!(cv["formations"][0]["titre"], "Master informatique");
    assert_eq!(cv["langues"][0]["nom"], "Anglais");
    assert_eq!(cv["skills_groups"][0]["skills"], serde_json::json!(["SQL", "Python"]));
    assert_eq!(lm["identity"]["nom"], "Martin");
    assert_eq!(lm["identity"]["title"], cv["identity"]["title"], "même sous-titre sur les deux documents");
    assert_ne!(cv["identity"]["title"], "", "sous-titre rédigé pour l'offre");
    assert_eq!(lm["recipient"]["company"], "Lumen Santé");
    assert_eq!(lm["paragraphs"].as_array().map(Vec::len), Some(3));

    // ------------------------------------------------------------------
    // 2. Prose : ni caractère ni formule interdits (le tiret cadratin de
    //    la réponse figée est remplacé mécaniquement)
    // ------------------------------------------------------------------
    for data in [&cv, &lm] {
        let violations = lint_prose(&prose_fields(data), "");
        assert!(violations.is_empty(), "prose non conforme : {:?}", violations.iter().map(|v| &v.message).collect::<Vec<_>>());
    }

    // ------------------------------------------------------------------
    // 3. PDF compilés, CV sur une page, candidature rattachée
    // ------------------------------------------------------------------
    for doc_type in ["cv", "lettre_motivation"] {
        let pdf = std::fs::read(artefact(&app.state, sid, doc_type, view.id, "pdf")).expect("PDF généré");
        assert!(pdf.starts_with(b"%PDF"), "{doc_type} : PDF invalide");
    }
    let detail = db::candidatures::fetch_detail(&app.state.pool, view.id).await.unwrap();
    assert!(detail.cv_path.is_some() && detail.lettre_motivation_path.is_some());
    let version = i64::from(detail.cv_reference_version.expect("version de référence"));
    let version_dir = storage_paths::to_absolute(&app.state.data_dir, &storage_paths::version_dir(sid, "cv", version));
    let template = std::fs::read_to_string(version_dir.join("template.typ")).expect("template enregistré");
    let rendered = compile_template(template, cv_raw.into_bytes(), version_dir, app.state.fonts_dir.clone())
        .expect("recompilation du CV");
    assert_eq!(rendered.page_count, 1);
    assert!(rendered.text.contains("ALEX MARTIN"), "texte du CV : {}", rendered.text);
}

#[tokio::test]
async fn prose_vide_relance_puis_echoue() {
    let (app, sid) = ready_app("generation-prose-vide").await;
    app.llm.respond_with(Step::CvContent, Reply::Fixture("cv_content_vide.json"));

    let view = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await.expect("création");
    let progress = wait_generation(&app.state, view.id).await;

    assert!(matches!(progress.status, GenerationStatus::Failed));
    let error = progress.error.expect("message d'échec");
    assert!(error.contains("reste invalide après 3 essais"), "erreur : {error}");
    // Un essai puis deux relances avec le retour de la revue.
    assert_eq!(app.llm.calls_for(Step::CvContent), 3);
    assert_eq!(count(&app.state.pool, "candidatures").await, 0);
    assert_no_generated_files(&app.state, sid, view.id);
}

#[tokio::test]
async fn echec_de_generation_supprime_candidature_et_fichiers() {
    let (app, sid) = ready_app("generation-echec").await;
    app.llm.respond_with(Step::LmContent, Reply::Error("API Claude erreur 529: surchargée"));

    let view = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await.expect("création");
    let progress = wait_generation(&app.state, view.id).await;

    assert!(matches!(progress.status, GenerationStatus::Failed));
    assert!(progress.error.is_some_and(|e| e.contains("529")));
    assert!(matches!(
        db::candidatures::fetch_detail(&app.state.pool, view.id).await,
        Err(AppError::NotFound(_))
    ));
    assert_eq!(count(&app.state.pool, "status_history").await, 0);
    assert_no_generated_files(&app.state, sid, view.id);
    // Les références de la session ne sont pas touchées.
    assert_eq!(count(&app.state.pool, "document_versions").await, 2);
}

#[tokio::test]
async fn creation_refusee_sans_references() {
    let app = test_state::app("generation-sans-reference").await;
    test_state::seed_profile(&app.state.pool).await;
    let sid = seed::session(&app.state.pool).await;

    let result = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await;

    match result {
        Err(AppError::Validation(message)) => assert!(message.starts_with("Les documents de référence"), "{message}"),
        other => panic!("validation attendue, obtenu {:?}", other.map(|v| v.id)),
    }
    assert_eq!(count(&app.state.pool, "candidatures").await, 0);
    assert_eq!(app.llm.total_calls(), 0);
}
