//! Tests des règles de récupération d'une page d'offre, sans réseau :
//! validation de l'URL, politique d'adresses IP, classification des statuts
//! HTTP et des types de contenu, sérialisation des raisons d'échec.

// Importation des modules standard
use std::net::IpAddr;

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::scraping::failure::ScrapingFailure;
use findmyjob_core::scraping::fetch_classification::{classify_status, is_html_content_type};
use findmyjob_core::scraping::ip_policy::is_public_ip;
use findmyjob_core::scraping::url_validation::validate_offer_url;

// ======================================================================
// Validation de l'URL
// ======================================================================

#[test]
fn urls_publiques_acceptees() {
    for url in [
        "https://www.exemple.fr/offres/42",
        "http://exemple.fr/offre?id=3#postuler",
        "  https://jobs.exemple.com:8443/a/b  ",
        "https://93.184.216.34/offre",
    ] {
        assert!(validate_offer_url(url).is_ok(), "refusée à tort : {url}");
    }
}

#[test]
fn urls_refusees() {
    for url in [
        "",
        "pas une url",
        "ftp://exemple.fr/offre",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,<p>offre</p>",
        "https://user:secret@exemple.fr/offre",
        "http://localhost:3000/api",
        "http://LOCALHOST/",
        "http://app.localhost/",
        "http://imprimante.local/",
        "http://intranet/offres",
        "http://127.0.0.1/",
        "http://0x7f.0.0.1/",
        "http://10.0.0.5/",
        "http://172.16.3.4/",
        "http://192.168.1.10/",
        "http://169.254.169.254/latest/meta-data",
        "http://100.64.0.1/",
        "http://0.0.0.0/",
        "http://[::1]/",
        "http://[::ffff:127.0.0.1]/",
        "http://[fd00::1]/",
        "http://[fe80::1]/",
    ] {
        match validate_offer_url(url) {
            Err(AppError::Scraping(ScrapingFailure::InvalidUrl)) => {}
            other => panic!("URL {url:?} : InvalidUrl attendu, obtenu {other:?}"),
        }
    }
}

#[test]
fn politique_ip() {
    let public: [IpAddr; 3] = ["8.8.8.8".parse().unwrap(), "93.184.216.34".parse().unwrap(), "2606:4700::1".parse().unwrap()];
    for ip in public {
        assert!(is_public_ip(ip), "{ip} devrait être publique");
    }
    let private: [IpAddr; 6] = [
        "127.0.0.1".parse().unwrap(),
        "10.1.2.3".parse().unwrap(),
        "198.18.0.1".parse().unwrap(),
        "255.255.255.255".parse().unwrap(),
        "::".parse().unwrap(),
        "::ffff:192.168.0.1".parse().unwrap(),
    ];
    for ip in private {
        assert!(!is_public_ip(ip), "{ip} devrait être refusée");
    }
}

// ======================================================================
// Classification des réponses
// ======================================================================

#[test]
fn classification_des_statuts() {
    assert_eq!(classify_status(200), None);
    assert_eq!(classify_status(204), None);
    assert_eq!(classify_status(404), Some(ScrapingFailure::OfferUnavailable));
    assert_eq!(classify_status(410), Some(ScrapingFailure::OfferUnavailable));
    for status in [401, 403, 429, 999] {
        assert_eq!(classify_status(status), Some(ScrapingFailure::AccessDenied), "statut {status}");
    }
    for status in [400, 500, 502, 503] {
        assert_eq!(classify_status(status), Some(ScrapingFailure::Network), "statut {status}");
    }
}

#[test]
fn classification_des_types_de_contenu() {
    assert!(is_html_content_type(Some("text/html")));
    assert!(is_html_content_type(Some("text/html; charset=utf-8")));
    assert!(is_html_content_type(Some("Application/XHTML+XML")));
    assert!(is_html_content_type(None));
    assert!(!is_html_content_type(Some("application/pdf")));
    assert!(!is_html_content_type(Some("application/json")));
    assert!(!is_html_content_type(Some("image/png")));
}

#[test]
fn raison_serialisee_en_code() {
    let json = serde_json::to_string(&AppError::Scraping(ScrapingFailure::AccessDenied)).expect("sérialisation");
    assert_eq!(json, r#"{"kind":"Scraping","message":"access_denied"}"#);
    let back: AppError = serde_json::from_str(r#"{"kind":"Scraping","message":"insufficient_content"}"#)
        .expect("désérialisation");
    assert!(matches!(back, AppError::Scraping(ScrapingFailure::InsufficientContent)));
}
