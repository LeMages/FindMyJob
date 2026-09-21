//! Scoring LLM des offres ayant survécu au filtre mécanique
//! ([`crate::filtering::mechanical`]) : envoie les offres par batchs de 5 à
//! Claude, qui attribue une note de pertinence et une justification par offre.
//!
//! Exception assumée au pattern habituel de `findmyjob-core::llm::*` (schema
//! et prompt construits dans `core`) : la règle du projet interdit de
//! toucher à `crates/core` pour cette fonctionnalité, le prompt et le schema
//! vivent donc ici plutôt que dans `core`.

// Importation des modules standard
use std::collections::HashSet;
use std::time::Duration;

// Importation des crates externes
use serde::Deserialize;
use sqlx::PgPool;

// Importation des modules internes
use crate::config::ProfileConfig;
use crate::db;
use crate::db::offers::UnscoredOffer;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::client::call_claude;

const BATCH_SIZE: usize = 5;
const DESCRIPTION_MAX_CHARS: usize = 500;
const BATCH_DELAY: Duration = Duration::from_secs(1);
const RATE_LIMIT_RETRY_DELAY: Duration = Duration::from_secs(30);
const STATUS_REVIEWED: &str = "reviewed";
/// Nombre maximal de tentatives par batch (1 essai + 1 retry sur JSON
/// invalide/incomplet, indépendamment des reprises sur 429 qui ne consomment
/// pas cette tentative).
const MAX_ATTEMPTS: u32 = 2;

#[derive(Deserialize)]
struct ScoreEntry {
    id: i64,
    score: f32,
    /// Justification du score, affichée en log — non persistée en base
    /// (seuls `score`/`status` le sont).
    reason: String,
}

#[derive(Deserialize)]
struct ScoringResponse {
    scores: Vec<ScoreEntry>,
}

/// Schéma JSON structured output attendu. Objet racine requis par l'API
/// Claude (`output_config.format`) — un tableau nu n'est pas accepté en
/// racine, cohérent avec tous les schémas de `core::llm::*`.
fn scoring_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "scores": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "score": { "type": "number" },
                        "reason": { "type": "string" }
                    },
                    "required": ["id", "score", "reason"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["scores"],
        "additionalProperties": false
    })
}

/// Construit le prompt système, à partir du profil candidat configuré.
fn system_prompt(profile: &ProfileConfig) -> String {
    format!(
        "Tu es un assistant de matching emploi. Tu évalues la pertinence d'offres d'emploi par rapport au profil d'un candidat.\n\n\
         Profil du candidat :\n{}\n\n\
         Pour chaque offre, donne un score de pertinence de 0.0 à 1.0 \
         (0.0-0.3 : pas pertinent, 0.3-0.6 : partiellement pertinent, 0.6-0.8 : pertinent, 0.8-1.0 : très pertinent) \
         et une justification courte.",
        profile.summary
    )
}

/// Construit le message utilisateur listant les offres du batch.
fn user_message(batch: &[&UnscoredOffer]) -> String {
    let entries: Vec<String> = batch
        .iter()
        .map(|o| {
            let description = o
                .description
                .as_deref()
                .map(|d| truncate_chars(d, DESCRIPTION_MAX_CHARS))
                .unwrap_or_default();
            format!(
                "id: {}\ntitre: {}\nentreprise: {}\nlieu: {}\ncontrat: {}\ndescription: {}",
                o.id,
                o.title,
                o.company,
                o.location,
                o.contract_type.as_deref().unwrap_or("non précisé"),
                description,
            )
        })
        .collect();
    format!("Évalue ces offres :\n\n{}", entries.join("\n\n---\n\n"))
}

/// Tronque une chaîne à `max_chars` caractères (pas d'octets, pour rester
/// valide sur les caractères accentués/multi-octets du texte des offres).
fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

/// Appelle Claude pour un batch, avec un retry unique si le JSON retourné ne
/// couvre pas exactement les ids attendus, et une reprise après 30s en cas
/// de rate limit (429).
///
/// # Arguments
///
/// * `batch` - Offres à scorer (5 maximum).
/// * `profile` - Profil candidat, injecté dans le prompt système.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// Les entrées de score correspondant exactement aux ids du batch.
async fn score_batch(
    batch: &[&UnscoredOffer],
    profile: &ProfileConfig,
    model: &str,
) -> Result<Vec<ScoreEntry>, AppError> {
    let system = system_prompt(profile);
    let message = user_message(batch);
    let expected_ids: HashSet<i64> = batch.iter().map(|o| o.id).collect();

    let mut attempt = 0;
    loop {
        let raw = match call_claude(&system, &message, Some(scoring_schema()), model).await {
            Ok(text) => text,
            Err(AppError::Llm(msg)) if msg.contains("429") => {
                tokio::time::sleep(RATE_LIMIT_RETRY_DELAY).await;
                continue;
            }
            Err(e) => return Err(e),
        };

        let parsed: Result<ScoringResponse, _> = serde_json::from_str(&raw);
        let valid = matches!(
            &parsed,
            Ok(resp) if resp.scores.iter().map(|s| s.id).collect::<HashSet<_>>() == expected_ids
        );

        if valid {
            return Ok(parsed.unwrap().scores);
        }

        attempt += 1;
        if attempt >= MAX_ATTEMPTS {
            return Err(AppError::Llm(format!(
                "Réponse de scoring invalide après {MAX_ATTEMPTS} tentative(s). Réponse brute: {raw}"
            )));
        }
    }
}

/// Score par batchs de 5 toutes les offres non encore traitées, et persiste
/// chaque résultat au fil de l'eau.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Session dont les offres non scorées doivent être scorées.
/// * `profile` - Profil candidat configuré (`agent.toml`), injecté dans le prompt.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// Nombre d'offres scorées avec succès.
pub async fn run(pool: &PgPool, session_id: i64, profile: &ProfileConfig, model: &str) -> Result<u64, AppError> {
    let candidates = db::offers::get_unscored_offers(pool, session_id).await?;
    let batches: Vec<&[UnscoredOffer]> = candidates.chunks(BATCH_SIZE).collect();
    let mut scored = 0u64;

    for (i, chunk) in batches.iter().enumerate() {
        let refs: Vec<&UnscoredOffer> = chunk.iter().collect();

        match score_batch(&refs, profile, model).await {
            Ok(entries) => {
                for entry in entries {
                    println!(
                        "[filtering::scoring] offre {} notée {:.2} — {}",
                        entry.id, entry.score, entry.reason
                    );
                    db::offers::set_score(pool, entry.id, entry.score, STATUS_REVIEWED).await?;
                    scored += 1;
                }
            }
            Err(e) => {
                eprintln!("[filtering::scoring] batch {i} en échec: {e}");
                crate::logging::log_error_to_file("scoring batch", &e);
            }
        }

        if i + 1 < batches.len() {
            tokio::time::sleep(BATCH_DELAY).await;
        }
    }

    Ok(scored)
}
