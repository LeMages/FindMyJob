//! Import d'une offre par son lien et par capture, au niveau des services
//! appelés par les commandes (`services::scraping`) : vraies pages réduites
//! servies par un serveur local, réponses LLM figées, aucun réseau externe.

// Importation des modules internes
use crate::shared::{fake_llm, offer_server, test_state};
use fake_llm::Step;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{ExtractionStage, OfferExtraction};
use findmyjob_core::scraping::failure::ScrapingFailure;
use findmyjob_core::scraping::fetch::fetch_html_allowing_loopback;
use findmyjob_lib::app_state::AppState;
use findmyjob_lib::services::scraping;
use offer_server::{read_offer, OfferServer, HTML};

/// Échecs pour lesquels l'interface propose la fenêtre de capture
/// (`CAPTURABLE_FAILURES` de `views/new_application/extraction_error.ts`).
const CAPTURABLE: [&str; 5] = ["access_denied", "insufficient_content", "timeout", "network", "empty_page"];

/// Importe une offre par son lien, en visant le serveur local.
async fn import(state: &AppState, url: &str) -> Result<OfferExtraction, AppError> {
    scraping::scrape_offer_with(state.claude(), url, fetch_html_allowing_loopback).await
}

/// Code de l'échec typé d'un import.
fn failure_code(result: Result<OfferExtraction, AppError>) -> &'static str {
    match result {
        Err(AppError::Scraping(failure)) => failure.code(),
        other => panic!("échec typé attendu, obtenu {:?}", other.map(|o| o.fields)),
    }
}

/// Vrai si le caractère est un emoji (plages retirées par `finalize_fields`).
fn is_emoji(c: char) -> bool {
    matches!(c as u32, 0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0xFE0F)
}

/// Vérifie qu'un texte finalisé n'a ni balise HTML ni emoji.
fn assert_clean(label: &str, text: &str) {
    assert!(!text.contains('<') && !text.contains('>'), "{label} contient du HTML : {text}");
    assert!(!text.chars().any(is_emoji), "{label} contient un emoji : {text}");
}

// ======================================================================
// Lien ouvert
// ======================================================================

#[tokio::test]
async fn lien_ouvert_json_ld_donne_une_offre_finalisee() {
    let app = test_state::app("import-json-ld").await;
    let server = OfferServer::start().await;
    let url = server.serve("/doctolib/stage", "ashby_json_ld.html", 200, HTML).await;

    let extraction = import(&app.state, &url).await.expect("import");

    assert!(matches!(extraction.stage, ExtractionStage::JsonLd));
    let fields = extraction.fields;
    assert_eq!(fields.poste.as_deref(), Some("Stage - Assistant Satisfaction (x/f/m) - janvier 2027"));
    assert_eq!(fields.entreprise.as_deref(), Some("Doctolib"));
    let lieu = fields.localisation.expect("localisation");
    assert!(lieu.contains("Levallois-Perret"), "localisation : {lieu}");
    assert_eq!(fields.type_contrat.as_deref(), Some("stage"));
    let description = fields.description_offre.expect("description");
    assert!(description.contains("Vos missions"), "description : {description}");
    assert!(description.contains("Salesforce"));
    assert_clean("description", &description);
    // La page ne nomme aucune personne à contacter.
    assert!(fields.contact.is_none());
    // Étape JSON-LD : ni clé lue, ni appel au modèle.
    assert_eq!(app.llm.total_calls(), 0);
}

#[tokio::test]
async fn lien_ouvert_sans_json_ld_passe_par_le_llm() {
    let app = test_state::app("import-llm").await;
    let server = OfferServer::start().await;
    let url = server.serve("/ubisoft/media-assistant", "smartrecruiters_sans_json_ld.html", 200, HTML).await;

    let extraction = import(&app.state, &url).await.expect("import");

    assert!(matches!(extraction.stage, ExtractionStage::Llm));
    assert_eq!(app.llm.calls_for(Step::ScrapedOffer), 1);
    // Le modèle reçoit le texte de la page, sans son HTML.
    let sent = &app.llm.messages_for(Step::ScrapedOffer)[0];
    assert!(sent.contains("Media Strategy"), "texte envoyé : {sent}");
    assert!(!sent.contains("<li>"));

    let fields = extraction.fields;
    assert_eq!(fields.poste.as_deref(), Some("Media Assistant - Stage (6 mois) Janvier 2027 (F/H/NB)"));
    assert_eq!(fields.entreprise.as_deref(), Some("Ubisoft"));
    assert_eq!(fields.localisation.as_deref(), Some("Paris, France"));
    assert_eq!(fields.type_contrat.as_deref(), Some("stage"));
    let description = fields.description_offre.expect("description");
    assert!(description.contains("Media Strategy Ubisoft Global Publishing"));
    assert_clean("description", &description);
    let contact = fields.contact.expect("contact");
    assert_eq!(contact.nom.as_deref(), Some("Équipe Campus"));
    assert_eq!(contact.email.as_deref(), Some("campus@exemple.invalid"));
}

#[tokio::test]
async fn page_absente_404() {
    let app = test_state::app("import-404").await;
    let server = OfferServer::start().await;
    let url = server.serve("/offre-expiree", "ashby_json_ld.html", 404, HTML).await;

    assert_eq!(failure_code(import(&app.state, &url).await), ScrapingFailure::OfferUnavailable.code());
}

#[tokio::test]
async fn content_type_invalide() {
    let app = test_state::app("import-not-html").await;
    let server = OfferServer::start().await;
    let url = server.serve("/offre.json", "ashby_json_ld.html", 200, "application/json").await;

    assert_eq!(failure_code(import(&app.state, &url).await), "not_html");
}

#[tokio::test]
async fn lien_vide_refuse_avec_un_message_clair() {
    let app = test_state::app("import-vide").await;

    match import(&app.state, "   ").await {
        Err(AppError::Validation(message)) => assert_eq!(message, "L'URL de l'offre est requise."),
        other => panic!("validation attendue, obtenu {:?}", other.map(|o| o.fields)),
    }
}

// ======================================================================
// Lien privé : échec qui propose la capture, puis HTML capturé
// ======================================================================

#[tokio::test]
async fn page_challenge_propose_la_capture() {
    let app = test_state::app("import-challenge").await;
    let server = OfferServer::start().await;
    let url = server.serve("/viewjob", "indeed_challenge.html", 200, HTML).await;

    let code = failure_code(import(&app.state, &url).await);

    assert_eq!(code, "insufficient_content");
    assert!(CAPTURABLE.contains(&code));
    // Une page de vérification n'est jamais envoyée au modèle.
    assert_eq!(app.llm.total_calls(), 0);
}

#[tokio::test]
async fn page_protegee_401_propose_la_capture() {
    let app = test_state::app("import-401").await;
    let server = OfferServer::start().await;
    let url = server.serve("/viewjob", "indeed_challenge.html", 401, HTML).await;

    let code = failure_code(import(&app.state, &url).await);

    assert_eq!(code, "access_denied");
    assert!(CAPTURABLE.contains(&code));
}

#[tokio::test]
async fn capture_donne_une_offre_finalisee() {
    let app = test_state::app("import-capture").await;
    let url = "https://www.welcometothejungle.com/fr/companies/doctolib/jobs/stage-product-content-manager-junior-nantes-x-f-m_nantes";
    let html = read_offer("wttj_capture.html");

    let extraction = scraping::extract_from_html(app.state.claude(), url, &html).await.expect("extraction");

    assert!(matches!(extraction.stage, ExtractionStage::JsonLd));
    let fields = extraction.fields;
    assert_eq!(fields.poste.as_deref(), Some("Stage - Product Content Manager Junior - Nantes (x/f/m)"));
    assert_eq!(fields.entreprise.as_deref(), Some("Doctolib"));
    assert!(fields.localisation.as_deref().is_some_and(|l| l.contains("Nantes")));
    assert_eq!(fields.type_contrat.as_deref(), Some("stage"));
    let description = fields.description_offre.expect("description");
    assert!(description.contains("Doctolib Assistant"));
    assert!(description.contains("Product Knowledge & Selfcare"), "entités décodées : {description}");
    assert_clean("description", &description);
    assert_eq!(app.llm.total_calls(), 0);
}

#[tokio::test]
async fn capture_refuse_une_url_locale() {
    let app = test_state::app("import-capture-locale").await;
    let html = read_offer("wttj_capture.html");

    let result = scraping::extract_from_html(app.state.claude(), "http://127.0.0.1/offre", &html).await;

    assert_eq!(failure_code(result), ScrapingFailure::InvalidUrl.code());
}
