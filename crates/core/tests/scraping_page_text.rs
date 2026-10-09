//! Tests de l'extraction du texte d'une page pour l'étage LLM
//! (`findmyjob_core::scraping::extract`) : choix de la racine, titres et
//! listes conservés, zones annexes exclues, détection des pages sans
//! contenu exploitable (coquille JavaScript, challenge, bandeau cookies).

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::scraping::extract::{extract_page_text, MAX_CHARS};
use findmyjob_core::scraping::failure::ScrapingFailure;

// ======================================================================
// Helpers
// ======================================================================

/// Corps d'une offre fictive (missions, profil), assez long pour être exploitable.
const OFFER_BODY: &str = "<h2>Vos missions</h2>\
<ul><li>Participer à la refonte du site e-commerce de l'enseigne</li>\
<li>Développer des composants Vue réutilisables et documentés</li>\
<li>Écrire des tests automatisés et participer aux revues de code</li></ul>\
<h2>Profil recherché</h2>\
<p>Étudiant en dernière année d'école d'ingénieur ou de master informatique, vous maîtrisez \
JavaScript et TypeScript et vous avez déjà réalisé un projet web complet. Curieux, autonome \
et rigoureux, vous aimez travailler en équipe.</p>";

/// Vérifie qu'une erreur est un échec « contenu insuffisant ».
fn assert_insufficient(result: Result<impl std::fmt::Debug, AppError>) {
    match result {
        Err(AppError::Scraping(ScrapingFailure::InsufficientContent)) => {}
        other => panic!("contenu insuffisant attendu, obtenu : {other:?}"),
    }
}

// ======================================================================
// Choix de la racine et rendu
// ======================================================================

#[test]
fn premiere_section_annexe_n_est_pas_retenue() {
    let html = format!(
        "<html><body>\
         <section class=\"related\"><h3>Offres similaires</h3><p>Stage marketing — Lyon</p></section>\
         <section class=\"offer\"><h1>Stage Développeur Front-end</h1>{OFFER_BODY}</section>\
         </body></html>"
    );
    let page = extract_page_text(&html).expect("texte exploitable");
    assert!(page.text.contains("Stage Développeur Front-end"));
    assert!(page.text.contains("Vos missions"));
    assert!(!page.text.contains("Offres similaires"), "section annexe retenue :\n{}", page.text);
}

#[test]
fn titres_et_listes_conserves() {
    let html = format!("<html><body><main>{OFFER_BODY}</main></body></html>");
    let page = extract_page_text(&html).expect("texte exploitable");
    let lines: Vec<&str> = page.text.lines().collect();
    assert!(lines.contains(&"Vos missions"), "titre perdu :\n{}", page.text);
    assert!(lines.contains(&"Profil recherché"));
    assert!(lines.contains(&"- Développer des composants Vue réutilisables et documentés"));
}

#[test]
fn navigation_et_pied_de_page_exclus() {
    let html = format!(
        "<html><body><nav class=\"job-menu\">Accueil Offres Entreprises Connexion</nav>\
         <article>{OFFER_BODY}<footer>Mentions légales</footer></article>\
         <footer>© Exemple 2026</footer></body></html>"
    );
    let page = extract_page_text(&html).expect("texte exploitable");
    assert!(!page.text.contains("Connexion"));
    assert!(!page.text.contains("Mentions légales"));
    assert!(page.text.contains("Profil recherché"));
}

#[test]
fn repli_sur_body_sans_conteneur_semantique() {
    let html = format!("<html><body><div>{OFFER_BODY}</div></body></html>");
    let page = extract_page_text(&html).expect("texte exploitable");
    assert!(page.text.contains("Vos missions"));
}

#[test]
fn troncature_apres_isolement_du_contenu() {
    // Une longue navigation avant l'offre ne doit pas consommer le budget de
    // caractères : seule l'offre isolée est tronquée.
    let long_nav = "<nav>".to_string() + &"Lien de menu ".repeat(2_000) + "</nav>";
    let long_offer = "<p>".to_string() + &"Mission détaillée de l'offre. ".repeat(1_000) + "</p>";
    let html = format!("<html><body>{long_nav}<main><h1>Stage Data</h1>{long_offer}</main></body></html>");
    let page = extract_page_text(&html).expect("texte exploitable");
    assert!(page.text.starts_with("Stage Data"), "début inattendu : {:?}", &page.text[..40]);
    assert_eq!(page.text.chars().count(), MAX_CHARS);
}

#[test]
fn indices_de_titre() {
    let html = format!(
        "<html><head><title>  Stage Front-end\n | Exemple </title>\
         <meta property=\"og:title\" content=\"Stage Front-end chez Exemple\"></head>\
         <body><main>{OFFER_BODY}</main></body></html>"
    );
    let page = extract_page_text(&html).expect("texte exploitable");
    assert_eq!(page.hints.title.as_deref(), Some("Stage Front-end | Exemple"));
    assert_eq!(page.hints.og_title.as_deref(), Some("Stage Front-end chez Exemple"));
}

// ======================================================================
// Pages sans contenu exploitable
// ======================================================================

#[test]
fn coquille_javascript() {
    let html = "<html><head><title>Exemple Jobs</title><script src=\"/app.js\"></script></head>\
                <body><noscript>You need to enable JavaScript to run this app.</noscript>\
                <div id=\"root\"></div></body></html>";
    assert_insufficient(extract_page_text(html));
}

#[test]
fn page_de_challenge() {
    let html = "<html><head><title>Just a moment...</title></head><body><main>\
                <h1>Just a moment...</h1>\
                <p>Checking your browser before accessing the website. This process is automatic. \
                Your browser will redirect to your requested content shortly. Please allow up to 5 seconds. \
                Enable JavaScript and cookies to continue. Ray ID: 0000000000000000. \
                Performance and security by an anti-bot service.</p>\
                <p>Si la page ne se charge pas, vérifiez que vous êtes humain en cochant la case ci-dessous \
                puis patientez quelques instants avant de réessayer.</p></main></body></html>";
    assert_insufficient(extract_page_text(html));
}

#[test]
fn bandeau_cookies_seul() {
    let html = "<html><body><div class=\"cookie-banner\"><p>Nous utilisons des cookies pour améliorer \
                votre expérience.</p><button>Tout accepter</button><button>Refuser</button></div>\
                <div id=\"app\"></div></body></html>";
    assert_insufficient(extract_page_text(html));
}

#[test]
fn vraie_offre_avec_mention_captcha_reste_exploitable() {
    // Au-delà d'une certaine longueur, un marqueur de challenge isolé (ici
    // dans le formulaire de candidature) ne disqualifie pas la page.
    let html = format!(
        "<html><body><main><h1>Stage Développeur</h1>{OFFER_BODY}{OFFER_BODY}{OFFER_BODY}{OFFER_BODY}{OFFER_BODY}{OFFER_BODY}\
         <p>Protégé par captcha.</p></main></body></html>"
    );
    assert!(extract_page_text(&html).is_ok());
}
