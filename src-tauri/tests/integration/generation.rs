//! Génération du CV et de la lettre au niveau des services appelés par les
//! commandes (`documents::save_reference`, `services::candidatures::create`) :
//! profil en base, références de test templatisées, réponses LLM figées,
//! compilation Typst réelle dans un dossier temporaire.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::shared::{fake_llm, seed, test_state};
use crate::shared::no_dashes::{assert_no_dash_in_data, assert_no_dash_in_text};
use fake_llm::{Reply, Step};
use findmyjob_lib::llm::client::TEMPLATIZE_MODEL;
use findmyjob_lib::docgen::lint::lint_prose;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{GenerationProgress, GenerationStatus, ProseField};
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

/// Champs de prose d'un `data.json` (titre, résumé, objet, paragraphes).
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
    // Les références de test passent la validation dès le premier essai,
    // avec le modèle imposé à la templatisation.
    for step in [Step::ExampleCv, Step::TemplateCv, Step::ExampleLm, Step::TemplateLm] {
        assert_eq!(app.llm.models_for(step), [TEMPLATIZE_MODEL], "{step:?}");
    }

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
    // Entrées : lignes exactes des descriptions du profil, jamais réécrites.
    assert_eq!(cv["experiences"][0]["bullets"], serde_json::json!(["Pipelines SQL pour le reporting."]));
    assert_eq!(cv["projects"][0]["titre"], "Tableau de bord associatif");
    assert_eq!(cv["projects"][0]["bullets"], serde_json::json!(["Suivi des adhésions en Python."]));
    assert_eq!(cv["projects"][0]["pour_qui"], "Association Escalade Lyon");
    assert_eq!(cv["projects"][0]["stack"], "Python");
    assert_eq!(cv["projects"][0]["url"], "");
    assert_eq!(cv["formations"][0]["details"], serde_json::json!(["Spécialité données"]));
    assert_eq!(cv["langues"][0]["nom"], "Anglais");
    assert_eq!(cv["skills_groups"][0]["skills"], serde_json::json!(["SQL", "Python"]));
    assert_eq!(lm["identity"]["nom"], "Martin");
    assert_eq!(lm["identity"]["title"], cv["identity"]["title"], "même sous-titre sur les deux documents");
    assert_ne!(cv["identity"]["title"], "", "sous-titre rédigé pour l'offre");
    assert_eq!(lm["recipient"]["company"], "Lumen Santé");
    assert_eq!(lm["paragraphs"].as_array().map(Vec::len), Some(3));
    assert!(!cv_raw.contains("Exemple de"), "aucune entrée de la référence : {cv_raw}");
    assert_eq!(app.llm.calls_for(Step::ProfileTranslation), 0, "même langue : pas de traduction");
    assert_eq!(app.llm.calls_for(Step::Matching), 1);

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
async fn aucun_tiret_interdit_dans_les_documents_generes() {
    let (app, sid) = ready_app("generation-sans-tiret").await;
    // « -- » et « --- » écrits par le LLM, « — » dans des noms officiels du
    // profil : la base n'est pas modifiée, seul le rendu l'est.
    app.llm.respond_with(Step::LmContent, Reply::Fixture("lm_content_double_tiret.json"));
    let pool = &app.state.pool;
    sqlx::query("UPDATE experiences SET poste = 'Fondateur — Plateforme de covoiturage' WHERE id = 1")
        .execute(pool)
        .await
        .expect("poste avec tiret");
    sqlx::query("UPDATE projets SET titre = 'ALGN — Autonomous Legal Graph Navigator' WHERE id = 1")
        .execute(pool)
        .await
        .expect("titre avec tiret");

    let (cid, progress) = generate(&app, sid).await;
    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);

    // ------------------------------------------------------------------
    // 1. data.json : aucune chaîne avec un tiret interdit
    // ------------------------------------------------------------------
    let (cv_raw, cv) = read_data(&app.state, sid, "cv", cid);
    let (lm_raw, lm) = read_data(&app.state, sid, "lettre_motivation", cid);
    assert_no_dash_in_data("CV", &cv);
    assert_no_dash_in_data("lettre", &lm);
    assert_eq!(cv["experiences"][0]["poste"], "Fondateur - Plateforme de covoiturage");
    assert_eq!(cv["projects"][0]["titre"], "ALGN - Autonomous Legal Graph Navigator");
    assert_eq!(lm["subject"], "Candidature, stage de data engineer");
    let paragraph = lm["paragraphs"][1].as_str().expect("paragraphe");
    assert!(paragraph.contains("pipelines SQL, un travail proche"), "{paragraph}");
    let profile = db::profile::build_profile(pool).await.expect("profil");
    assert_eq!(profile.experiences[0].poste, "Fondateur — Plateforme de covoiturage", "base inchangée");

    // ------------------------------------------------------------------
    // 2. Texte rendu : aucun tiret interdit non plus
    // ------------------------------------------------------------------
    for (doc_type, raw) in [("cv", cv_raw), ("lettre_motivation", lm_raw)] {
        let doc = db::documents::get_current(pool, sid, doc_type).await.expect("lecture").expect("référence");
        let dir = storage_paths::to_absolute(
            &app.state.data_dir,
            &storage_paths::version_dir(sid, doc_type, doc.current_version_id),
        );
        let template = std::fs::read_to_string(dir.join("template.typ")).expect("template");
        let rendered = compile_template(template, raw.into_bytes(), dir, app.state.fonts_dir.clone()).expect("compilation");
        assert_no_dash_in_text(doc_type, &rendered.text);
    }
}

#[tokio::test]
async fn prose_vide_relance_puis_echoue() {
    let (app, sid) = ready_app("generation-prose-vide").await;
    app.llm.respond_with(Step::CvContent, Reply::Fixture("cv_content_vide.json"));

    let view = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await.expect("création");
    let progress = wait_generation(&app.state, view.id).await;

    assert!(matches!(progress.status, GenerationStatus::Failed));
    let error = progress.error.expect("message d'échec");
    // Message lisible pour l'utilisateur ; le détail technique est tracé.
    assert!(error.starts_with("La génération des documents a échoué"), "erreur : {error}");
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
    assert!(progress.error.is_some_and(|e| e.contains("indisponible ou surchargée")));
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

/// Lance une génération et attend sa fin.
async fn generate(app: &TestApp, sid: i64) -> (i64, GenerationProgress) {
    let view = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await.expect("création");
    let progress = wait_generation(&app.state, view.id).await;
    (view.id, progress)
}

#[tokio::test]
async fn matching_invalide_relance_puis_echoue() {
    let (app, sid) = ready_app("generation-matching-invalide").await;
    app.llm.respond_with(Step::Matching, Reply::Fixture("matching_id_inconnu.json"));

    let (cid, progress) = generate(&app, sid).await;

    assert!(matches!(progress.status, GenerationStatus::Failed));
    let error = progress.error.expect("message d'échec");
    assert!(error.starts_with("La génération des documents a échoué"), "erreur : {error}");
    assert_eq!(app.llm.calls_for(Step::Matching), 3);
    assert_eq!(app.llm.calls_for(Step::CvContent), 0, "aucune rédaction sans projets valides");
    assert_eq!(count(&app.state.pool, "candidatures").await, 0);
    assert_no_generated_files(&app.state, sid, cid);
}

#[tokio::test]
async fn matching_invalide_puis_corrige() {
    let (app, sid) = ready_app("generation-matching-corrige").await;
    app.llm.respond_with(Step::Matching, Reply::Sequence(&["matching_id_inconnu.json", "matching.json"]));

    let (cid, progress) = generate(&app, sid).await;

    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);
    assert_eq!(app.llm.calls_for(Step::Matching), 2);
    let retry = &app.llm.messages_for(Step::Matching)[1];
    assert!(retry.contains("PROBLÈMES") && retry.contains("l'id 99"), "retour de l'essai précédent : {retry}");
    let (_, cv) = read_data(&app.state, sid, "cv", cid);
    assert_eq!(cv["projects"][0]["titre"], "Tableau de bord associatif");
}

/// Application dont l'offre est analysée en anglais (référence en français).
async fn english_app(name: &str) -> (TestApp, i64) {
    let (app, sid) = ready_app(name).await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_en.json"));
    (app, sid)
}

#[tokio::test]
async fn offre_dans_une_autre_langue_traduit_les_entrees() {
    let (app, sid) = english_app("generation-traduction").await;

    let (cid, progress) = generate(&app, sid).await;

    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);
    assert_eq!(app.llm.calls_for(Step::ProfileTranslation), 1, "un seul appel pour toutes les entrées");
    let sent = &app.llm.messages_for(Step::ProfileTranslation)[0];
    assert!(sent.contains("Pipelines SQL pour le reporting."), "{sent}");
    for official in ["Alternant data engineer", "Lumen Data", "Tableau de bord associatif", "Master informatique", "Université Lyon 1"] {
        assert!(!sent.contains(official), "{official} n'est jamais envoyé à la traduction : {sent}");
    }

    let (_, cv) = read_data(&app.state, sid, "cv", cid);
    assert_eq!(cv["lang"], "en");
    assert_eq!(cv["experiences"][0]["bullets"], serde_json::json!(["SQL pipelines for reporting."]));
    assert_eq!(cv["projects"][0]["bullets"], serde_json::json!(["Membership tracking in Python."]));
    assert_eq!(cv["projects"][0]["pour_qui"], "Escalade Lyon club");
    assert_eq!(cv["formations"][0]["details"], serde_json::json!(["Data major"]));
    // Noms officiels : ceux du profil, tels quels.
    assert_eq!(cv["experiences"][0]["poste"], "Alternant data engineer");
    assert_eq!(cv["experiences"][0]["entreprise"], "Lumen Data");
    assert_eq!(cv["experiences"][0]["lieu"], "Lyon");
    assert_eq!(cv["projects"][0]["titre"], "Tableau de bord associatif");
    assert_eq!(cv["projects"][0]["stack"], "Python");
    assert_eq!(cv["formations"][0]["titre"], "Master informatique");
    assert_eq!(cv["formations"][0]["ecole"], "Université Lyon 1");
}

#[tokio::test]
async fn traduction_invalide_relance_puis_echoue() {
    let (app, sid) = english_app("generation-traduction-invalide").await;
    app.llm.respond_with(Step::ProfileTranslation, Reply::Fixture("profile_translation_lignes.json"));

    let (cid, progress) = generate(&app, sid).await;

    assert!(matches!(progress.status, GenerationStatus::Failed));
    let error = progress.error.expect("message d'échec");
    assert!(error.starts_with("La génération des documents a échoué"), "erreur : {error}");
    assert_eq!(app.llm.calls_for(Step::ProfileTranslation), 3);
    assert_eq!(count(&app.state.pool, "candidatures").await, 0);
    assert_no_generated_files(&app.state, sid, cid);
}

#[tokio::test]
async fn traduction_invalide_puis_corrigee() {
    let (app, sid) = english_app("generation-traduction-corrigee").await;
    app.llm.respond_with(
        Step::ProfileTranslation,
        Reply::Sequence(&["profile_translation_lignes.json", "profile_translation.json"]),
    );

    let (cid, progress) = generate(&app, sid).await;

    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);
    assert_eq!(app.llm.calls_for(Step::ProfileTranslation), 2);
    let retry = &app.llm.messages_for(Step::ProfileTranslation)[1];
    assert!(retry.contains("PROBLÈMES") && retry.contains("attendu exactement 1"), "{retry}");
    let (_, cv) = read_data(&app.state, sid, "cv", cid);
    assert_eq!(cv["experiences"][0]["bullets"], serde_json::json!(["SQL pipelines for reporting."]));
}
