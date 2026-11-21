//! Requêtes impliquant des jointures entre tables (`candidatures` ×
//! `sessions`, interlocuteurs des entretiens d'une session).

// Importation des crates externes
use sqlx::{Row, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::db::sessions;
use crate::error::AppError;
use crate::models::{DuplicateCandidatureView, InterviewContactView};

/// Vérifie que la session d'une candidature autorise les modifications
/// (session `active`). Appelée dans la transaction d'écriture de chaque
/// modification d'une candidature (champs, timeline, contacts, postulation,
/// suppression).
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Identifiant de la session de la candidature, `AppError::NotFound` si la
/// candidature n'existe pas, ou `AppError::SessionLocked`.
pub async fn ensure_candidature_session_active<'e, E: SqliteExecutor<'e>>(
    executor: E,
    candidature_id: i64,
) -> Result<i64, AppError> {
    let (session_id, statut): (i64, String) = sqlx::query_as(
        "SELECT c.session_id, s.statut FROM candidatures c JOIN sessions s ON s.id = c.session_id WHERE c.id = $1",
    )
    .bind(candidature_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_err)?
    .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?;
    sessions::check_active(&statut)?;
    Ok(session_id)
}

/// Liste toutes les candidatures qui ont une URL d'offre, toutes sessions
/// confondues, avec le nom de leur session.
///
/// La normalisation des URL est faite en Rust par l'appelant (une seule
/// implémentation, testée) plutôt qu'en SQL.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Paires (URL d'offre brute, candidature), de la plus récente à la plus
/// ancienne.
pub async fn list_candidatures_with_offer_url(
    pool: &SqlitePool,
) -> Result<Vec<(String, DuplicateCandidatureView)>, AppError> {
    let rows = sqlx::query(
        "SELECT c.id, c.session_id, s.nom AS session_nom, c.poste, c.entreprise, c.status, c.url_offre \
         FROM candidatures c \
         JOIN sessions s ON s.id = c.session_id \
         WHERE c.url_offre IS NOT NULL AND trim(c.url_offre) <> '' \
         ORDER BY c.created_at DESC, c.id DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<(String, DuplicateCandidatureView), sqlx::Error> {
            Ok((
                r.try_get("url_offre")?,
                DuplicateCandidatureView {
                    id: r.try_get("id")?,
                    session_id: r.try_get("session_id")?,
                    session_nom: r.try_get("session_nom")?,
                    poste: r.try_get("poste")?,
                    entreprise: r.try_get("entreprise")?,
                    status: r.try_get("status")?,
                },
            ))
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Liste les interlocuteurs des entretiens datés d'une session (mêmes
/// entretiens que `interviews::list_for_session`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Paires (identifiant de l'entretien, interlocuteur), triées par nom.
pub async fn list_interview_contacts_for_session(
    pool: &SqlitePool,
    session_id: i64,
) -> Result<Vec<(i64, InterviewContactView)>, AppError> {
    let rows = sqlx::query(
        "SELECT ic.interview_id, ct.id, ct.nom, ct.role \
         FROM interview_contacts ic \
         JOIN interviews i ON i.id = ic.interview_id \
         JOIN candidatures c ON c.id = i.candidature_id \
         JOIN contacts ct ON ct.id = ic.contact_id \
         WHERE c.session_id = $1 AND i.date_entretien IS NOT NULL \
         ORDER BY ct.nom COLLATE NOCASE, ct.id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<(i64, InterviewContactView), sqlx::Error> {
            Ok((
                r.try_get("interview_id")?,
                InterviewContactView { id: r.try_get("id")?, nom: r.try_get("nom")?, role: r.try_get("role")? },
            ))
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}
