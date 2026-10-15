//! Classification des réponses et erreurs HTTP de la récupération d'une page
//! d'offre en raisons d'échec typées (`ScrapingFailure`). Fonctions pures,
//! testables sans réseau.

// Importation des modules internes
use crate::scraping::failure::ScrapingFailure;

/// Classe un statut HTTP non 2xx.
///
/// # Arguments
///
/// * `status` - Code de statut HTTP de la réponse.
///
/// # Returns
///
/// `None` si le statut est un succès (2xx), sinon la raison d'échec :
/// 404/410 → offre introuvable ou expirée ; 401/403/429/999 → accès refusé ;
/// tout autre statut → erreur réseau.
pub fn classify_status(status: u16) -> Option<ScrapingFailure> {
    match status {
        200..=299 => None,
        404 | 410 => Some(ScrapingFailure::OfferUnavailable),
        // 999 : code non standard renvoyé par LinkedIn aux clients non connectés.
        401 | 403 | 429 | 999 => Some(ScrapingFailure::AccessDenied),
        _ => Some(ScrapingFailure::Network),
    }
}

/// Indique si un en-tête `Content-Type` désigne une page HTML. Un en-tête
/// absent est accepté : certains serveurs ne l'envoient pas pour du HTML.
///
/// # Arguments
///
/// * `content_type` - Valeur brute de l'en-tête, si présent.
///
/// # Returns
///
/// `true` pour `text/html`, `application/xhtml+xml` ou un en-tête absent.
pub fn is_html_content_type(content_type: Option<&str>) -> bool {
    let Some(raw) = content_type else {
        return true;
    };
    let mime = raw.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    matches!(mime.as_str(), "text/html" | "application/xhtml+xml")
}

/// Classe une erreur de transport `reqwest` (aucune réponse exploitable).
///
/// # Arguments
///
/// * `err` - Erreur renvoyée par `reqwest`.
///
/// # Returns
///
/// `Timeout` si le délai est dépassé, `InvalidUrl` si une redirection mène
/// vers une adresse refusée, `Network` sinon.
pub fn classify_transport_error(err: &reqwest::Error) -> ScrapingFailure {
    if err.is_timeout() {
        ScrapingFailure::Timeout
    } else if err.is_redirect() {
        ScrapingFailure::InvalidUrl
    } else {
        ScrapingFailure::Network
    }
}
