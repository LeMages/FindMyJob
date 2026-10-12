//! Accès et règles de persistance du carnet de contacts recruteur : table
//! `contacts` et sa liaison avec les candidatures (`candidature_contacts`).
//! Un contact existe indépendamment des candidatures et peut être lié à
//! plusieurs ; la validation d'une saisie de contact est faite ici et nulle
//! part ailleurs.

// Importation des modules standard
use std::collections::HashSet;

// Importation des crates externes
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqliteExecutor, SqlitePool};

// Importation des modules internes
use crate::db::candidatures;
use crate::db::common::{map_db_err, normalize};
use crate::db::interviews;
use crate::db::transaction::begin_write;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CandidatureContactInput, ContactInput, ContactView};

/// Colonnes lues pour un `ContactView` (alias `c` sur `contacts`), avec le
/// nombre de candidatures liées.
const VIEW_COLUMNS: &str = "c.id, c.nom, c.role, c.email, c.telephone, c.notes, c.created_at, \
     (SELECT COUNT(*) FROM candidature_contacts l WHERE l.contact_id = c.id) AS candidature_count";

/// Message renvoyé quand un contact n'existe pas.
const CONTACT_INTROUVABLE: &str = "Contact introuvable.";

/// Saisie de contact validée : texte trimé, chaîne vide → `None`, nom présent.
///
/// # Attributes
///
/// * `nom` - Nom du contact, non vide.
/// * `role`, `email`, `telephone`, `notes` - Champs facultatifs normalisés.
#[derive(Debug, Clone)]
pub struct ValidContact {
    pub nom: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub notes: Option<String>,
}

/// Contact à lier à une candidature en création, après validation.
#[derive(Debug, Clone)]
pub enum ContactToLink {
    /// Contact existant du carnet.
    Existing(i64),
    /// Nouveau contact à créer.
    New(ValidContact),
}

/// Valide une saisie de contact.
///
/// # Arguments
///
/// * `input` - Saisie reçue du frontend.
///
/// # Returns
///
/// Contact normalisé, ou `AppError::Validation` si le nom est vide.
pub fn validate(input: &ContactInput) -> Result<ValidContact, AppError> {
    let nom = normalize(Some(input.nom.clone()))
        .ok_or_else(|| AppError::Validation("Le nom du contact est requis.".to_string()))?;
    Ok(ValidContact {
        nom,
        role: normalize(input.role.clone()),
        email: normalize(input.email.clone()),
        telephone: normalize(input.telephone.clone()),
        notes: normalize(input.notes.clone()),
    })
}

/// Valide les contacts saisis avec une nouvelle candidature. Un même
/// contact existant cité deux fois n'est gardé qu'une fois.
///
/// # Arguments
///
/// * `inputs` - Contacts saisis dans le formulaire de création.
///
/// # Returns
///
/// Contacts à lier, dans l'ordre de saisie, ou `AppError::Validation` au
/// premier nouveau contact invalide.
pub fn validate_for_candidature(inputs: &[CandidatureContactInput]) -> Result<Vec<ContactToLink>, AppError> {
    let mut seen = HashSet::new();
    let mut contacts = Vec::with_capacity(inputs.len());
    for input in inputs {
        match input {
            CandidatureContactInput::Existing { contact_id } => {
                if seen.insert(*contact_id) {
                    contacts.push(ContactToLink::Existing(*contact_id));
                }
            }
            CandidatureContactInput::New { contact } => contacts.push(ContactToLink::New(validate(contact)?)),
        }
    }
    Ok(contacts)
}

/// Convertit une ligne lue avec `VIEW_COLUMNS` en `ContactView`.
fn row_to_view(r: &SqliteRow) -> Result<ContactView, sqlx::Error> {
    Ok(ContactView {
        id: r.try_get("id")?,
        nom: r.try_get("nom")?,
        role: r.try_get("role")?,
        email: r.try_get("email")?,
        telephone: r.try_get("telephone")?,
        notes: r.try_get("notes")?,
        candidature_count: r.try_get("candidature_count")?,
        created_at: r.try_get("created_at")?,
    })
}

/// Liste tout le carnet de contacts.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Contacts triés par nom (sans tenir compte de la casse).
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ContactView>, AppError> {
    let sql = format!("SELECT {VIEW_COLUMNS} FROM contacts c ORDER BY c.nom COLLATE NOCASE, c.id");
    let rows = sqlx::query(&sql).fetch_all(pool).await.map_err(map_db_err)?;
    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Liste les contacts liés à une candidature.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Contacts dans l'ordre de leur liaison à la candidature.
pub async fn list_for_candidature<'e, E: SqliteExecutor<'e>>(
    executor: E,
    candidature_id: i64,
) -> Result<Vec<ContactView>, AppError> {
    let sql = format!(
        "SELECT {VIEW_COLUMNS} FROM contacts c \
         JOIN candidature_contacts cc ON cc.contact_id = c.id \
         WHERE cc.candidature_id = $1 \
         ORDER BY cc.created_at, c.id"
    );
    let rows = sqlx::query(&sql).bind(candidature_id).fetch_all(executor).await.map_err(map_db_err)?;
    rows.iter().map(row_to_view).collect::<Result<_, _>>().map_err(map_db_err)
}

/// Lit un contact par identifiant.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `id` - Identifiant du contact.
///
/// # Returns
///
/// Le contact, ou `AppError::NotFound`.
pub async fn fetch<'e, E: SqliteExecutor<'e>>(executor: E, id: i64) -> Result<ContactView, AppError> {
    let sql = format!("SELECT {VIEW_COLUMNS} FROM contacts c WHERE c.id = $1");
    let row = sqlx::query(&sql)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound(CONTACT_INTROUVABLE.to_string()))?;
    row_to_view(&row).map_err(map_db_err)
}

/// Insère un contact validé dans la transaction en cours.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `contact` - Contact validé (voir `validate`).
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// Identifiant du contact créé.
pub async fn insert_in(conn: &mut SqliteConnection, contact: &ValidContact, now: &str) -> Result<i64, AppError> {
    sqlx::query_scalar(
        "INSERT INTO contacts (nom, role, email, telephone, notes, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&contact.nom)
    .bind(&contact.role)
    .bind(&contact.email)
    .bind(&contact.telephone)
    .bind(&contact.notes)
    .bind(now)
    .fetch_one(conn)
    .await
    .map_err(map_db_err)
}

/// Crée un contact dans le carnet, lié à une candidature si elle est donnée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `input` - Saisie du contact.
/// * `candidature_id` - Candidature à laquelle lier le contact, si renseignée.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// Le contact créé, `AppError::Validation` ou `AppError::NotFound` si la
/// candidature n'existe pas.
pub async fn create(
    pool: &SqlitePool,
    input: &ContactInput,
    candidature_id: Option<i64>,
    now: &str,
) -> Result<ContactView, AppError> {
    let contact = validate(input)?;
    let mut tx = begin_write(pool).await?;
    let id = insert_in(&mut tx, &contact, now).await?;
    if let Some(candidature_id) = candidature_id {
        link_in(&mut tx, candidature_id, id, now).await?;
    }
    let view = fetch(&mut *tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(view)
}

/// Modifie un contact, quel que soit le statut des candidatures liées.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant du contact.
/// * `input` - Nouvelle saisie.
///
/// # Returns
///
/// Le contact modifié, `AppError::Validation` ou `AppError::NotFound`.
pub async fn update(pool: &SqlitePool, id: i64, input: &ContactInput) -> Result<ContactView, AppError> {
    let contact = validate(input)?;
    let result = sqlx::query(
        "UPDATE contacts SET nom = $1, role = $2, email = $3, telephone = $4, notes = $5 WHERE id = $6",
    )
    .bind(&contact.nom)
    .bind(&contact.role)
    .bind(&contact.email)
    .bind(&contact.telephone)
    .bind(&contact.notes)
    .bind(id)
    .execute(pool)
    .await
    .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(CONTACT_INTROUVABLE.to_string()));
    }
    fetch(pool, id).await
}

/// Supprime définitivement un contact du carnet. Ses liaisons aux
/// candidatures et aux entretiens partent par `ON DELETE CASCADE`.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant du contact.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::NotFound`.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM contacts WHERE id = $1").bind(id).execute(pool).await.map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(CONTACT_INTROUVABLE.to_string()));
    }
    Ok(())
}

/// Indique si un contact est lié à une candidature.
///
/// # Arguments
///
/// * `executor` - Pool de connexions ou transaction active.
/// * `contact_id` - Identifiant du contact.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// `true` si la liaison existe.
pub async fn is_linked_to_candidature<'e, E: SqliteExecutor<'e>>(
    executor: E,
    contact_id: i64,
    candidature_id: i64,
) -> Result<bool, AppError> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM candidature_contacts WHERE contact_id = $1 AND candidature_id = $2)",
    )
    .bind(contact_id)
    .bind(candidature_id)
    .fetch_one(executor)
    .await
    .map_err(map_db_err)
}

/// Lie un contact existant à une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature.
/// * `contact_id` - Identifiant du contact.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// `Ok(())`, `AppError::NotFound` ou `AppError::Conflict` si déjà lié.
pub async fn link(pool: &SqlitePool, candidature_id: i64, contact_id: i64, now: &str) -> Result<(), AppError> {
    let mut tx = begin_write(pool).await?;
    link_in(&mut tx, candidature_id, contact_id, now).await?;
    tx.commit().await.map_err(map_db_err)
}

/// Lie un contact à une candidature dans la transaction en cours, après
/// avoir vérifié que les deux existent et ne sont pas déjà liés (message
/// clair plutôt que l'erreur brute de la clé primaire).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `candidature_id` - Identifiant de la candidature.
/// * `contact_id` - Identifiant du contact.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// `Ok(())`, `AppError::NotFound` ou `AppError::Conflict`.
pub async fn link_in(conn: &mut SqliteConnection, candidature_id: i64, contact_id: i64, now: &str) -> Result<(), AppError> {
    candidatures::fetch_status(&mut *conn, candidature_id).await?;
    fetch(&mut *conn, contact_id).await?;
    if is_linked_to_candidature(&mut *conn, contact_id, candidature_id).await? {
        return Err(AppError::Conflict("Ce contact est déjà lié à cette candidature.".to_string()));
    }
    sqlx::query("INSERT INTO candidature_contacts (candidature_id, contact_id, created_at) VALUES ($1, $2, $3)")
        .bind(candidature_id)
        .bind(contact_id)
        .bind(now)
        .execute(conn)
        .await
        .map_err(map_db_err)?;
    Ok(())
}

/// Crée et lie les contacts saisis avec une nouvelle candidature, dans la
/// transaction qui crée la candidature.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `candidature_id` - Candidature tout juste insérée.
/// * `contacts` - Contacts validés (voir `validate_for_candidature`).
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::NotFound` si un contact existant a disparu.
pub async fn link_all_in(
    conn: &mut SqliteConnection,
    candidature_id: i64,
    contacts: &[ContactToLink],
    now: &str,
) -> Result<(), AppError> {
    for contact in contacts {
        let contact_id = match contact {
            ContactToLink::Existing(id) => *id,
            ContactToLink::New(valid) => insert_in(&mut *conn, valid, now).await?,
        };
        link_in(&mut *conn, candidature_id, contact_id, now).await?;
    }
    Ok(())
}

/// Retire un contact d'une candidature, et de ses entretiens. Le contact
/// reste dans le carnet.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature.
/// * `contact_id` - Identifiant du contact.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::NotFound` si le contact n'était pas lié.
pub async fn unlink(pool: &SqlitePool, candidature_id: i64, contact_id: i64) -> Result<(), AppError> {
    let mut tx = begin_write(pool).await?;
    interviews::remove_contact_for_candidature(&mut tx, contact_id, candidature_id).await?;
    let result = sqlx::query("DELETE FROM candidature_contacts WHERE candidature_id = $1 AND contact_id = $2")
        .bind(candidature_id)
        .bind(contact_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Ce contact n'est pas lié à cette candidature.".to_string()));
    }
    tx.commit().await.map_err(map_db_err)
}
