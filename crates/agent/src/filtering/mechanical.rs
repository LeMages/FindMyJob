//! Filtre mécanique des offres non scorées : écarte par règles simples
//! (contrat non pertinent, mots-clés exclus, offre trop ancienne) avant que
//! le scoring LLM ([`crate::filtering::scoring`]) ne traite le reste.

// Importation des crates externes
use chrono::Utc;
use sqlx::PgPool;

// Importation des modules internes
use crate::config::FilterConfig;
use crate::db;
use crate::db::offers::UnscoredOffer;
use findmyjob_core::error::AppError;

const SCORE_DISMISSED: f32 = -1.0;
const STATUS_DISMISSED: &str = "dismissed";

/// Détermine si une offre doit être écartée mécaniquement.
///
/// # Arguments
///
/// * `offer` - Offre candidate, non encore scorée.
/// * `filters` - Règles de filtrage configurées (`agent.toml`).
///
/// # Returns
///
/// `true` si l'offre doit être écartée (contrat non pertinent, mot-clé
/// exclu, ou trop ancienne).
fn should_dismiss(offer: &UnscoredOffer, filters: &FilterConfig) -> bool {
    // 1. Type de contrat non pertinent (liste vide = pas de filtre sur le contrat).
    if let Some(contract_type) = &offer.contract_type {
        let allowed = filters.contract_types.is_empty()
            || filters
                .contract_types
                .iter()
                .any(|c| contract_type.to_lowercase().contains(&c.to_lowercase()));
        if !allowed {
            return true;
        }
    }

    // 2. Mot-clé exclu, cherché dans le titre + la description.
    let haystack =
        format!("{} {}", offer.title, offer.description.as_deref().unwrap_or("")).to_lowercase();
    if filters.exclude_keywords.iter().any(|kw| haystack.contains(&kw.to_lowercase())) {
        return true;
    }

    // 3. Offre trop ancienne.
    if let Some(posted_at) = offer.posted_at {
        let age_days = (Utc::now() - posted_at).num_days();
        if age_days > filters.max_days_old as i64 {
            return true;
        }
    }

    false
}

/// Applique le filtre mécanique à toutes les offres non encore scorées.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session dont les offres non scorées doivent être filtrées.
/// * `filters` - Règles de filtrage configurées (`agent.toml`).
///
/// # Returns
///
/// Nombre d'offres écartées.
pub async fn run(pool: &PgPool, session_id: i64, filters: &FilterConfig) -> Result<u64, AppError> {
    let candidates = db::offers::get_unscored_offers(pool, session_id).await?;
    let mut dismissed = 0u64;

    for offer in &candidates {
        if should_dismiss(offer, filters) {
            db::offers::set_score(pool, offer.id, SCORE_DISMISSED, STATUS_DISMISSED).await?;
            dismissed += 1;
        }
    }

    Ok(dismissed)
}
