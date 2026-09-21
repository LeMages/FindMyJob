//! Accès et logique de persistance pour les entretiens rattachés à une
//! candidature. `interviews` n'a pas de colonne `user_id` : la propriété se
//! vérifie via `candidature_id` (comme `db::contacts`), jamais via
//! `db::common::assert_owned_by_user`.

// Importation des crates externes
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{InterviewView, PlanningInterviewView};

/// Types d'entretien acceptés (voir `interviews.type` en base).
pub const TYPES_ENTRETIEN_VALIDES: [&str; 5] = ["rh", "technique", "manager", "final", "autre"];

/// Issues possibles pour un entretien passé (voir `interviews.issue` en base).
pub const ISSUES_VALIDES: [&str; 2] = ["positif", "negatif"];

/// Modalités d'entretien acceptées (voir `interviews.modalite` en base).
pub const MODALITES_VALIDES: [&str; 3] = ["visio", "presentiel", "telephone"];

/// Liste les entretiens encore en attente d'une candidature — utilisé pour
/// proposer à l'utilisateur lequel marquer comme passé.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entretiens au statut `en_attente`, triés par date d'entretien croissante.
pub async fn list_pending_for_candidature(
    pool: &PgPool,
    candidature_id: i64,
) -> Result<Vec<InterviewView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, type AS type_entretien, \
                date_entretien, lieu, issue, notes, contact_id \
         FROM search.interviews \
         WHERE candidature_id = $1 AND issue = 'en_attente' \
         ORDER BY date_entretien",
    )
    .bind(candidature_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<InterviewView, sqlx::Error> {
            Ok(InterviewView {
                id: r.try_get("id")?,
                candidature_id: r.try_get("candidature_id")?,
                type_entretien: r.try_get("type_entretien")?,
                date_entretien: r.try_get("date_entretien")?,
                lieu: r.try_get("lieu")?,
                issue: r.try_get("issue")?,
                notes: r.try_get("notes")?,
                contact_id: r.try_get("contact_id")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Liste les entretiens planifiés d'une session, avec entreprise et poste de
/// leur candidature — utilisé par la vue planning.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Entretiens dont la date est renseignée, triés par date croissante.
pub async fn list_for_session(
    pool: &PgPool,
    session_id: i64,
) -> Result<Vec<PlanningInterviewView>, AppError> {
    let rows = sqlx::query(
        "SELECT i.id, i.candidature_id, c.entreprise, c.poste, \
                i.type AS type_entretien, \
                i.date_entretien, \
                i.lieu, i.modalite, i.issue, i.notes \
         FROM search.interviews i \
         JOIN search.candidatures c ON c.id = i.candidature_id \
         WHERE c.session_id = $1 AND i.date_entretien IS NOT NULL \
         ORDER BY i.date_entretien",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<PlanningInterviewView, sqlx::Error> {
            Ok(PlanningInterviewView {
                id: r.try_get("id")?,
                candidature_id: r.try_get("candidature_id")?,
                entreprise: r.try_get("entreprise")?,
                poste: r.try_get("poste")?,
                type_entretien: r.try_get("type_entretien")?,
                date_entretien: r.try_get("date_entretien")?,
                lieu: r.try_get("lieu")?,
                modalite: r.try_get("modalite")?,
                issue: r.try_get("issue")?,
                notes: r.try_get("notes")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Compte les entretiens en attente d'une candidature.
///
/// Générique sur l'exécuteur PostgreSQL (pool ou transaction active) pour être
/// appelable à la fois par la commande `get_available_events` (lecture seule,
/// hors transaction) et par `db::timeline_events::add_event` (dans sa
/// transaction), sans dupliquer la requête.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Nombre d'entretiens au statut `en_attente`.
pub async fn count_pending<'e, E>(executor: E, candidature_id: i64) -> Result<i64, AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM search.interviews WHERE candidature_id = $1 AND issue = 'en_attente'",
    )
    .bind(candidature_id)
    .fetch_one(executor)
    .await
    .map_err(map_db_err)?;
    Ok(n)
}

/// Crée un entretien planifié pour une candidature.
///
/// Appelé uniquement depuis `db::timeline_events::add_event` (l'événement
/// `entretien_planifie` est le seul point d'entrée pour créer un entretien),
/// dans la même transaction que l'écriture de la timeline.
///
/// # Arguments
///
/// * `tx` - Transaction PostgreSQL active.
/// * `candidature_id` - Identifiant de la candidature.
/// * `type_entretien` - Type d'entretien (voir `TYPES_ENTRETIEN_VALIDES`).
/// * `date_entretien` - Date/heure de l'entretien, si renseignée.
/// * `lieu` - Lieu de l'entretien, si renseigné.
/// * `modalite` - Modalité de l'entretien (voir `MODALITES_VALIDES`), si renseignée.
/// * `contact_id` - Contact recruteur rencontré, si renseigné.
///
/// # Returns
///
/// Identifiant de l'entretien créé.
pub(crate) async fn insert(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    candidature_id: i64,
    type_entretien: &str,
    date_entretien: Option<DateTime<Utc>>,
    lieu: Option<&str>,
    modalite: Option<&str>,
    contact_id: Option<i64>,
) -> Result<i64, AppError> {
    let row = sqlx::query(
        "INSERT INTO search.interviews (candidature_id, type, date_entretien, lieu, modalite, contact_id) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(candidature_id)
    .bind(type_entretien)
    .bind(date_entretien)
    .bind(lieu)
    .bind(modalite)
    .bind(contact_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_err)?;
    row.try_get("id").map_err(map_db_err)
}

/// Marque un entretien en attente comme passé.
///
/// Appelé uniquement depuis `db::timeline_events::add_event`, dans la même
/// transaction que l'écriture de la timeline.
///
/// # Arguments
///
/// * `tx` - Transaction PostgreSQL active.
/// * `interview_id` - Identifiant de l'entretien.
/// * `candidature_id` - Identifiant de la candidature propriétaire (vérifié
///   pour ne pas marquer l'entretien d'une autre candidature).
/// * `issue` - Issue de l'entretien (voir `ISSUES_VALIDES`).
/// * `notes` - Notes libres sur l'entretien, si renseignées.
///
/// # Returns
///
/// `Ok(())` une fois la mise à jour effectuée, ou `AppError::NotFound` si
/// l'entretien n'existe pas, n'appartient pas à la candidature, ou n'est
/// plus `en_attente`.
pub(crate) async fn mark_passed(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    interview_id: i64,
    candidature_id: i64,
    issue: &str,
    notes: Option<&str>,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE search.interviews SET issue = $1, notes = $2 \
         WHERE id = $3 AND candidature_id = $4 AND issue = 'en_attente'",
    )
    .bind(issue)
    .bind(notes)
    .bind(interview_id)
    .bind(candidature_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Entretien introuvable ou déjà passé.".to_string(),
        ));
    }
    Ok(())
}
