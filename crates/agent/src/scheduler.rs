//! Ordonnancement du pipeline de scraping + filtrage + scoring des offres,
//! par session : job cron à 12h et 18h itérant sur les sessions actives.
//! Le déclenchement manuel (`POST /api/sessions/{id}/search`) appelle
//! directement [`run_search_for_session`].

// Importation des crates externes
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

// Importation des modules internes
use crate::config::AgentConfig;
use crate::db;
use crate::filtering::{mechanical, scoring};
use crate::logging;
use crate::scraping::types::{JobScraper, SearchQuery};
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
    // 1. Scraping + insertion, une requête à la fois
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
        let search_query: SearchQuery = query.into();

        // `JobScraper::scrape` retourne `Box<dyn Error>` (non `Send`) : converti en
        // `String` avant le prochain `await`, cohérent avec l'ancien `run_pipeline`.
        match scraper.scrape(std::slice::from_ref(&search_query)).await.map_err(|e| e.to_string()) {
            Ok(offers) => match db::offers::insert_offers(pool, session_id, &offers).await {
                Ok(inserted) => println!(
                    "[pipeline] session {session_id} : \"{}\" — {inserted} nouvelle(s) offre(s) insérée(s)",
                    query.keywords
                ),
                Err(e) => {
                    eprintln!("[pipeline] session {session_id} : erreur insertion: {e}");
                    logging::log_error_to_file("insertion offres", &e);
                }
            },
            Err(e) => eprintln!(
                "[pipeline] session {session_id} : erreur scraping \"{}\": {e}",
                query.keywords
            ),
        }
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

    match scoring::run(pool, session_id, &config.profile, &model).await {
        Ok(count) => println!("[pipeline] session {session_id} : scoring LLM — {count} offre(s) scorée(s)"),
        Err(e) => {
            eprintln!("[pipeline] session {session_id} : erreur scoring LLM: {e}");
            logging::log_error_to_file("scoring LLM", &e);
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
