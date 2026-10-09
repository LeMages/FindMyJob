//! Requêtes impliquant des jointures entre tables (`candidatures` ×
//! `sessions`).

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;
use findmyjob_core::models::DuplicateCandidatureView;

/// Liste toutes les candidatures qui ont une URL d'offre, toutes sessions
/// confondues, avec le nom de leur session.
///
/// La normalisation des URL est faite en Rust par l'appelant (une seule
/// implémentation, testée) plutôt qu'en SQL.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
///
/// # Returns
///
/// Paires (URL d'offre brute, candidature), de la plus récente à la plus
/// ancienne.
pub async fn list_candidatures_with_offer_url(
    pool: &PgPool,
) -> Result<Vec<(String, DuplicateCandidatureView)>, AppError> {
    let rows = sqlx::query(
        "SELECT c.id, c.session_id, s.nom AS session_nom, c.poste, c.entreprise, c.status, c.url_offre \
         FROM search.candidatures c \
         JOIN search.sessions s ON s.id = c.session_id \
         WHERE c.url_offre IS NOT NULL AND btrim(c.url_offre) <> '' \
         ORDER BY c.created_at DESC",
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
