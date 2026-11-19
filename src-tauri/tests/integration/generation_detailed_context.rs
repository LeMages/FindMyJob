//! Tests : contexte détaillé des expériences et des projets dans la
//! génération. Il n'atteint jamais le CV (données, template, texte rendu,
//! prompts du CV et de la traduction) ; le matching ne reçoit que celui des
//! projets ; la lettre reçoit les deux, et la recopie d'un passage relance
//! la rédaction puis, si elle persiste, laisse un avertissement.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::shared::fake_llm::{fixtures_dir, Reply, Step};
use crate::shared::seed;
use crate::shared::test_state::{self, artefact, candidature_input, wait_generation, TestApp};
use findmyjob_lib::db;
use findmyjob_lib::documents::typst::compile_template;
use findmyjob_lib::llm::writing_loop::MAX_WRITING_ATTEMPTS;
use findmyjob_lib::models::GenerationStatus;
use findmyjob_lib::services::candidatures;
use findmyjob_lib::storage_paths;

/// Texte d'offre transmis à la génération.
const OFFRE: &str = "Stage data engineer : construire des pipelines SQL et Python pour nos équipes santé.";
/// Marqueur du contexte de l'expérience 1.
const MARQUEUR_EXPERIENCE: &str = "CTX-EXPERIENCE-9b21";
/// Marqueur du contexte du projet 1.
const MARQUEUR_PROJET: &str = "CTX-PROJET-7f3a";
/// Contexte de l'expérience 1 ; `lm_content_recopie_contexte.json` en
/// recopie un passage.
const CONTEXTE_EXPERIENCE: &str = "CTX-EXPERIENCE-9b21 J'ai repris seul la maintenance des pipelines SQL de l'équipe reporting pendant trois mois d'absence de mon tuteur.";
/// Contexte du projet 1.
const CONTEXTE_PROJET: &str = "CTX-PROJET-7f3a Le club cherchait à remplacer un tableur partagé par une application simple pour suivre ses adhérents.";
/// Avertissement attendu quand la recopie persiste.
const AVERTISSEMENT_RECOPIE: &str = "La lettre reprend mot pour mot un passage de votre contexte détaillé.";

/// Renseigne le contexte détaillé de l'expérience 1 et du projet 1.
async fn seed_contexts(pool: &SqlitePool) {
    for (table, contexte) in [("experiences", CONTEXTE_EXPERIENCE), ("projets", CONTEXTE_PROJET)] {
        sqlx::query(&format!("UPDATE {table} SET contexte = $1 WHERE id = 1"))
            .bind(contexte)
            .execute(pool)
            .await
            .expect("contexte détaillé");
    }
}

/// Application avec profil (contextes renseignés), session et références.
async fn app_with_contexts(name: &str) -> (TestApp, i64) {
    let app = test_state::app(name).await;
    test_state::seed_profile(&app.state.pool).await;
    seed_contexts(&app.state.pool).await;
    let sid = seed::session(&app.state.pool).await;
    test_state::seed_references(&app.state, sid).await;
    (app, sid)
}

/// Lance une génération, attend sa fin et vérifie qu'elle a réussi.
async fn generate(app: &TestApp, sid: i64) -> i64 {
    let view = candidatures::create(&app.state, &candidature_input(sid, OFFRE, None)).await.expect("création");
    let progress = wait_generation(&app.state, view.id).await;
    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);
    view.id
}

/// Vérifie qu'aucun marqueur n'apparaît dans un texte.
fn assert_no_marker(what: &str, text: &str) {
    for marker in [MARQUEUR_EXPERIENCE, MARQUEUR_PROJET] {
        assert!(!text.contains(marker), "{what} : {marker} ne doit pas apparaître\n{text}");
    }
}

/// Template `.typ` du CV (celui que la génération compile, sans le
/// modifier) et texte rendu du CV généré avec ce template.
async fn rendered_cv(app: &TestApp, sid: i64, data_json: String) -> (String, String) {
    let doc = db::documents::get_current(&app.state.pool, sid, "cv").await.expect("lecture").expect("référence");
    let dir = storage_paths::to_absolute(&app.state.data_dir, &storage_paths::version_dir(sid, "cv", doc.current_version_id));
    let template = std::fs::read_to_string(dir.join("template.typ")).expect("template");
    let rendered = compile_template(template.clone(), data_json.into_bytes(), dir, app.state.fonts_dir.clone()).expect("compilation");
    (template, rendered.text)
}

/// Avertissements enregistrés sur la fiche d'une candidature.
async fn warnings(app: &TestApp, cid: i64) -> Vec<String> {
    db::candidatures::fetch_detail(&app.state.pool, cid).await.expect("fiche").generation_warnings
}

#[tokio::test]
async fn contexte_jamais_dans_le_cv_ni_ses_prompts() {
    let (app, sid) = app_with_contexts("contexte-jamais-dans-le-cv").await;

    let cid = generate(&app, sid).await;

    // ------------------------------------------------------------------
    // 1. Fichiers du CV : data.json, template .typ compilé, texte rendu
    // ------------------------------------------------------------------
    let data_json = std::fs::read_to_string(artefact(&app.state, sid, "cv", cid, "data.json")).expect("data.json du CV");
    assert_no_marker("data.json du CV", &data_json);
    let (typ, text) = rendered_cv(&app, sid, data_json).await;
    assert_no_marker(".typ du CV", &typ);
    assert_no_marker("texte rendu du CV", &text);
    // Rien ne recopie mécaniquement le contexte dans la lettre non plus.
    let lm_json = std::fs::read_to_string(artefact(&app.state, sid, "lettre_motivation", cid, "data.json")).expect("data.json de la lettre");
    assert_no_marker("data.json de la lettre", &lm_json);

    // ------------------------------------------------------------------
    // 2. Prompts : jamais dans la rédaction du CV
    // ------------------------------------------------------------------
    let cv_messages = app.llm.messages_for(Step::CvContent);
    assert!(!cv_messages.is_empty());
    for message in cv_messages {
        assert_no_marker("rédaction du CV", &message);
    }
}

#[tokio::test]
async fn matching_recoit_le_contexte_des_projets_seulement() {
    let (app, sid) = app_with_contexts("contexte-matching").await;

    generate(&app, sid).await;

    let matching = &app.llm.messages_for(Step::Matching)[0];
    assert!(matching.contains(MARQUEUR_PROJET), "{matching}");
    assert!(!matching.contains(MARQUEUR_EXPERIENCE), "jamais le contexte d'une expérience : {matching}");
}

#[tokio::test]
async fn lettre_recoit_les_contextes_avant_le_modele_de_style() {
    let (app, sid) = app_with_contexts("contexte-lettre").await;

    let cid = generate(&app, sid).await;

    let letter = &app.llm.messages_for(Step::LmContent)[0];
    let block = letter.find("CONTEXTE DÉTAILLÉ DU CANDIDAT").expect("bloc du contexte détaillé");
    let style = letter.find("MODÈLE DE STYLE").expect("modèle de style");
    assert!(block < style, "le modèle de style reste la dernière référence lue");
    for marker in [MARQUEUR_EXPERIENCE, MARQUEUR_PROJET] {
        assert!(letter[block..style].contains(marker), "{marker} dans le bloc : {letter}");
    }
    assert!(!warnings(&app, cid).await.iter().any(|w| w.contains(AVERTISSEMENT_RECOPIE)));
}

#[tokio::test]
async fn sans_contexte_prompt_de_lettre_identique() {
    let app = test_state::app("contexte-absent-prompt-identique").await;
    test_state::seed_profile(&app.state.pool).await;
    let sid = seed::session(&app.state.pool).await;
    test_state::seed_references(&app.state, sid).await;

    generate(&app, sid).await;

    // Message capturé avant l'ajout du contexte détaillé ; à régénérer si le
    // prompt de la lettre change volontairement. Les fins de ligne du
    // fichier dépendent de la configuration Git : on les ramène à `\n`.
    let path = fixtures_dir().join("llm").join("lm_prompt_sans_contexte.txt");
    let expected = std::fs::read_to_string(&path).expect("prompt de référence").replace("\r\n", "\n");
    assert_eq!(app.llm.messages_for(Step::LmContent)[0], expected);
}

#[tokio::test]
async fn contexte_jamais_envoye_a_la_traduction() {
    let (app, sid) = app_with_contexts("contexte-traduction").await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_en.json"));

    let cid = generate(&app, sid).await;

    assert!(app.llm.calls_for(Step::ProfileTranslation) > 0, "les entrées du profil sont traduites");
    for step in [Step::ProfileTranslation, Step::ReferenceTranslation, Step::CvLabels, Step::LmLabels] {
        for message in app.llm.messages_for(step) {
            assert_no_marker("traduction", &message);
        }
    }
    let data_json = std::fs::read_to_string(artefact(&app.state, sid, "cv", cid, "data.json")).expect("data.json du CV");
    assert_no_marker("data.json du CV traduit", &data_json);
}

#[tokio::test]
async fn recopie_du_contexte_relance_la_lettre() {
    let (app, sid) = app_with_contexts("contexte-recopie-corrigee").await;
    app.llm.respond_with(Step::LmContent, Reply::Sequence(&["lm_content_recopie_contexte.json", "lm_content_contexte_reformule.json"]));

    let cid = generate(&app, sid).await;

    let messages = app.llm.messages_for(Step::LmContent);
    assert_eq!(messages.len(), 2, "une seule relance après la recopie");
    let retry = &messages[1];
    assert!(retry.contains("PROBLÈMES") && retry.contains("reprend mot pour mot un passage du contexte détaillé"), "{retry}");
    assert!(!warnings(&app, cid).await.iter().any(|w| w.contains(AVERTISSEMENT_RECOPIE)));
}

#[tokio::test]
async fn recopie_persistante_lettre_generee_avec_avertissement() {
    let (app, sid) = app_with_contexts("contexte-recopie-persistante").await;
    app.llm.respond_with(Step::LmContent, Reply::Fixture("lm_content_recopie_contexte.json"));

    let cid = generate(&app, sid).await;

    assert_eq!(app.llm.calls_for(Step::LmContent), usize::from(MAX_WRITING_ATTEMPTS));
    let warnings = warnings(&app, cid).await;
    assert!(warnings.iter().any(|w| w.contains(AVERTISSEMENT_RECOPIE)), "{warnings:?}");
}
