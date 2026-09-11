//! Client Welcome to the Jungle basé sur leur API Algolia publique
//! (clé search-only, exposée côté client par le frontend WTTJ).

// Importation des modules standard
use std::time::Duration;

// Importation des crates externes
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

// Importation des modules internes
use super::types::{JobScraper, ScrapedOffer, SearchQuery};

const ALGOLIA_ENDPOINT: &str = "https://csekhvms53-dsn.algolia.net/1/indexes/*/queries";
const ALGOLIA_APPLICATION_ID: &str = "CSEKHVMS53";
const ALGOLIA_API_KEY: &str = "4bd8f6215d0cc52b26430765769e65a0";
const ALGOLIA_INDEX: &str = "wk_cms_jobs_production";
/// Referer requis par Algolia : la clé search-only est restreinte aux
/// requêtes qui se présentent comme venant du site WTTJ.
const WTTJ_REFERER: &str = "https://www.welcometothejungle.com/";
const HITS_PER_PAGE: u32 = 30;
/// Garde-fou : on ne va jamais au-delà de cette page, quel que soit `nbPages`.
const MAX_PAGES: u32 = 5;
const PAGE_DELAY: Duration = Duration::from_millis(500);

/// Une offre telle que renvoyée par l'index Algolia `wk_cms_jobs_production`.
///
/// Ne modélise que les champs utilisés par [`ScrapedOffer`] — la réponse
/// complète contient bien plus de champs (images, secteurs, salaire...).
#[derive(Deserialize)]
struct AlgoliaHit {
    slug: String,
    name: String,
    organization: AlgoliaOrganization,
    office: Option<AlgoliaOffice>,
    contract_type_names: Option<AlgoliaContractTypeNames>,
    published_at: Option<String>,
}

#[derive(Deserialize)]
struct AlgoliaOrganization {
    name: String,
    slug: String,
}

#[derive(Deserialize)]
struct AlgoliaOffice {
    city: Option<String>,
}

#[derive(Deserialize)]
struct AlgoliaContractTypeNames {
    fr: Option<String>,
}

#[derive(Deserialize)]
struct AlgoliaResult {
    hits: Vec<AlgoliaHit>,
    #[serde(rename = "nbPages")]
    nb_pages: u32,
}

#[derive(Deserialize)]
struct AlgoliaResponse {
    results: Vec<AlgoliaResult>,
}

/// Client de scraping pour Welcome to the Jungle, via l'API Algolia
/// publique utilisée par leur frontend (pas de scraping HTML : le domaine
/// `welcometothejungle.com` est derrière un challenge AWS WAF JS, mais
/// l'API Algolia est un domaine tiers non protégé).
pub struct WttjScraper {
    client: Client,
}

impl WttjScraper {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Construit le filtre Algolia à partir d'une recherche : toujours
    /// restreint au site FR, puis par pays et par ville si précisés.
    ///
    /// Args:
    ///     query: Recherche pour laquelle construire le filtre.
    ///
    /// Returns:
    ///     Expression de filtre Algolia (`filters`).
    fn build_filters(query: &SearchQuery) -> String {
        let mut filters = "website.reference:wttj_fr".to_string();

        if let Some(country_code) = &query.country_code {
            filters.push_str(&format!(" AND offices.country_code:{country_code}"));
        }
        if let Some(location) = &query.location {
            filters.push_str(&format!(" AND offices.city:{location}"));
        }

        filters
    }

    /// Construit la chaîne `params` Algolia (mots-clés + filtre + pagination).
    ///
    /// Args:
    ///     query: Recherche à exécuter.
    ///     page: Numéro de page (commence à 0).
    ///
    /// Returns:
    ///     Chaîne `params` prête à être envoyée dans le corps de la requête.
    fn build_params(query: &SearchQuery, page: u32) -> String {
        let filters = Self::build_filters(query);
        let keywords = &query.keywords;
        format!("hitsPerPage={HITS_PER_PAGE}&page={page}&query={keywords}&filters={filters}")
    }

    /// Récupère une page de résultats Algolia pour une recherche donnée.
    ///
    /// Args:
    ///     query: Recherche à exécuter.
    ///     page: Numéro de page (commence à 0).
    ///
    /// Returns:
    ///     Le résultat Algolia désérialisé pour cette page, ou une erreur
    ///     réseau/parsing.
    async fn fetch_page(
        &self,
        query: &SearchQuery,
        page: u32,
    ) -> Result<AlgoliaResult, Box<dyn std::error::Error>> {
        let body = json!({
            "requests": [{
                "indexName": ALGOLIA_INDEX,
                "params": Self::build_params(query, page),
            }]
        });

        // Quirk Algolia : le body est du JSON mais le Content-Type attendu
        // est `application/x-www-form-urlencoded` — on force le header
        // explicitement plutôt que de laisser `.json()` imposer le sien.
        let response = self
            .client
            .post(ALGOLIA_ENDPOINT)
            .header("x-algolia-application-id", ALGOLIA_APPLICATION_ID)
            .header("x-algolia-api-key", ALGOLIA_API_KEY)
            .header("content-type", "application/x-www-form-urlencoded")
            .header("referer", WTTJ_REFERER)
            .body(serde_json::to_string(&body)?)
            .send()
            .await?
            .error_for_status()?;

        let mut parsed: AlgoliaResponse = response.json().await?;
        parsed
            .results
            .pop()
            .ok_or_else(|| "Réponse Algolia sans résultat".into())
    }
}

impl Default for WttjScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl From<AlgoliaHit> for ScrapedOffer {
    fn from(hit: AlgoliaHit) -> Self {
        let location = hit
            .office
            .as_ref()
            .and_then(|office| office.city.clone())
            .unwrap_or_else(|| "Lieu non précisé".to_string());
        let url = format!(
            "https://www.welcometothejungle.com/fr/companies/{}/jobs/{}",
            hit.organization.slug, hit.slug
        );

        Self {
            title: hit.name,
            company: hit.organization.name,
            location,
            url,
            contract_type: hit.contract_type_names.and_then(|names| names.fr),
            posted_at: hit.published_at,
        }
    }
}

#[async_trait]
impl JobScraper for WttjScraper {
    async fn scrape(
        &self,
        queries: &[SearchQuery],
    ) -> Result<Vec<ScrapedOffer>, Box<dyn std::error::Error>> {
        let mut offers = Vec::new();

        for query in queries {
            let mut page = 0;

            loop {
                let result = self.fetch_page(query, page).await?;
                let nb_pages = result.nb_pages;
                offers.extend(result.hits.into_iter().map(ScrapedOffer::from));

                page += 1;
                if page >= nb_pages || page >= MAX_PAGES {
                    break;
                }

                tokio::time::sleep(PAGE_DELAY).await;
            }
        }

        Ok(offers)
    }
}
