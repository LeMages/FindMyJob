//! Configuration statique de l'agent (profil candidat, filtres de scoring),
//! chargée depuis un fichier TOML au démarrage. Les requêtes de recherche
//! (mots-clés à scraper) ne sont plus dans ce fichier — elles sont propres à
//! chaque session, voir `db::search_queries`.

// Importation des crates externes
use serde::Deserialize;

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Chemin par défaut du fichier de configuration, utilisé si
/// `AGENT_CONFIG_PATH` n'est pas défini.
const DEFAULT_CONFIG_PATH: &str = "./agent.toml";

/// Configuration complète de l'agent, chargée une fois au démarrage puis
/// partagée entre le serveur HTTP et le scheduler.
///
/// Attributes:
///     profile: Profil du candidat, injecté dans le prompt de scoring.
///     filters: Règles du filtre mécanique appliqué avant le scoring LLM.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub profile: ProfileConfig,
    pub filters: FilterConfig,
}

/// Profil du candidat, tel que décrit dans `agent.toml`.
///
/// Attributes:
///     summary: Résumé libre du profil (formation, stack, expérience, zones
///         géographiques...), injecté tel quel dans le prompt de scoring.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileConfig {
    pub summary: String,
}

/// Règles du filtre mécanique appliqué avant le scoring LLM.
///
/// Attributes:
///     contract_types: Types de contrat acceptés (comparaison insensible à
///         la casse, correspondance partielle). Liste vide = pas de filtre
///         sur le contrat.
///     exclude_keywords: Mots-clés qui, présents dans le titre ou la
///         description, font écarter l'offre.
///     max_days_old: Âge maximal (en jours) d'une offre pour rester éligible.
#[derive(Debug, Clone, Deserialize)]
pub struct FilterConfig {
    pub contract_types: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub max_days_old: u64,
}

impl AgentConfig {
    /// Charge la configuration depuis `AGENT_CONFIG_PATH` (défaut
    /// [`DEFAULT_CONFIG_PATH`]).
    ///
    /// # Returns
    ///
    /// `AgentConfig` prête à l'emploi, ou `AppError::Config` si le fichier
    /// est absent ou mal formé.
    pub fn load() -> Result<Self, AppError> {
        let path = std::env::var("AGENT_CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());

        let raw = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Config(format!("Impossible de lire la configuration agent ({path}): {e}")))?;

        toml::from_str(&raw)
            .map_err(|e| AppError::Config(format!("Configuration agent invalide ({path}): {e}")))
    }
}
