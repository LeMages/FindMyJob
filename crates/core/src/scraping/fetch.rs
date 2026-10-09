//! Récupération du HTML brut d'une page d'offre d'emploi, déclenchée par
//! l'utilisateur pour une seule URL : aucune boucle de retry, aucun
//! contournement de protection. Un site qui refuse la requête simple produit
//! une erreur typée ; c'est l'interface qui propose la suite.

// Importation des modules standard
use std::sync::Arc;
use std::time::Duration;

// Importation des crates externes
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE};
use reqwest::redirect::Policy;
use reqwest::{Client, Response};

// Importation des modules internes
use crate::error::AppError;
use crate::scraping::failure::ScrapingFailure;
use crate::scraping::fetch_classification::{classify_status, classify_transport_error, is_html_content_type};
use crate::scraping::safe_resolver::PublicOnlyResolver;
use crate::scraping::url_validation::{is_allowed_url, validate_offer_url};

/// User-Agent envoyé avec chaque requête — certains sites d'offres bloquent
/// les clients HTTP sans en-tête `User-Agent` reconnaissable.
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
/// En-tête `Accept` d'un navigateur pour une navigation de page.
const ACCEPT_VALUE: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
/// Langues préférées : les offres visées sont francophones.
const ACCEPT_LANGUAGE_VALUE: &str = "fr-FR,fr;q=0.9,en;q=0.8";

/// Délai maximal d'une récupération complète (redirections comprises).
const FETCH_TIMEOUT: Duration = Duration::from_secs(20);
/// Délai maximal d'établissement d'une connexion.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Nombre maximal de redirections suivies.
const MAX_REDIRECTS: usize = 10;
/// Taille maximale du corps HTML accepté (5 Mo, après décompression).
pub const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// Récupère le HTML brut d'une URL d'offre d'emploi.
///
/// Valide l'URL, puis effectue une seule requête GET avec des en-têtes de
/// navigateur. Les cookies posés par le site sont conservés le temps des
/// redirections uniquement (client et cookie store neufs à chaque appel).
/// Seules les adresses publiques sont contactées, y compris après
/// redirection.
///
/// # Arguments
///
/// * `url` - URL de l'offre à récupérer.
///
/// # Returns
///
/// Le HTML de la page, ou `AppError::Scraping` avec la raison typée
/// (URL invalide, offre introuvable, accès refusé, délai dépassé, erreur
/// réseau, contenu non HTML, page trop lourde, page vide).
pub async fn fetch_html(url: &str) -> Result<String, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation de l'URL puis requête unique
    // ------------------------------------------------------------------
    let url = validate_offer_url(url)?;
    let client = build_client()?;
    let response = client.get(url.clone()).send().await.map_err(|e| {
        let failure = classify_transport_error(&e);
        eprintln!("[scraping::fetch] {url} : {failure} ({e})");
        AppError::Scraping(failure)
    })?;

    // ------------------------------------------------------------------
    // 2. Statut HTTP et type de contenu
    // ------------------------------------------------------------------
    let status = response.status().as_u16();
    if let Some(failure) = classify_status(status) {
        eprintln!("[scraping::fetch] {url} : HTTP {status} → {failure}");
        return Err(AppError::Scraping(failure));
    }
    let content_type = response.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok());
    if !is_html_content_type(content_type) {
        eprintln!("[scraping::fetch] {url} : Content-Type {content_type:?} refusé");
        return Err(AppError::Scraping(ScrapingFailure::NotHtml));
    }

    // ------------------------------------------------------------------
    // 3. Corps, plafonné à MAX_RESPONSE_BYTES
    // ------------------------------------------------------------------
    let body = read_capped_body(response).await?;
    let html = String::from_utf8_lossy(&body).into_owned();
    if html.trim().is_empty() {
        return Err(AppError::Scraping(ScrapingFailure::EmptyPage));
    }
    Ok(html)
}

/// Construit un client neuf pour une récupération : cookie store vierge,
/// résolveur DNS filtrant, redirections bornées et revalidées à chaque saut.
fn build_client() -> Result<Client, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_VALUE));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static(ACCEPT_LANGUAGE_VALUE));

    // Une redirection vers une IP littérale privée échapperait au résolveur
    // (aucune résolution DNS) : chaque cible est donc revalidée ici.
    let redirect_policy = Policy::custom(|attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            attempt.error("trop de redirections")
        } else if !is_allowed_url(attempt.url()) {
            attempt.error("redirection vers une adresse refusée")
        } else {
            attempt.follow()
        }
    });

    Client::builder()
        .user_agent(USER_AGENT)
        .default_headers(headers)
        .cookie_provider(Arc::new(Jar::default()))
        .dns_resolver(Arc::new(PublicOnlyResolver))
        .redirect(redirect_policy)
        .timeout(FETCH_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .map_err(|e| AppError::Config(format!("Impossible d'initialiser le client HTTP de récupération : {e}")))
}

/// Lit le corps de la réponse par morceaux et s'arrête dès que la taille
/// dépasse [`MAX_RESPONSE_BYTES`], sans tout charger en mémoire.
async fn read_capped_body(mut response: Response) -> Result<Vec<u8>, AppError> {
    let too_large = || AppError::Scraping(ScrapingFailure::TooLarge);
    // `Content-Length` porte la taille compressée : simple refus anticipé,
    // le compteur ci-dessous reste la vraie garde.
    if response.content_length().is_some_and(|len| len > MAX_RESPONSE_BYTES as u64) {
        return Err(too_large());
    }

    let mut body = Vec::new();
    loop {
        let chunk = response.chunk().await.map_err(|e| {
            let failure = classify_transport_error(&e);
            eprintln!("[scraping::fetch] lecture du corps : {failure} ({e})");
            AppError::Scraping(failure)
        })?;
        let Some(chunk) = chunk else {
            return Ok(body);
        };
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err(too_large());
        }
        body.extend_from_slice(&chunk);
    }
}
