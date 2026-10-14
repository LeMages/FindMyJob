//! Serveur HTTP local des tests d'import d'offre : sert les pages de
//! `tests/fixtures/offers/` sur 127.0.0.1, port aléatoire. Il n'existe que
//! dans le processus de test (dev-dependency `wiremock`).

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Content-Type d'une page HTML.
pub const HTML: &str = "text/html; charset=utf-8";

/// Serveur d'offres démarré pour un test.
///
/// # Attributes
///
/// * `server` - Serveur `wiremock` (arrêté à sa destruction).
pub struct OfferServer {
    server: MockServer,
}

impl OfferServer {
    /// Démarre un serveur vide.
    pub async fn start() -> Self {
        Self { server: MockServer::start().await }
    }

    /// Sert une fixture HTML à un chemin donné.
    ///
    /// # Arguments
    ///
    /// * `route` - Chemin servi (ex. `/offre`).
    /// * `fixture` - Nom du fichier dans `tests/fixtures/offers/`.
    /// * `status` - Code HTTP renvoyé.
    /// * `content_type` - En-tête `Content-Type` renvoyé.
    ///
    /// # Returns
    ///
    /// L'URL complète de la page.
    pub async fn serve(&self, route: &str, fixture: &str, status: u16, content_type: &str) -> String {
        let body = std::fs::read(offer_fixture(fixture)).unwrap_or_else(|e| panic!("fixture {fixture} illisible : {e}"));
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(status).set_body_raw(body, content_type))
            .mount(&self.server)
            .await;
        format!("{}{route}", self.server.uri())
    }
}

/// Chemin d'une page de `tests/fixtures/offers/`.
pub fn offer_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("offers").join(name)
}

/// Contenu d'une page de `tests/fixtures/offers/`.
pub fn read_offer(name: &str) -> String {
    std::fs::read_to_string(offer_fixture(name)).unwrap_or_else(|e| panic!("fixture {name} illisible : {e}"))
}
