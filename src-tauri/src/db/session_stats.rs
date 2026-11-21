//! Lecture des données brutes d'une session pour ses statistiques :
//! candidatures (versions de référence, contact), historique et entretiens.
//! Requêtes sur plusieurs tables, sans calcul : voir `services::session_stats`.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::error::AppError;
use crate::models::{StatsCandidature, StatsEvent, StatsInput, StatsInterview};

/// Lit les candidatures, leurs événements et leurs entretiens d'une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Données brutes de la session (événements triés par date croissante).
pub async fn load(pool: &SqlitePool, session_id: i64) -> Result<StatsInput, AppError> {
    Ok(StatsInput {
        candidatures: list_candidatures(pool, session_id).await?,
        events: list_events(pool, session_id).await?,
        interviews: list_interviews(pool, session_id).await?,
    })
}

/// Candidatures de la session avec le numéro de leurs versions de référence.
async fn list_candidatures(pool: &SqlitePool, session_id: i64) -> Result<Vec<StatsCandidature>, AppError> {
    let rows = sqlx::query(
        "SELECT c.id, c.status, c.source, cv.version AS cv_version, lm.version AS lm_version, \
                EXISTS (SELECT 1 FROM candidature_contacts cc WHERE cc.candidature_id = c.id) AS has_contact \
         FROM candidatures c \
         LEFT JOIN document_versions cv ON cv.id = c.cv_reference_version_id \
         LEFT JOIN document_versions lm ON lm.id = c.lm_reference_version_id \
         WHERE c.session_id = $1 \
         ORDER BY c.id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<StatsCandidature, sqlx::Error> {
            Ok(StatsCandidature {
                id: r.try_get("id")?,
                status: r.try_get("status")?,
                source: r.try_get("source")?,
                cv_version: r.try_get("cv_version")?,
                lm_version: r.try_get("lm_version")?,
                has_contact: r.try_get("has_contact")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Événements de l'historique des candidatures de la session.
async fn list_events(pool: &SqlitePool, session_id: i64) -> Result<Vec<StatsEvent>, AppError> {
    let rows = sqlx::query(
        "SELECT h.candidature_id, h.event_type, h.changed_at \
         FROM status_history h \
         JOIN candidatures c ON c.id = h.candidature_id \
         WHERE c.session_id = $1 \
         ORDER BY h.changed_at, h.id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<StatsEvent, sqlx::Error> {
            Ok(StatsEvent {
                candidature_id: r.try_get("candidature_id")?,
                event_type: r.try_get("event_type")?,
                changed_at: r.try_get("changed_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Entretiens des candidatures de la session.
async fn list_interviews(pool: &SqlitePool, session_id: i64) -> Result<Vec<StatsInterview>, AppError> {
    let rows = sqlx::query(
        "SELECT i.candidature_id, i.type_entretien, i.date_entretien, i.issue \
         FROM interviews i \
         JOIN candidatures c ON c.id = i.candidature_id \
         WHERE c.session_id = $1 \
         ORDER BY i.id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<StatsInterview, sqlx::Error> {
            Ok(StatsInterview {
                candidature_id: r.try_get("candidature_id")?,
                type_entretien: r.try_get("type_entretien")?,
                date_entretien: r.try_get("date_entretien")?,
                issue: r.try_get("issue")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}
