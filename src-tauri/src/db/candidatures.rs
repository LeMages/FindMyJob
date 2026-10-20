//! Accès et logique de persistance pour les candidatures.
//!
//! Les champs reçus sont déjà validés et normalisés par
//! `services::candidature_validation` (`CandidatureFields`).

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::db::contacts::{self, ContactToLink};
use crate::db::status_history::{self, NewHistoryEntry};
use crate::db::transaction::begin_write;
use crate::error::AppError;
use crate::models::{CandidatureDetail, CandidatureView, EventType};

/// Champs éditables d'une candidature, validés et normalisés.
///
/// # Attributes
///
/// * `poste` - Intitulé du poste.
/// * `entreprise` - Entreprise.
/// * `localisation` - Lieu.
/// * `type_contrat` - Type de contrat, en minuscules.
/// * `source` - Plateforme de l'offre.
/// * `remuneration` - Rémunération, si renseignée.
/// * `url_offre` - Lien de l'offre, si renseigné.
/// * `notes` - Notes libres, si renseignées.
pub struct CandidatureFields {
    pub poste: String,
    pub entreprise: String,
    pub localisation: String,
    pub type_contrat: String,
    pub source: String,
    pub remuneration: Option<String>,
    pub url_offre: Option<String>,
    pub notes: Option<String>,
}

/// Colonnes de `CandidatureDetail`, versions de référence résolues en numéro
/// d'affichage par sous-requête.
const DETAIL_COLUMNS: &str = "id, session_id, poste, entreprise, localisation, status, source, type_contrat, \
     remuneration, url_offre, description_offre, notes, cv_path, lettre_motivation_path, created_at, generation_warnings, \
     (SELECT version FROM document_versions WHERE id = cv_reference_version_id) AS cv_reference_version, \
     (SELECT version FROM document_versions WHERE id = lm_reference_version_id) AS lm_reference_version";

/// Construit un `CandidatureDetail` à partir d'une ligne sélectionnant `DETAIL_COLUMNS`.
///
/// # Arguments
///
/// * `row` - Ligne SQL brute.
///
/// # Returns
///
/// `CandidatureDetail` reconstruite, ou erreur si une colonne attendue est absente.
fn row_to_detail(row: &SqliteRow) -> Result<CandidatureDetail, sqlx::Error> {
    Ok(CandidatureDetail {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        poste: row.try_get("poste")?,
        entreprise: row.try_get("entreprise")?,
        localisation: row.try_get("localisation")?,
        status: row.try_get("status")?,
        source: row.try_get("source")?,
        type_contrat: row.try_get("type_contrat")?,
        remuneration: row.try_get("remuneration")?,
        url_offre: row.try_get("url_offre")?,
        description_offre: row.try_get("description_offre")?,
        notes: row.try_get("notes")?,
        cv_path: row.try_get("cv_path")?,
        lettre_motivation_path: row.try_get("lettre_motivation_path")?,
        cv_reference_version: row.try_get("cv_reference_version")?,
        lm_reference_version: row.try_get("lm_reference_version")?,
        generation_warnings: warnings_from_json(&row.try_get::<String, _>("generation_warnings")?),
        created_at: row.try_get("created_at")?,
    })
}

/// Lit la liste JSON des avertissements ; une valeur illisible (écrite à la
/// main) est rendue telle quelle plutôt que perdue.
fn warnings_from_json(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_else(|_| vec![raw.to_string()])
}

/// Construit un `CandidatureView` à partir d'une ligne de liste.
///
/// # Arguments
///
/// * `row` - Ligne SQL brute (voir `list_for_session`).
///
/// # Returns
///
/// `CandidatureView` reconstruite, ou erreur si une colonne attendue est absente.
fn row_to_view(row: &SqliteRow) -> Result<CandidatureView, sqlx::Error> {
    Ok(CandidatureView {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        poste: row.try_get("poste")?,
        entreprise: row.try_get("entreprise")?,
        localisation: row.try_get("localisation")?,
        status: row.try_get("status")?,
        source: row.try_get("source")?,
        type_contrat: row.try_get("type_contrat")?,
        derniere_maj_statut: row.try_get("derniere_maj_statut")?,
        cv_path: row.try_get("cv_path")?,
        lettre_motivation_path: row.try_get("lettre_motivation_path")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Lit le statut d'une candidature dans une transaction en cours.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// Statut brut, ou `AppError::NotFound` si la candidature n'existe pas.
pub async fn fetch_status<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<String, AppError> {
    sqlx::query_scalar("SELECT status FROM candidatures WHERE id = $1")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))
}

/// Vérifie qu'une candidature est encore au statut « à postuler ».
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `id` - Identifiant de la candidature.
/// * `conflict_message` - Message renvoyé si le statut ne convient plus.
///
/// # Returns
///
/// Statut courant (`a_postuler`), `AppError::NotFound` ou `AppError::Conflict`.
async fn ensure_a_postuler(conn: &mut SqliteConnection, id: i64, conflict_message: &str) -> Result<String, AppError> {
    let status = fetch_status(&mut *conn, id).await?;
    if status != "a_postuler" {
        return Err(AppError::Conflict(conflict_message.to_string()));
    }
    Ok(status)
}

/// Crée une candidature, son entrée d'historique `creation` et ses contacts
/// (nouveaux contacts créés, contacts du carnet liés), dans une même
/// transaction et au même horodatage — la candidature ne doit jamais exister
/// sans que son passage au statut « à postuler » soit tracé, et un contact
/// en échec annule toute la création. Les chemins de documents sont
/// renseignés ensuite par `set_documents`.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session de rattachement.
/// * `fields` - Champs validés.
/// * `description_offre` - Contenu de l'offre, validé.
/// * `contacts` - Contacts à lier, validés.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// `CandidatureView` fraîchement créée (chemins de documents à `None`), ou
/// `AppError::NotFound` si un contact du carnet n'existe plus.
pub async fn insert(
    pool: &SqlitePool,
    session_id: i64,
    fields: &CandidatureFields,
    description_offre: &str,
    contacts: &[ContactToLink],
    now: &str,
) -> Result<CandidatureView, AppError> {
    let mut tx = begin_write(pool).await?;

    let row = sqlx::query(
        "INSERT INTO candidatures \
           (poste, entreprise, localisation, type_contrat, source, remuneration, \
            url_offre, description_offre, notes, session_id, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
         RETURNING id, session_id, poste, entreprise, localisation, status, source, type_contrat, \
                   created_at AS derniere_maj_statut, cv_path, lettre_motivation_path, created_at",
    )
    .bind(&fields.poste)
    .bind(&fields.entreprise)
    .bind(&fields.localisation)
    .bind(&fields.type_contrat)
    .bind(&fields.source)
    .bind(&fields.remuneration)
    .bind(&fields.url_offre)
    .bind(description_offre)
    .bind(&fields.notes)
    .bind(session_id)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_err)?;
    // `derniere_maj_statut` = `created_at` : l'entrée `creation` ci-dessous
    // reçoit le même horodatage `now`.
    let view = row_to_view(&row).map_err(map_db_err)?;

    let entry = NewHistoryEntry {
        candidature_id: view.id,
        event_type: EventType::Creation,
        ancien_status: None,
        nouveau_status: Some(&view.status),
        content: None,
        interview_id: None,
    };
    status_history::record(&mut tx, &entry, now).await?;
    contacts::link_all_in(&mut tx, view.id, contacts, now).await?;

    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Met à jour les champs éditables d'une candidature, uniquement autorisé
/// tant que son statut est `a_postuler` — au-delà, la candidature est
/// considérée comme engagée et ne doit plus être modifiée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la candidature à modifier.
/// * `fields` - Nouvelles valeurs validées.
///
/// # Returns
///
/// `CandidatureDetail` mise à jour, `AppError::NotFound` si la candidature
/// n'existe pas, ou `AppError::Conflict` si son statut ne permet plus l'édition.
pub async fn update(pool: &SqlitePool, id: i64, fields: &CandidatureFields) -> Result<CandidatureDetail, AppError> {
    let mut tx = begin_write(pool).await?;
    ensure_a_postuler(&mut tx, id, "Seule une candidature au statut « À postuler » peut être modifiée.").await?;

    sqlx::query(
        "UPDATE candidatures \
         SET poste = $1, entreprise = $2, localisation = $3, type_contrat = $4, source = $5, \
             remuneration = $6, url_offre = $7, notes = $8 \
         WHERE id = $9",
    )
    .bind(&fields.poste)
    .bind(&fields.entreprise)
    .bind(&fields.localisation)
    .bind(&fields.type_contrat)
    .bind(&fields.source)
    .bind(&fields.remuneration)
    .bind(&fields.url_offre)
    .bind(&fields.notes)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    let detail = fetch_detail(&mut *tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(detail)
}

/// Renseigne les chemins des documents générés (CV / lettre de motivation)
/// d'une candidature, avec les versions de référence utilisées.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la candidature.
/// * `paths` - Chemins (CV, lettre), relatifs au dossier de données.
/// * `reference_versions` - Ids des versions de référence (CV, lettre) ayant
///   servi à la génération.
/// * `warnings` - Avertissements de la génération, pour l'utilisateur.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::NotFound` si la candidature n'existe plus.
pub async fn set_documents(
    pool: &SqlitePool,
    id: i64,
    paths: (&str, &str),
    reference_versions: (i64, i64),
    warnings: &[String],
) -> Result<(), AppError> {
    let warnings = serde_json::to_string(warnings)
        .map_err(|e| AppError::Validation(format!("Avertissements non sérialisables : {e}")))?;
    let result = sqlx::query(
        "UPDATE candidatures SET cv_path = $1, lettre_motivation_path = $2, \
         cv_reference_version_id = $3, lm_reference_version_id = $4, generation_warnings = $5 WHERE id = $6",
    )
    .bind(paths.0)
    .bind(paths.1)
    .bind(reference_versions.0)
    .bind(reference_versions.1)
    .bind(warnings)
    .bind(id)
    .execute(pool)
    .await
    .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Candidature introuvable.".to_string()));
    }
    Ok(())
}

/// Supprime définitivement une candidature, sans condition de statut.
///
/// Usage restreint : génération échouée juste après la création, ou
/// génération interrompue retrouvée au démarrage. Le bouton « Supprimer »
/// de la page de détail passe par `delete_a_postuler`, qui vérifie le statut.
/// Historique, liaisons aux contacts et entretiens partent par
/// `ON DELETE CASCADE` ; les contacts restent dans le carnet.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la candidature à supprimer.
///
/// # Returns
///
/// Identifiant de session de la candidature supprimée, `None` si elle
/// n'existait déjà plus.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<Option<i64>, AppError> {
    sqlx::query_scalar("DELETE FROM candidatures WHERE id = $1 RETURNING session_id")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_err)
}

/// Supprime définitivement une candidature encore au statut « à postuler »
/// (bouton « Supprimer » de la page de détail). La transaction immédiate
/// empêche un « Postuler » concurrent de s'intercaler entre le contrôle et
/// la suppression.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la candidature à supprimer.
///
/// # Returns
///
/// Identifiant de la session de la candidature supprimée (pour le nettoyage
/// de ses documents sur disque), `AppError::NotFound` si elle n'existe pas,
/// ou `AppError::Conflict` si elle n'est plus au statut « à postuler ».
pub async fn delete_a_postuler(pool: &SqlitePool, id: i64) -> Result<i64, AppError> {
    let mut tx = begin_write(pool).await?;
    ensure_a_postuler(&mut tx, id, "Seule une candidature au statut « À postuler » peut être supprimée.").await?;

    let session_id: i64 = sqlx::query_scalar("DELETE FROM candidatures WHERE id = $1 RETURNING session_id")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_err)?;

    tx.commit().await.map_err(map_db_err)?;
    Ok(session_id)
}

/// Marque une candidature comme postulée (statut `postule`), une fois la
/// candidature effectivement envoyée. Le changement de statut et son entrée
/// d'historique sont écrits dans la même transaction, au même horodatage.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la candidature.
/// * `content` - Message envoyé, `None` si postulée sans message.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// `Ok(())` une fois le changement effectué, `AppError::NotFound` si la
/// candidature n'existe pas, ou `AppError::Conflict` si elle n'est plus au
/// statut « à postuler ».
pub async fn mark_postule(pool: &SqlitePool, id: i64, content: Option<&str>, now: &str) -> Result<(), AppError> {
    let mut tx = begin_write(pool).await?;
    let ancien_status = ensure_a_postuler(
        &mut tx,
        id,
        "Seule une candidature au statut « À postuler » peut être marquée comme postulée.",
    )
    .await?;

    sqlx::query("UPDATE candidatures SET status = 'postule' WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;

    let entry = NewHistoryEntry {
        candidature_id: id,
        event_type: EventType::CandidatureEnvoyee,
        ancien_status: Some(&ancien_status),
        nouveau_status: Some("postule"),
        content,
        interview_id: None,
    };
    status_history::record(&mut tx, &entry, now).await?;

    tx.commit().await.map_err(map_db_err)?;
    Ok(())
}

/// Liste les candidatures d'une session, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Candidatures triées par date de création décroissante.
pub async fn list_for_session(pool: &SqlitePool, session_id: i64) -> Result<Vec<CandidatureView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, session_id, poste, entreprise, localisation, status, source, type_contrat, \
                COALESCE( \
                    (SELECT MAX(changed_at) FROM status_history \
                     WHERE status_history.candidature_id = candidatures.id), \
                    created_at \
                ) AS derniere_maj_statut, \
                cv_path, lettre_motivation_path, created_at \
         FROM candidatures \
         WHERE session_id = $1 \
         ORDER BY created_at DESC, id DESC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Récupère le détail complet d'une candidature (page de détail).
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `CandidatureDetail` correspondante, ou `AppError::NotFound` si elle n'existe pas.
pub async fn fetch_detail<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<CandidatureDetail, AppError> {
    let row = sqlx::query(&format!("SELECT {DETAIL_COLUMNS} FROM candidatures WHERE id = $1"))
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?;
    row_to_detail(&row).map_err(map_db_err)
}

/// Liste les générations interrompues : candidatures au statut « à
/// postuler » sans aucun document (l'application a été fermée pendant la
/// génération, qui n'a ni abouti ni été nettoyée).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Paires (identifiant de candidature, identifiant de session).
pub async fn list_interrupted_generations(pool: &SqlitePool) -> Result<Vec<(i64, i64)>, AppError> {
    sqlx::query_as(
        "SELECT id, session_id FROM candidatures \
         WHERE status = 'a_postuler' AND cv_path IS NULL AND lettre_motivation_path IS NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_err)
}
