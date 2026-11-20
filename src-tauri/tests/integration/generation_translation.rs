//! Tests : langue des documents et traduction complète, avec les références
//! balisées (zones [FIXE] / [VARIABLE] / [ADAPTABLE], texte hors
//! schéma) templatisées par le LLM factice.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::shared::fake_llm::{read_reference, Reply, Step};
use crate::shared::no_dashes::{assert_no_dash_in_data, assert_no_dash_in_text};
use crate::shared::tagged_references::{app_with_tagged_references, data, generate, version_dir};
use findmyjob_lib::documents::typst::compile_template;
use findmyjob_lib::llm::offer::OFFER_SYSTEM_PROMPT;
use findmyjob_lib::models::REFERENCE_TEXTS_FILE;
use findmyjob_lib::storage_paths;

#[tokio::test]
async fn offre_anglaise_documents_entierement_en_anglais() {
    let (app, sid) = app_with_tagged_references("traduction-complete").await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_en.json"));

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    // Textes en dur et libellés dans la langue de l'offre.
    let lm = data(&app, sid, "lettre_motivation", cid);
    assert_eq!(lm["lang"], "en");
    assert_eq!(lm["texts"]["t1"], "[en] P.J. : Curriculum vitae");
    assert_eq!(lm["labels"]["salutation"], "Dear Sir or Madam,");
    let cv = data(&app, sid, "cv", cid);
    assert_eq!(cv["lang"], "en");
    for id in ["t1", "t2"] {
        assert!(cv["texts"][id].as_str().is_some_and(|t| t.starts_with("[en] ")), "{}", cv["texts"]);
    }
    assert_eq!(cv["langues"][0]["nom"], "English", "langues traduites");

    // Zones [FIXE], consignes et langue cible transmises à la rédaction.
    let message = &app.llm.messages_for(Step::LmContent)[0];
    assert!(message.contains("[en] Actuellement étudiant"), "zone [FIXE] traduite");
    assert!(message.contains("[en] - Toutes les valeurs"), "consignes générales traduites");
    assert_eq!(app.llm.calls_for(Step::ReferenceTranslation), 2, "une traduction par référence");
    // Le contrôle des zones [FIXE] compare au texte fixe traduit.
    let debug = storage_paths::to_absolute(&app.state.data_dir, &storage_paths::debug_dir(sid, cid));
    let lint = std::fs::read_to_string(debug.join("lint_report.json")).expect("rapport du linter");
    assert!(lint.contains("[en] Actuellement") && !lint.contains("« Actuellement"), "{lint}");

    // Caches créés, puis réutilisés à la génération suivante.
    for doc_type in ["cv", "lettre_motivation"] {
        assert!(version_dir(&app, sid, doc_type).await.join("reference_texts.en.json").exists());
    }
    let (_, error, completed) = generate(&app, sid).await;
    assert!(completed, "seconde génération : {error:?}");
    assert_eq!(app.llm.calls_for(Step::ReferenceTranslation), 2, "caches réutilisés");
    assert_eq!(app.llm.calls_for(Step::CvLabels), 1, "libellés en cache");
}

#[tokio::test]
async fn references_avec_tirets_documents_sans_tiret() {
    // Les références balisées emploient des « – » (objet de la lettre,
    // périodes et titres du CV) : aucun ne doit rester dans les documents.
    for reference in ["tagged/lm_balisee.typ", "tagged/cv_balise.typ"] {
        assert!(read_reference(reference).contains('\u{2013}'), "{reference} doit contenir un « – »");
    }
    let (app, sid) = app_with_tagged_references("references-sans-tiret").await;

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    for doc_type in ["cv", "lettre_motivation"] {
        let generated = data(&app, sid, doc_type, cid);
        assert_no_dash_in_data(doc_type, &generated);
        // Les templates de ces références n'écrivent aucun tiret en dur : le
        // texte rendu n'en contient donc aucun non plus.
        let dir = version_dir(&app, sid, doc_type).await;
        let template = std::fs::read_to_string(dir.join("template.typ")).expect("template");
        let raw = serde_json::to_vec(&generated).expect("data.json");
        let rendered = compile_template(template, raw, dir, app.state.fonts_dir.clone()).expect("compilation");
        assert_no_dash_in_text(doc_type, &rendered.text);
    }
}

#[tokio::test]
async fn offre_francaise_aucun_appel_de_traduction() {
    let (app, sid) = app_with_tagged_references("traduction-aucune").await;
    let (_, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");
    for step in [Step::ReferenceTranslation, Step::CvLabels, Step::LmLabels, Step::ProfileTranslation] {
        assert_eq!(app.llm.calls_for(step), 0, "{step:?}");
    }
}

#[tokio::test]
async fn langue_inconnue_documents_dans_la_langue_de_leur_reference() {
    let (app, sid) = app_with_tagged_references("traduction-inconnue").await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_inconnue.json"));

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");
    assert_eq!(data(&app, sid, "cv", cid)["lang"], "fr");
    assert_eq!(data(&app, sid, "lettre_motivation", cid)["lang"], "fr");
    assert_eq!(app.llm.calls_for(Step::ReferenceTranslation), 0);
}

#[test]
fn langue_detectee_sur_le_corps_de_l_offre() {
    // Une offre au corps anglais sur une plateforme française est en anglais :
    // la consigne le dit explicitement au modèle.
    assert!(OFFER_SYSTEM_PROMPT.contains("langue du corps de l'offre"));
    assert!(OFFER_SYSTEM_PROMPT.contains("une offre rédigée en anglais sur un site français est en anglais"));
}

#[tokio::test]
async fn cache_invalide_non_reutilise() {
    let (app, sid) = app_with_tagged_references("traduction-cache-invalide").await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_en.json"));
    let (_, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    // Cache de la lettre altéré : un texte vidé.
    let cache = version_dir(&app, sid, "lettre_motivation").await.join("reference_texts.en.json");
    let mut cached: Value = serde_json::from_str(&std::fs::read_to_string(&cache).expect("cache")).expect("JSON");
    cached["texts"]["texts"]["t1"] = "".into();
    std::fs::write(&cache, cached.to_string()).expect("écriture");

    let (_, error, completed) = generate(&app, sid).await;
    assert!(completed, "seconde génération : {error:?}");
    assert_eq!(app.llm.calls_for(Step::ReferenceTranslation), 3, "la lettre est retraduite, pas le CV");
}

#[tokio::test]
async fn ancienne_version_demande_un_reimport() {
    let (app, sid) = app_with_tagged_references("traduction-ancienne-version").await;
    let texts = version_dir(&app, sid, "cv").await.join(REFERENCE_TEXTS_FILE);
    std::fs::remove_file(&texts).expect("suppression");

    let (_, error, completed) = generate(&app, sid).await;
    assert!(!completed);
    let error = error.expect("message");
    assert!(error.contains("Réimportez-le"), "{error}");
}
