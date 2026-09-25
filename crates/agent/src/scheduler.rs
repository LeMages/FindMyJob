//! Ordonnancement du pipeline de scraping + filtrage + scoring des offres,
//! par session : job cron à 12h et 18h itérant sur les sessions actives.
//! Le déclenchement manuel (`POST /api/sessions/{id}/search`) appelle
//! directement [`run_search_for_session`].

// Importation des modules standard
use std::collections::HashSet;

// Importation des crates externes
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

// Importation des modules internes
use crate::config::AgentConfig;
use crate::db;
use crate::filtering::profile_summary::format_scoring_context;
use crate::filtering::{mechanical, scoring};
use crate::logging;
use crate::scraping::types::{JobScraper, ScrapedOffer, SearchQuery};
use crate::scraping::wttj::WttjScraper;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::client::DEFAULT_MODEL;
use findmyjob_core::models::SearchQueryView;

/// Expression cron : tous les jours à 12h00 et 18h00.
const CRON_MIDDAY_AND_EVENING: &str = "0 0 12,18 * * *";

impl From<&SearchQueryView> for SearchQuery {
    fn from(view: &SearchQueryView) -> Self {
        SearchQuery {
            keywords: view.keywords.clone(),
            location: view.location.clone(),
            country_code: view.country_code.clone(),
        }
    }
}

/// Exécute une recherche complète pour une session : scraping (une passe par
/// requête, pas un batch concaténé — isole l'erreur d'une requête sans
/// affecter les autres), insertion, filtre mécanique puis scoring LLM,
/// scopés à la session. Chaque étape journalise son résultat ; l'échec d'une
/// étape n'empêche pas l'exécution des étapes suivantes.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session dont on exécute les requêtes de recherche.
/// * `config` - Configuration agent chargée au démarrage (`agent.toml`).
pub async fn run_search_for_session(pool: &PgPool, session_id: i64, config: &AgentConfig) {
    // ------------------------------------------------------------------
    // 1. Scraping + enrichissement + insertion, une requête à la fois
    // ------------------------------------------------------------------
    let queries = match db::search_queries::list_for_session(pool, session_id).await {
        Ok(queries) => queries,
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur chargement des requêtes: {e}");
            logging::log_error_to_file("chargement requêtes de recherche", &e);
            return;
        }
    };

    let scraper = WttjScraper::new();
    for query in &queries {
        scrape_and_insert(pool, session_id, &scraper, query).await;
    }

    // ------------------------------------------------------------------
    // 2. Filtre mécanique
    // ------------------------------------------------------------------
    match mechanical::run(pool, session_id, &config.filters).await {
        Ok(dismissed) => println!("[pipeline] session {session_id} : filtre mécanique — {dismissed} offre(s) écartée(s)"),
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur filtre mécanique: {e}");
            logging::log_error_to_file("filtre mécanique", &e);
        }
    }

    // ------------------------------------------------------------------
    // 3. Scoring LLM — modèle résolu depuis les paramètres (settings), avec
    //    repli sur DEFAULT_MODEL, comme partout ailleurs dans l'agent.
    // ------------------------------------------------------------------
    let model = db::settings::resolve_llm_model(pool, DEFAULT_MODEL)
        .await
        .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

    let Some(scoring_context) = load_scoring_context(pool, session_id).await else {
        return;
    };

    match scoring::run(pool, session_id, &scoring_context, &model).await {
        Ok(count) => println!("[pipeline] session {session_id} : scoring LLM — {count} offre(s) scorée(s)"),
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur scoring LLM: {e}");
            logging::log_error_to_file("scoring LLM", &e);
        }
    }
}

/// Scrape une requête, écarte les offres déjà connues, enrichit les
/// nouvelles (description complète via l'API WTTJ v1) puis les insère.
/// Chaque échec est journalisé sans interrompre la session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session à laquelle rattacher les offres.
/// * `scraper` - Client WTTJ partagé entre les requêtes de la session.
/// * `query` - Requête de recherche à exécuter.
async fn scrape_and_insert(pool: &PgPool, session_id: i64, scraper: &WttjScraper, query: &SearchQueryView) {
    // ------------------------------------------------------------------
    // 1. Scraping Algolia
    // ------------------------------------------------------------------
    let search_query: SearchQuery = query.into();
    // `JobScraper::scrape` retourne `Box<dyn Error>` (non `Send`) : converti en
    // `String` avant le prochain `await`, cohérent avec l'ancien `run_pipeline`.
    let scraped = match scraper.scrape(std::slice::from_ref(&search_query)).await.map_err(|e| e.to_string()) {
        Ok(offers) => offers,
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur scraping \"{}\": {e}", query.keywords);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 2. Dédup puis enrichissement des seules nouvelles offres : un appel
    //    HTTP par offre, inutile pour celles déjà en base. Si la dédup
    //    échoue, on insère sans enrichir (ON CONFLICT reste la garde).
    // ------------------------------------------------------------------
    let mut offers = dedup_by_url(scraped);
    let urls: Vec<String> = offers.iter().map(|o| o.url.clone()).collect();
    match db::offers::find_existing_urls(pool, &urls).await {
        Ok(existing) => {
            offers.retain(|o| !existing.contains(&o.url));
            let enriched = scraper.enrich(&mut offers).await;
            println!(
                "[pipeline] session {session_id} : \"{}\" — {enriched}/{} description(s) complète(s) via l'API v1",
                query.keywords,
                offers.len()
            );
        }
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : dédup impossible, descriptions non enrichies: {e}");
            logging::log_error_to_file("dédup offres avant enrichissement", &e);
        }
    }

    // ------------------------------------------------------------------
    // 3. Insertion
    // ------------------------------------------------------------------
    match db::offers::insert_offers(pool, session_id, &offers).await {
        Ok(inserted) => println!(
            "[pipeline] session {session_id} : \"{}\" — {inserted} nouvelle(s) offre(s) insérée(s)",
            query.keywords
        ),
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur insertion: {e}");
            logging::log_error_to_file("insertion offres", &e);
        }
    }
}

/// Retire les doublons d'URL d'un lot (une même offre peut remonter sur
/// plusieurs pages Algolia), en gardant la première occurrence.
fn dedup_by_url(offers: Vec<ScrapedOffer>) -> Vec<ScrapedOffer> {
    let mut seen = HashSet::new();
    offers.into_iter().filter(|o| seen.insert(o.url.clone())).collect()
}

/// Charge le profil (DB) et la session pour construire le contexte du scoring.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session dont `zone`/`contexte` complètent le profil.
///
/// # Returns
///
/// Le texte de contexte, ou `None` (avec log) si le profil n'est pas
/// configuré ou si une lecture échoue — les offres restent alors non scorées.
async fn load_scoring_context(pool: &PgPool, session_id: i64) -> Option<String> {
    let profile = match db::profile::build_profile(pool).await {
        Ok(profile) => profile,
        Err(AppError::NotFound(_)) => {
            eprintln!("[pipeline] session {session_id} : Profil non configuré, scoring ignoré");
            return None;
        }
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur chargement du profil: {e}");
            logging::log_error_to_file("chargement profil pour scoring", &e);
            return None;
        }
    };

    match db::sessions::fetch_one(pool, session_id).await {
        Ok(session) => Some(format_scoring_context(&profile, &session)),
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur chargement de la session: {e}");
            logging::log_error_to_file("chargement session pour scoring", &e);
            None
        }
    }
}

/// Démarre le scheduler : job cron à 12h et 18h chaque jour, itérant sur les
/// sessions au statut `active`. Aucune exécution immédiate au démarrage —
/// contrairement à l'ancien pipeline global, la recherche est désormais
/// pilotée par session (bouton manuel ou cron).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `config` - Configuration agent chargée au démarrage.
///
/// # Returns
///
/// Le `JobScheduler` démarré — l'appelant doit le garder en vie (ne pas le
/// laisser sortir de portée) tant que le processus tourne.
pub async fn start(pool: PgPool, config: AgentConfig) -> Result<JobScheduler, AppError> {
    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| AppError::Config(format!("Impossible d'initialiser le scheduler: {e}")))?;

    let job = Job::new_async(CRON_MIDDAY_AND_EVENING, move |_uuid, _lock| {
        let pool = pool.clone();
        let config = config.clone();
        Box::pin(async move {
            let sessions = db::sessions::list_all(&pool).await.unwrap_or_else(|e| {
                eprintln!("[pipeline] erreur chargement des sessions: {e}");
                Vec::new()
            });

            for session in sessions.into_iter().filter(|s| s.statut == "active") {
                run_search_for_session(&pool, session.id, &config).await;
            }
        })
    })
    .map_err(|e| AppError::Config(format!("Expression cron invalide: {e}")))?;

    scheduler.add(job).await.map_err(|e| AppError::Config(format!("Impossible d'ajouter le job cron: {e}")))?;
    scheduler.start().await.map_err(|e| AppError::Config(format!("Impossible de démarrer le scheduler: {e}")))?;

    Ok(scheduler)
}
