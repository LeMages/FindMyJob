//! Accès et logique de persistance pour les entretiens rattachés à une
//! candidature.

// Importation des modules standard
use std::collections::HashMap;

// Importation des crates externes
use sqlx::{Row, SqliteConnection, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::db::joins;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{InterviewContactView, InterviewView, PlanningInterviewView};

/// Types d'entretien acceptés (voir `interviews.type_entretien` en base).
pub const TYPES_ENTRETIEN_VALIDES: [&str; 5] = ["rh", "technique", "manager", "final", "autre"];

/// Issues possibles pour un entretien passé (voir `interviews.issue` en base).
pub const ISSUES_VALIDES: [&str; 2] = ["positif", "negatif"];

/// Modalités d'entretien acceptées (voir `interviews.modalite` en base).
pub const MODALITES_VALIDES: [&str; 3] = ["visio", "presentiel", "telephone"];

/// Entretien à créer.
///
/// # Attributes
///
/// * `candidature_id` - Identifiant de la candidature.
/// * `type_entretien` - Type d'entretien (voir `TYPES_ENTRETIEN_VALIDES`).
/// * `date_entretien` - Date/heure de l'entretien au format de stockage, si renseignée.
/// * `lieu` - Lieu de l'entretien, si renseigné.
/// * `modalite` - Modalité (voir `MODALITES_VALIDES`), si renseignée.
/// * `contact_ids` - Interlocuteurs, déjà vérifiés comme liés à la candidature.
pub struct NewInterview<'a> {
    pub candidature_id: i64,
    pub type_entretien: &'a str,
    pub date_entretien: Option<&'a str>,
    pub lieu: Option<&'a str>,
    pub modalite: Option<&'a str>,
    pub contact_ids: &'a [i64],
}

/// Liste les entretiens encore en attente d'une candidature — utilisé pour
/// proposer à l'utilisateur lequel marquer comme passé.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Entretiens au statut `en_attente`, triés par date d'entretien croissante
/// (sans date en dernier, comme sous PostgreSQL).
pub async fn list_pending_for_candidature(
    pool: &SqlitePool,
    candidature_id: i64,
) -> Result<Vec<InterviewView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, type_entretien, date_entretien, lieu, issue, notes \
         FROM interviews \
         WHERE candidature_id = $1 AND issue = 'en_attente' \
         ORDER BY date_entretien NULLS LAST, id",
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
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Entretiens dont la date est renseignée, triés par date croissante, avec
/// leurs interlocuteurs.
pub async fn list_for_session(pool: &SqlitePool, session_id: i64) -> Result<Vec<PlanningInterviewView>, AppError> {
    let rows = sqlx::query(
        "SELECT i.id, i.candidature_id, c.entreprise, c.poste, i.type_entretien, \
                i.date_entretien, i.lieu, i.modalite, i.issue, i.notes \
         FROM interviews i \
         JOIN candidatures c ON c.id = i.candidature_id \
         WHERE c.session_id = $1 AND i.date_entretien IS NOT NULL \
         ORDER BY i.date_entretien, i.id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    let mut contacts_by_interview: HashMap<i64, Vec<InterviewContactView>> = HashMap::new();
    for (interview_id, contact) in joins::list_interview_contacts_for_session(pool, session_id).await? {
        contacts_by_interview.entry(interview_id).or_default().push(contact);
    }

    rows.iter()
        .map(|r| -> Result<PlanningInterviewView, sqlx::Error> {
            let id: i64 = r.try_get("id")?;
            Ok(PlanningInterviewView {
                id,
                candidature_id: r.try_get("candidature_id")?,
                entreprise: r.try_get("entreprise")?,
                poste: r.try_get("poste")?,
                type_entretien: r.try_get("type_entretien")?,
                date_entretien: r.try_get("date_entretien")?,
                lieu: r.try_get("lieu")?,
                modalite: r.try_get("modalite")?,
                issue: r.try_get("issue")?,
                notes: r.try_get("notes")?,
                contacts: contacts_by_interview.remove(&id).unwrap_or_default(),
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Compte les entretiens en attente d'une candidature.
///
/// Générique sur l'exécuteur (pool ou transaction active) pour servir à la
/// fois la lecture des événements disponibles et `timeline_events::add_event`.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Nombre d'entretiens au statut `en_attente`.
pub async fn count_pending<'e, E: SqliteExecutor<'e>>(executor: E, candidature_id: i64) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT COUNT(*) FROM interviews WHERE candidature_id = $1 AND issue = 'en_attente'")
        .bind(candidature_id)
        .fetch_one(executor)
        .await
        .map_err(map_db_err)
}

/// Crée un entretien planifié pour une candidature, avec ses interlocuteurs.
///
/// Appelé uniquement depuis `db::timeline_events::add_event` (l'événement
/// `entretien_planifie` est le seul point d'entrée pour créer un entretien),
/// dans la même transaction que l'écriture de la timeline.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `new` - Entretien à créer.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// Identifiant de l'entretien créé.
pub(crate) async fn insert(conn: &mut SqliteConnection, new: &NewInterview<'_>, now: &str) -> Result<i64, AppError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO interviews \
           (candidature_id, type_entretien, date_entretien, lieu, modalite, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(new.candidature_id)
    .bind(new.type_entretien)
    .bind(new.date_entretien)
    .bind(new.lieu)
    .bind(new.modalite)
    .bind(now)
    .fetch_one(&mut *conn)
    .await
    .map_err(map_db_err)?;

    for contact_id in new.contact_ids {
        sqlx::query("INSERT INTO interview_contacts (interview_id, contact_id) VALUES ($1, $2)")
            .bind(id)
            .bind(contact_id)
            .execute(&mut *conn)
            .await
            .map_err(map_db_err)?;
    }
    Ok(id)
}

/// Retire un contact des entretiens d'une candidature — appelé quand le
/// contact est retiré de la candidature (`contacts::unlink`).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `contact_id` - Identifiant du contact.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// `Ok(())` une fois les liaisons supprimées (n'en trouver aucune n'est pas
/// une erreur).
pub(crate) async fn remove_contact_for_candidature(
    conn: &mut SqliteConnection,
    contact_id: i64,
    candidature_id: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM interview_contacts \
         WHERE contact_id = $1 \
           AND interview_id IN (SELECT id FROM interviews WHERE candidature_id = $2)",
    )
    .bind(contact_id)
    .bind(candidature_id)
    .execute(conn)
    .await
    .map_err(map_db_err)?;
    Ok(())
}

/// Marque un entretien en attente comme passé.
///
/// Appelé uniquement depuis `db::timeline_events::add_event`, dans la même
/// transaction que l'écriture de la timeline.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
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
    conn: &mut SqliteConnection,
    interview_id: i64,
    candidature_id: i64,
    issue: &str,
    notes: Option<&str>,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE interviews SET issue = $1, notes = $2 \
         WHERE id = $3 AND candidature_id = $4 AND issue = 'en_attente'",
    )
    .bind(issue)
    .bind(notes)
    .bind(interview_id)
    .bind(candidature_id)
    .execute(conn)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Entretien introuvable ou déjà passé.".to_string()));
    }
    Ok(())
}
