//! Validation stricte de l'URL d'une offre avant toute requête : l'application
//! ne charge qu'une page publique en http(s), sur demande de l'utilisateur.

// Importation des modules standard
use std::net::IpAddr;

// Importation des crates externes
use url::{Host, Url};

// Importation des modules internes
use crate::error::AppError;
use crate::scraping::failure::ScrapingFailure;
use crate::scraping::ip_policy::is_public_ip;

/// Valide l'URL d'une offre et la renvoie parsée.
///
/// Refuse : URL non parsable, schéma autre que `http`/`https`, hôte absent,
/// identifiants dans l'URL, `localhost` (et ses sous-domaines), noms sans
/// point (machines du réseau local) et adresses IP non publiques. Les noms
/// DNS publics qui résolvent vers une adresse privée sont refusés plus tard,
/// au moment de la résolution (voir `scraping::safe_resolver`).
///
/// # Arguments
///
/// * `raw` - URL saisie par l'utilisateur.
///
/// # Returns
///
/// L'URL parsée, ou `AppError::Scraping(InvalidUrl)`.
pub fn validate_offer_url(raw: &str) -> Result<Url, AppError> {
    let url = Url::parse(raw.trim()).map_err(|_| invalid())?;
    if is_allowed_url(&url) {
        Ok(url)
    } else {
        Err(invalid())
    }
}

/// Indique si une URL déjà parsée respecte la politique de récupération.
/// Réutilisée pour chaque saut de redirection.
///
/// # Arguments
///
/// * `url` - URL à vérifier.
///
/// # Returns
///
/// `true` si l'URL peut être chargée.
pub fn is_allowed_url(url: &Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    match url.host() {
        Some(Host::Domain(domain)) => is_allowed_domain(domain),
        Some(Host::Ipv4(ip)) => is_public_ip(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => is_public_ip(IpAddr::V6(ip)),
        None => false,
    }
}

/// Refuse `localhost`, `*.localhost` et les noms sans point (ex. `intranet`),
/// qui désignent la machine ou le réseau local.
fn is_allowed_domain(domain: &str) -> bool {
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    !domain.is_empty()
        && domain != "localhost"
        && !domain.ends_with(".localhost")
        && !domain.ends_with(".local")
        && domain.contains('.')
}

fn invalid() -> AppError {
    AppError::Scraping(ScrapingFailure::InvalidUrl)
}
