//! Accès et logique de persistance pour les contacts recruteur.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::ContactView;

/// Liste les contacts recruteur d'une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Contacts triés par date de création croissante.
pub async fn list_for_candidature(
    pool: &PgPool,
    candidature_id: i64,
) -> Result<Vec<ContactView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, candidature_id, nom, role, email, telephone, notes, created_at \
         FROM search.contacts \
         WHERE candidature_id = $1 \
         ORDER BY created_at",
    )
    .bind(candidature_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<ContactView, sqlx::Error> {
            Ok(ContactView {
                id: r.try_get("id")?,
                candidature_id: r.try_get("candidature_id")?,
                nom: r.try_get("nom")?,
                role: r.try_get("role")?,
                email: r.try_get("email")?,
                telephone: r.try_get("telephone")?,
                notes: r.try_get("notes")?,
                created_at: r.try_get("created_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}
