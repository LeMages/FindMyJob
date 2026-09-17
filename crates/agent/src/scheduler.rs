//! Ordonnancement du pipeline de scraping + filtrage + scoring des offres :
//! une exécution complète au démarrage, puis un job cron à 12h et 18h.

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

/// Expression cron : tous les jours à 12h00 et 18h00.
const CRON_MIDDAY_AND_EVENING: &str = "0 0 12,18 * * *";

/// Exécute une passe complète : scrape → insertion → filtre mécanique →
/// scoring LLM. Chaque étape journalise son résultat ; l'échec d'une étape
/// n'empêche pas l'exécution des étapes suivantes.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `config` - Configuration agent chargée au démarrage (`agent.toml`).
pub async fn run_pipeline(pool: &PgPool, config: &AgentConfig) {
    // ------------------------------------------------------------------
    // 1. Scraping + insertion
    // ------------------------------------------------------------------
    let queries: Vec<SearchQuery> = config.searches.iter().cloned().map(SearchQuery::from).collect();
    let scraper = WttjScraper::new();

    // `JobScraper::scrape` retourne `Box<dyn Error>` (non `Send`) : converti en
    // `String` avant le prochain `await`, cohérent avec l'ancien `run_scraping_once`.
    match scraper.scrape(&queries).await.map_err(|e| e.to_string()) {
        Ok(offers) => match db::offers::insert_offers(pool, &offers).await {
            Ok(inserted) => println!("[pipeline] scraping : {inserted} nouvelle(s) offre(s) insérée(s)"),
            Err(e) => {
                eprintln!("[pipeline] erreur insertion: {e}");
                logging::log_error_to_file("insertion offres", &e);
            }
        },
        Err(e) => eprintln!("[pipeline] erreur scraping: {e}"),
    }

    // ------------------------------------------------------------------
    // 2. Filtre mécanique
    // ------------------------------------------------------------------
    match mechanical::run(pool, &config.filters).await {
        Ok(dismissed) => println!("[pipeline] filtre mécanique : {dismissed} offre(s) écartée(s)"),
        Err(e) => {
            eprintln!("[pipeline] erreur filtre mécanique: {e}");
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

    match scoring::run(pool, &config.profile, &model).await {
        Ok(count) => println!("[pipeline] scoring LLM : {count} offre(s) scorée(s)"),
        Err(e) => {
            eprintln!("[pipeline] erreur scoring LLM: {e}");
            logging::log_error_to_file("scoring LLM", &e);
        }
    }
}

/// Démarre le scheduler : exécution immédiate du pipeline en tâche de fond,
/// puis job cron à 12h et 18h chaque jour.
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
    // Exécution immédiate, ne bloque pas le démarrage du serveur HTTP.
    {
        let pool = pool.clone();
        let config = config.clone();
        tokio::spawn(async move { run_pipeline(&pool, &config).await });
    }

    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| AppError::Config(format!("Impossible d'initialiser le scheduler: {e}")))?;

    let job = Job::new_async(CRON_MIDDAY_AND_EVENING, move |_uuid, _lock| {
        let pool = pool.clone();
        let config = config.clone();
        Box::pin(async move { run_pipeline(&pool, &config).await })
    })
    .map_err(|e| AppError::Config(format!("Expression cron invalide: {e}")))?;

    scheduler.add(job).await.map_err(|e| AppError::Config(format!("Impossible d'ajouter le job cron: {e}")))?;
    scheduler.start().await.map_err(|e| AppError::Config(format!("Impossible de démarrer le scheduler: {e}")))?;

    Ok(scheduler)
}
