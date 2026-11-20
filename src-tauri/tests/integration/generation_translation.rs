//! Tests : langue des documents, avec les références balisées au contrat.
//! La langue de l'offre choisit la variante de la référence (`L.<lang>`),
//! sans aucun appel de traduction de la référence ; seules les entrées du
//! profil sont traduites, et seulement quand les langues diffèrent.

// Importation des modules internes
use crate::shared::fake_llm::{read_reference, Reply, Step};
use crate::shared::no_dashes::{assert_no_dash_in_data, assert_no_dash_in_text};
use crate::shared::tagged_references::{app_with_tagged_references, data, generate, render};
use findmyjob_lib::llm::offer::OFFER_SYSTEM_PROMPT;

#[tokio::test]
async fn offre_anglaise_documents_entierement_en_anglais() {
    let (app, sid) = app_with_tagged_references("traduction-complete").await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_en.json"));

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    // Variante anglaise de la référence, choisie par `data.lang`.
    let lm = data(&app, sid, "lettre_motivation", cid);
    assert_eq!(lm["lang"], "en");
    let letter = render(&app, sid, "lettre_motivation", &lm).await.text;
    assert!(letter.contains("Dear Sir or Madam,") && letter.contains("Encl.: Curriculum vitae"), "{letter}");
    assert!(!letter.contains("Madame, Monsieur"), "{letter}");
    let cv = data(&app, sid, "cv", cid);
    assert_eq!(cv["lang"], "en");
    let cv_text = render(&app, sid, "cv", &cv).await.text;
    assert!(cv_text.contains("PROFESSIONAL EXPERIENCE"), "{cv_text}");
    assert_eq!(cv["langues"][0]["nom"], "English", "langues traduites");

    // Le rédacteur reçoit la référence rendue dans la langue cible.
    let message = &app.llm.messages_for(Step::LmContent)[0];
    assert!(message.contains("DOCUMENT DE RÉFÉRENCE COMPLET"), "{message}");
    assert!(message.contains("Dear Sir or Madam,"), "référence rendue en anglais : {message}");
    assert!(app.llm.calls_for(Step::ProfileTranslation) > 0, "entrées du profil traduites");
}

#[tokio::test]
async fn documents_generes_sans_tiret() {
    for reference in ["tagged/lm_balisee.typ", "tagged/cv_balise.typ"] {
        assert!(!read_reference(reference).contains('\u{2013}'), "{reference} ne doit contenir aucun « – »");
    }
    let (app, sid) = app_with_tagged_references("references-sans-tiret").await;

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    for doc_type in ["cv", "lettre_motivation"] {
        let generated = data(&app, sid, doc_type, cid);
        assert_no_dash_in_data(doc_type, &generated);
        let rendered = render(&app, sid, doc_type, &generated).await;
        assert_no_dash_in_text(doc_type, &rendered.text);
    }
}

#[tokio::test]
async fn offre_francaise_aucun_appel_de_traduction() {
    let (app, sid) = app_with_tagged_references("traduction-aucune").await;
    let (_, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");
    assert_eq!(app.llm.calls_for(Step::ProfileTranslation), 0);
}

#[tokio::test]
async fn langue_inconnue_documents_dans_la_langue_de_leur_reference() {
    let (app, sid) = app_with_tagged_references("traduction-inconnue").await;
    app.llm.respond_with(Step::OfferAnalysis, Reply::Fixture("offer_analysis_inconnue.json"));

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");
    assert_eq!(data(&app, sid, "cv", cid)["lang"], "fr");
    assert_eq!(data(&app, sid, "lettre_motivation", cid)["lang"], "fr");
    assert_eq!(app.llm.calls_for(Step::ProfileTranslation), 0);
}

#[test]
fn langue_detectee_sur_le_corps_de_l_offre() {
    // Une offre au corps anglais sur une plateforme française est en anglais :
    // la consigne le dit explicitement au modèle.
    assert!(OFFER_SYSTEM_PROMPT.contains("langue du corps de l'offre"));
    assert!(OFFER_SYSTEM_PROMPT.contains("une offre rédigée en anglais sur un site français est en anglais"));
}

#[tokio::test]
async fn ancienne_version_demande_un_reimport() {
    let (app, sid) = app_with_tagged_references("traduction-ancienne-version").await;
    let dir = crate::shared::tagged_references::version_dir(&app, sid, "cv").await;
    std::fs::remove_file(dir.join(findmyjob_lib::models::CONTRACT_FILE)).expect("suppression");

    let (_, error, completed) = generate(&app, sid).await;
    assert!(!completed);
    let error = error.expect("message");
    assert!(error.contains("Réimportez-le"), "{error}");
}
