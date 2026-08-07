//! Accès et logique de persistance pour les entretiens rattachés à une
//! candidature. `interviews` n'a pas de colonne `user_id` : la propriété se
//! vérifie via `candidature_id` (comme `db::contacts`), jamais via
//! `db::common::assert_owned_by_user`.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::error::AppError;
use crate::models::InterviewView;

/// Types d'entretien acceptés (voir `interviews.type` en base).
pub const TYPES_ENTRETIEN_VALIDES: [&str; 5] = ["rh", "technique", "manager", "final", "autre"];

/// Issues possibles pour un entretien passé (voir `interviews.issue` en base).
pub const ISSUES_VALIDES: [&str; 2] = ["positif", "negatif"];

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
/// Entretiens au statut `en_attente`, triés par date d'entretien croissante.
pub async fn list_pending_for_candidature(
    pool: &SqlitePool,
    candidature_id: i64,
) -> Result<Vec<InterviewView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, type AS type_entretien, \
                CAST(date_entretien AS TEXT) AS date_entretien, lieu, issue, notes, contact_id \
         FROM interviews \
         WHERE candidature_id = ? AND issue = 'en_attente' \
         ORDER BY date_entretien",
    )
    .bind(candidature_id)
    .fetch_all(pool)
    .await?;

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
        .map_err(AppError::from)
}

/// Compte les entretiens en attente d'une candidature.
///
/// Générique sur l'exécuteur SQLite (pool ou transaction active) pour être
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
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM interviews WHERE candidature_id = ? AND issue = 'en_attente'",
    )
    .bind(candidature_id)
    .fetch_one(executor)
    .await?;
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
/// * `tx` - Transaction SQLite active.
/// * `candidature_id` - Identifiant de la candidature.
/// * `type_entretien` - Type d'entretien (voir `TYPES_ENTRETIEN_VALIDES`).
/// * `date_entretien` - Date/heure de l'entretien, si renseignée.
/// * `lieu` - Lieu de l'entretien, si renseigné.
/// * `contact_id` - Contact recruteur rencontré, si renseigné.
///
/// # Returns
///
/// Identifiant de l'entretien créé.
pub(crate) async fn insert(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    candidature_id: i64,
    type_entretien: &str,
    date_entretien: Option<&str>,
    lieu: Option<&str>,
    contact_id: Option<i64>,
) -> Result<i64, AppError> {
    let row = sqlx::query(
        "INSERT INTO interviews (candidature_id, type, date_entretien, lieu, contact_id) \
         VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(candidature_id)
    .bind(type_entretien)
    .bind(date_entretien)
    .bind(lieu)
    .bind(contact_id)
    .fetch_one(&mut **tx)
    .await?;
    row.try_get("id").map_err(AppError::from)
}

/// Marque un entretien en attente comme passé.
///
/// Appelé uniquement depuis `db::timeline_events::add_event`, dans la même
/// transaction que l'écriture de la timeline.
///
/// # Arguments
///
/// * `tx` - Transaction SQLite active.
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
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    interview_id: i64,
    candidature_id: i64,
    issue: &str,
    notes: Option<&str>,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE interviews SET issue = ?, notes = ? \
         WHERE id = ? AND candidature_id = ? AND issue = 'en_attente'",
    )
    .bind(issue)
    .bind(notes)
    .bind(interview_id)
    .bind(candidature_id)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Entretien introuvable ou déjà passé.".to_string(),
        ));
    }
    Ok(())
}
