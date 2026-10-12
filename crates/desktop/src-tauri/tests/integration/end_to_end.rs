//! Parcours complet au niveau des services appelés par les commandes :
//! profil → session et références → import d'une offre par son lien
//! (serveur local) → création de la candidature → génération (réponses LLM
//! figées) → passage à « postulé » → suppression. L'état de la base et des
//! fichiers est vérifié à chaque étape.

// Importation des modules internes
use crate::shared::{fake_llm, offer_server, seed, test_state};
use fake_llm::Step;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateCandidatureInput, ExtractionStage, GenerationStatus};
use findmyjob_core::scraping::fetch::fetch_html_allowing_loopback;
use findmyjob_lib::db;
use findmyjob_lib::services::{candidatures, scraping, sessions};
use findmyjob_lib::storage_paths;
use offer_server::{OfferServer, HTML};
use test_state::{artefact, count, wait_generation};

#[tokio::test]
async fn parcours_complet_de_l_offre_a_la_suppression() {
    let app = test_state::app("parcours-complet").await;
    let pool = &app.state.pool;

    // ------------------------------------------------------------------
    // 1. Profil
    // ------------------------------------------------------------------
    test_state::seed_profile(pool).await;
    let profil = db::profile::build_profile(pool).await.expect("profil");
    assert_eq!(profil.user.prenom.as_deref(), Some("Alex"));
    assert_eq!((profil.experiences.len(), profil.projets.len(), profil.skills.len()), (1, 1, 2));

    // ------------------------------------------------------------------
    // 2. Session et références templatisées
    // ------------------------------------------------------------------
    let sid = db::sessions::insert(pool, "Stage 2027", seed::NOW).await.expect("session").id;
    assert!(!db::documents::status_for_session(pool, sid).await.unwrap().all_ready);
    test_state::seed_references(&app.state, sid).await;
    assert!(db::documents::status_for_session(pool, sid).await.unwrap().all_ready);
    let session_dir = storage_paths::to_absolute(&app.state.data_dir, &storage_paths::session_dir(sid));
    assert!(session_dir.exists());

    // ------------------------------------------------------------------
    // 3. Import de l'offre par son lien
    // ------------------------------------------------------------------
    let server = OfferServer::start().await;
    let url = server.serve("/doctolib/stage", "ashby_json_ld.html", 200, HTML).await;
    let offre = scraping::scrape_offer_with(app.state.claude(), &url, fetch_html_allowing_loopback)
        .await
        .expect("import");
    assert!(matches!(offre.stage, ExtractionStage::JsonLd));
    let fields = offre.fields;

    // ------------------------------------------------------------------
    // 4. Création de la candidature à partir des champs importés
    // ------------------------------------------------------------------
    let input = CreateCandidatureInput {
        poste: fields.poste.clone().expect("poste"),
        entreprise: fields.entreprise.clone(),
        localisation: fields.localisation.clone(),
        type_contrat: fields.type_contrat.clone(),
        source: Some("Site carrière".into()),
        remuneration: fields.remuneration.clone(),
        url_offre: Some(url.clone()),
        description_offre: fields.description_offre.clone(),
        notes: None,
        cv_path: None,
        lettre_motivation_path: None,
        session_id: Some(sid),
        contacts: Vec::new(),
    };
    let view = candidatures::create(&app.state, &input).await.expect("création");
    assert_eq!(view.status, "a_postuler");
    assert_eq!(candidatures::find_by_offer_url(pool, &url).await.unwrap().len(), 1);

    // ------------------------------------------------------------------
    // 5. Génération : documents écrits et rattachés
    // ------------------------------------------------------------------
    let progress = wait_generation(&app.state, view.id).await;
    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);
    // L'analyse d'offre reçoit la description importée.
    let analysed = &app.llm.messages_for(Step::OfferAnalysis)[0];
    assert!(analysed.contains("Salesforce"), "offre analysée : {analysed}");
    let detail = db::candidatures::fetch_detail(pool, view.id).await.unwrap();
    assert_eq!(detail.entreprise.as_deref(), Some("Doctolib"));
    assert!(detail.cv_path.is_some() && detail.lettre_motivation_path.is_some());
    for doc_type in ["cv", "lettre_motivation"] {
        for extension in ["pdf", "data.json"] {
            assert!(artefact(&app.state, sid, doc_type, view.id, extension).exists(), "{doc_type}.{extension} absent");
        }
    }
    let lm = std::fs::read_to_string(artefact(&app.state, sid, "lettre_motivation", view.id, "data.json")).unwrap();
    assert!(lm.contains("Doctolib"), "destinataire de la lettre : {lm}");

    // ------------------------------------------------------------------
    // 6. Passage à « postulé »
    // ------------------------------------------------------------------
    candidatures::postuler_sans_message(pool, view.id).await.expect("postuler");
    let detail = db::candidatures::fetch_detail(pool, view.id).await.unwrap();
    assert_eq!(detail.status, "postule");
    assert_eq!(count(pool, "status_history").await, 2);

    // ------------------------------------------------------------------
    // 7. Suppression : refusée seule une fois postulée, puis avec la session
    // ------------------------------------------------------------------
    assert!(matches!(candidatures::delete(&app.state, view.id).await, Err(AppError::Conflict(_))));
    assert!(artefact(&app.state, sid, "cv", view.id, "pdf").exists());

    sessions::delete(&app.state, sid).await.expect("suppression de la session");
    for table in ["sessions", "candidatures", "status_history", "document_versions", "documents"] {
        assert_eq!(count(pool, table).await, 0, "table {table} non vidée");
    }
    assert!(!session_dir.exists(), "dossier de session restant");
    // Le profil n'appartient pas à la session.
    assert_eq!(db::profile::build_profile(pool).await.unwrap().experiences.len(), 1);
}
