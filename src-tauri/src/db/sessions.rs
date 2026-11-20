//! Accès et logique de persistance pour les sessions de candidature.

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::common::map_db_err;
use crate::db::transaction::begin_write;
use crate::error::AppError;
use crate::models::{
    CreateSessionInput, SessionView, UpdateSessionInput, UpdateSessionListFormatInput, UpdateSessionLocationInput,
};

/// Valeurs acceptées pour `sessions.statut` (miroir de la contrainte CHECK en base).
const VALID_STATUTS: [&str; 3] = ["active", "pause", "termine"];

/// Valeurs acceptées pour `sessions.format_liste` (miroir de la contrainte CHECK en base).
const VALID_LIST_FORMATS: [&str; 2] = ["puce", "tiret"];

/// Seul statut de session qui autorise les modifications : en `pause` ou
/// `termine`, la session est en lecture seule (sauf son statut).
pub const STATUT_ACTIVE: &str = "active";

/// Statut donné automatiquement à une session dont une candidature est acceptée.
pub const STATUT_TERMINE: &str = "termine";

/// Requête agrégée sessions + compteur des candidatures non clôturées,
/// partagée par le listing et la mise à jour d'une session.
const SELECT_WITH_STATS: &str = "SELECT s.id, s.nom, s.statut, s.ville, s.zip_code, s.format_liste, s.created_at, \
            (SELECT COUNT(*) FROM candidatures c \
             WHERE c.session_id = s.id \
               AND c.status NOT IN ('abandon', 'ghosting', 'refus', 'offre_refusee')) AS candidature_count \
     FROM sessions s";

/// Convertit une ligne de `SELECT_WITH_STATS` en `SessionView`.
///
/// # Arguments
///
/// * `row` - Ligne retournée par une requête basée sur `SELECT_WITH_STATS`.
///
/// # Returns
///
/// `SessionView` correspondante, ou erreur si une colonne attendue est absente.
fn row_to_session_view(row: &SqliteRow) -> Result<SessionView, sqlx::Error> {
    Ok(SessionView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        statut: row.try_get("statut")?,
        ville: row.try_get("ville")?,
        zip_code: row.try_get("zip_code")?,
        format_liste: row.try_get("format_liste")?,
        candidature_count: row.try_get("candidature_count")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Valide le nom d'une session.
///
/// # Arguments
///
/// * `nom` - Nom saisi.
///
/// # Returns
///
/// Nom trimé, ou `AppError::Validation` s'il est vide.
fn validate_nom(nom: &str) -> Result<String, AppError> {
    let nom = nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la session est requis.".to_string()));
    }
    Ok(nom)
}

/// Nettoie la localisation saisie : trim, chaîne vide = absent. La ville
/// n'est pas exigée ici (elle l'est à la génération, voir
/// `services::candidatures::create`), le code postal est facultatif.
///
/// # Arguments
///
/// * `ville` - Ville saisie.
/// * `zip_code` - Code postal saisi.
///
/// # Returns
///
/// Couple `(ville, zip_code)` nettoyé.
fn validate_location(ville: Option<&str>, zip_code: Option<&str>) -> (Option<String>, Option<String>) {
    let clean = |value: Option<&str>| value.map(str::trim).filter(|v| !v.is_empty()).map(str::to_string);
    (clean(ville), clean(zip_code))
}

/// Vérifie qu'un statut de session autorise les modifications.
///
/// # Arguments
///
/// * `statut` - Statut courant de la session.
///
/// # Returns
///
/// `Ok(())` si la session est active, sinon `AppError::SessionLocked` avec
/// un message qui dit comment la déverrouiller.
pub fn check_active(statut: &str) -> Result<(), AppError> {
    let label = match statut {
        STATUT_ACTIVE => return Ok(()),
        "pause" => "en pause",
        STATUT_TERMINE => "terminée",
        other => return Err(AppError::Database(format!("Statut de session inconnu : « {other} »."))),
    };
    Err(AppError::SessionLocked(format!(
        "Cette session est {label} : elle est en lecture seule. \
         Repasse-la en « Active » dans ses paramètres pour la modifier."
    )))
}

/// Lit le statut d'une session.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant de la session.
///
/// # Returns
///
/// Statut brut, ou `AppError::NotFound` si la session n'existe pas.
async fn fetch_statut<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<String, AppError> {
    sqlx::query_scalar("SELECT statut FROM sessions WHERE id = $1")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Session introuvable.".to_string()))
}

/// Vérifie qu'une session existe et autorise les modifications. Appelée dans
/// la transaction d'écriture de chaque modification de la session, de ses
/// candidatures ou de ses documents, pour qu'un changement de statut
/// concurrent ne puisse pas s'intercaler.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant de la session.
///
/// # Returns
///
/// `Ok(())`, `AppError::NotFound` ou `AppError::SessionLocked`.
pub async fn ensure_active<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<(), AppError> {
    check_active(&fetch_statut(executor, id).await?)
}

/// Change le statut d'une session sans autre contrôle, dans une transaction
/// en cours (passage automatique à `termine` quand une offre est acceptée).
///
/// # Arguments
///
/// * `executor` - Transaction d'écriture en cours.
/// * `id` - Identifiant de la session.
/// * `statut` - Nouveau statut (une des valeurs de `VALID_STATUTS`).
///
/// # Returns
///
/// `Ok(())`, ou `AppError::NotFound` si la session n'existe pas.
pub async fn set_statut<'e, E: SqliteExecutor<'e>>(executor: E, id: i64, statut: &str) -> Result<(), AppError> {
    debug_assert!(VALID_STATUTS.contains(&statut), "statut de session invalide : {statut}");
    let result = sqlx::query("UPDATE sessions SET statut = $1 WHERE id = $2")
        .bind(statut)
        .bind(id)
        .execute(executor)
        .await
        .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session introuvable.".to_string()));
    }
    Ok(())
}

/// Lit une session avec ses statistiques.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant de la session.
///
/// # Returns
///
/// `SessionView`, ou `AppError::NotFound` si la session n'existe pas.
pub async fn fetch_view<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<SessionView, AppError> {
    let row = sqlx::query(&format!("{SELECT_WITH_STATS} WHERE s.id = $1"))
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Session introuvable.".to_string()))?;
    row_to_session_view(&row).map_err(map_db_err)
}

/// Crée une nouvelle session de candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Nom et localisation (facultative) de la session.
/// * `now` - Horodatage de création.
///
/// # Returns
///
/// `SessionView` fraîchement créée (compteurs à zéro).
pub async fn insert(pool: &SqlitePool, input: &CreateSessionInput, now: &str) -> Result<SessionView, AppError> {
    let nom = validate_nom(&input.nom)?;
    let (ville, zip_code) = validate_location(input.ville.as_deref(), input.zip_code.as_deref());

    let row = sqlx::query(
        "INSERT INTO sessions (nom, ville, zip_code, created_at) VALUES ($1, $2, $3, $4) \
         RETURNING id, nom, statut, ville, zip_code, format_liste, created_at",
    )
    .bind(&nom)
    .bind(&ville)
    .bind(&zip_code)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(SessionView {
        id: row.try_get("id").map_err(map_db_err)?,
        nom: row.try_get("nom").map_err(map_db_err)?,
        statut: row.try_get("statut").map_err(map_db_err)?,
        ville: row.try_get("ville").map_err(map_db_err)?,
        zip_code: row.try_get("zip_code").map_err(map_db_err)?,
        format_liste: row.try_get("format_liste").map_err(map_db_err)?,
        candidature_count: 0,
        created_at: row.try_get("created_at").map_err(map_db_err)?,
    })
}

/// Liste toutes les sessions avec leurs statistiques agrégées.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Sessions triées par date de création décroissante, avec leur statut et
/// le nombre de candidatures non clôturées (tout sauf refus, offre déclinée,
/// abandon et ghosting : une candidature acceptée compte).
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<SessionView>, AppError> {
    let sql = format!("{SELECT_WITH_STATS} ORDER BY s.created_at DESC, s.id DESC");
    let rows = sqlx::query(&sql).fetch_all(pool).await.map_err(map_db_err)?;

    rows.iter().map(row_to_session_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Met à jour le nom et le statut d'une session existante. Le statut se
/// change toujours ; le nom seulement si la session est active avant la
/// mise à jour (une session en pause ou terminée est en lecture seule).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session à modifier.
/// * `input` - Nouvelles valeurs des champs éditables de la session.
///
/// # Returns
///
/// `SessionView` mise à jour, `AppError::NotFound` si la session n'existe
/// pas, ou `AppError::SessionLocked` si le nom change sur une session non active.
pub async fn update(pool: &SqlitePool, id: i64, input: &UpdateSessionInput) -> Result<SessionView, AppError> {
    let nom = validate_nom(&input.nom)?;
    if !VALID_STATUTS.contains(&input.statut.as_str()) {
        return Err(AppError::Validation("Statut de session invalide.".to_string()));
    }

    let mut tx = begin_write(pool).await?;
    let current = fetch_view(&mut *tx, id).await?;
    if nom != current.nom {
        check_active(&current.statut)?;
    }
    sqlx::query("UPDATE sessions SET nom = $1, statut = $2 WHERE id = $3")
        .bind(&nom)
        .bind(&input.statut)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    let view = fetch_view(&mut *tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Met à jour la localisation de l'expéditeur d'une session, sans toucher
/// au nom ni au statut.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session.
/// * `input` - Ville et code postal (trim, chaîne vide = effacé).
///
/// # Returns
///
/// `SessionView` mise à jour, `AppError::NotFound` si la session n'existe
/// pas, ou `AppError::SessionLocked` si elle n'est pas active.
pub async fn update_location(
    pool: &SqlitePool,
    id: i64,
    input: &UpdateSessionLocationInput,
) -> Result<SessionView, AppError> {
    let (ville, zip_code) = validate_location(input.ville.as_deref(), input.zip_code.as_deref());

    let mut tx = begin_write(pool).await?;
    ensure_active(&mut *tx, id).await?;
    let result = sqlx::query("UPDATE sessions SET ville = $1, zip_code = $2 WHERE id = $3")
        .bind(&ville)
        .bind(&zip_code)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session introuvable.".to_string()));
    }
    let view = fetch_view(&mut *tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Met à jour le format des listes des CV d'une session, sans toucher au
/// reste. Comme la localisation, il ne change que sur une session active.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session.
/// * `input` - Nouveau format (`puce` ou `tiret`).
///
/// # Returns
///
/// `SessionView` mise à jour, `AppError::Validation` si le format est
/// inconnu, `AppError::NotFound` si la session n'existe pas, ou
/// `AppError::SessionLocked` si elle n'est pas active.
pub async fn update_list_format(
    pool: &SqlitePool,
    id: i64,
    input: &UpdateSessionListFormatInput,
) -> Result<SessionView, AppError> {
    if !VALID_LIST_FORMATS.contains(&input.format_liste.as_str()) {
        return Err(AppError::Validation(format!(
            "Format de liste inconnu : « {} » (attendu : « puce » ou « tiret »).",
            input.format_liste
        )));
    }

    let mut tx = begin_write(pool).await?;
    ensure_active(&mut *tx, id).await?;
    sqlx::query("UPDATE sessions SET format_liste = $1 WHERE id = $2")
        .bind(&input.format_liste)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    let view = fetch_view(&mut *tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Supprime une session et toutes les données qui en dépendent
/// (candidatures, historique, liaisons aux contacts, entretiens, versions et
/// références de documents ; les contacts restent dans le carnet), par les `ON DELETE CASCADE` du schéma. Les fichiers de la
/// session sont supprimés par l'appelant.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la session à supprimer.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi, `AppError::NotFound` si absente.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session introuvable.".to_string()));
    }
    Ok(())
}
