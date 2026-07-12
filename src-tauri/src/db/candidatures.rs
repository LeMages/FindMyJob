//! Accès et logique de persistance pour les candidatures.

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::normalize;
use crate::error::AppError;
use crate::models::{CandidatureView, CreateCandidatureInput};

const TYPES_CONTRAT_VALIDES: [&str; 6] = ["stage", "alternance", "cdi", "cdd", "interim", "freelance"];

/// Crée une candidature (sans les chemins de documents, renseignés séparément
/// une fois les fichiers finalisés sur le disque — voir `set_documents`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `input` - Données de la candidature à créer.
///
/// # Returns
///
/// `CandidatureView` fraîchement créée (chemins de documents à `None`).
pub async fn insert(
    pool: &SqlitePool,
    user_id: i64,
    input: &CreateCandidatureInput,
) -> Result<CandidatureView, AppError> {
    // ------------------------------------------------------------------
    // 1. Validation des champs requis
    // ------------------------------------------------------------------
    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation("L'intitulé du poste est requis.".to_string()));
    }

    let entreprise = normalize(input.entreprise.clone())
        .ok_or_else(|| AppError::Validation("L'entreprise est requise.".to_string()))?;
    let localisation = normalize(input.localisation.clone())
        .ok_or_else(|| AppError::Validation("Le lieu est requis.".to_string()))?;
    let source = normalize(input.source.clone())
        .ok_or_else(|| AppError::Validation("La plateforme de l'offre est requise.".to_string()))?;
    let description_offre = normalize(input.description_offre.clone())
        .ok_or_else(|| AppError::Validation("Le contenu de l'offre est requis.".to_string()))?;
    let type_contrat = normalize(input.type_contrat.clone())
        .ok_or_else(|| AppError::Validation("Le type de contrat est requis.".to_string()))?;
    let type_contrat_lower = type_contrat.to_lowercase();
    if !TYPES_CONTRAT_VALIDES.contains(&type_contrat_lower.as_str()) {
        return Err(AppError::Validation("Type de contrat invalide.".to_string()));
    }

    // ------------------------------------------------------------------
    // 2. Insertion en base
    // ------------------------------------------------------------------
    let row = sqlx::query(
        "INSERT INTO candidatures \
           (user_id, poste, entreprise, localisation, type, source, remuneration, \
            url_offre, description_offre, notes, session_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, session_id, poste, entreprise, localisation, status, source, \
                   type AS type_contrat, \
                   CAST(date_candidature AS TEXT) AS date_candidature, \
                   CAST(next_relance     AS TEXT) AS next_relance, \
                   created_at",
    )
    .bind(user_id)
    .bind(&poste)
    .bind(&entreprise)
    .bind(&localisation)
    .bind(&type_contrat_lower)
    .bind(&source)
    .bind(normalize(input.remuneration.clone()))
    .bind(normalize(input.url_offre.clone()))
    .bind(&description_offre)
    .bind(normalize(input.notes.clone()))
    .bind(input.session_id)
    .fetch_one(pool)
    .await?;

    // ------------------------------------------------------------------
    // 3. Reconstruction du DTO — cv_path / lettre_motivation_path ne sont pas
    //    dans la clause RETURNING : ils sont renseignés après coup, une fois
    //    les fichiers finalisés (voir set_documents).
    // ------------------------------------------------------------------
    Ok(CandidatureView {
        id: row.try_get("id").map_err(AppError::from)?,
        session_id: row.try_get("session_id").map_err(AppError::from)?,
        poste: row.try_get("poste").map_err(AppError::from)?,
        entreprise: row.try_get("entreprise").map_err(AppError::from)?,
        localisation: row.try_get("localisation").map_err(AppError::from)?,
        status: row.try_get("status").map_err(AppError::from)?,
        source: row.try_get("source").map_err(AppError::from)?,
        type_contrat: row.try_get("type_contrat").map_err(AppError::from)?,
        date_candidature: row.try_get("date_candidature").map_err(AppError::from)?,
        next_relance: row.try_get("next_relance").map_err(AppError::from)?,
        cv_path: None,
        lettre_motivation_path: None,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

/// Renseigne les chemins des documents finalisés (CV / lettre de motivation) d'une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `id` - Identifiant de la candidature.
/// * `cv_path` - Chemin final du CV, si fourni.
/// * `lettre_motivation_path` - Chemin final de la lettre de motivation, si fournie.
///
/// # Returns
///
/// `Ok(())` une fois la mise à jour effectuée.
pub async fn set_documents(
    pool: &SqlitePool,
    id: i64,
    cv_path: &Option<String>,
    lettre_motivation_path: &Option<String>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE candidatures SET cv_path = ?, lettre_motivation_path = ? WHERE id = ?")
        .bind(cv_path)
        .bind(lettre_motivation_path)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Liste les candidatures d'une session, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Candidatures triées par date de création décroissante.
pub async fn list_for_session(
    pool: &SqlitePool,
    user_id: i64,
    session_id: i64,
) -> Result<Vec<CandidatureView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, session_id, poste, entreprise, localisation, status, source, \
                type AS type_contrat, \
                CAST(date_candidature AS TEXT) AS date_candidature, \
                CAST(next_relance     AS TEXT) AS next_relance, \
                cv_path, lettre_motivation_path, \
                created_at \
         FROM candidatures \
         WHERE user_id = ? AND session_id = ? \
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<CandidatureView, sqlx::Error> {
            Ok(CandidatureView {
                id: r.try_get("id")?,
                session_id: r.try_get("session_id")?,
                poste: r.try_get("poste")?,
                entreprise: r.try_get("entreprise")?,
                localisation: r.try_get("localisation")?,
                status: r.try_get("status")?,
                source: r.try_get("source")?,
                type_contrat: r.try_get("type_contrat")?,
                date_candidature: r.try_get("date_candidature")?,
                next_relance: r.try_get("next_relance")?,
                cv_path: r.try_get("cv_path")?,
                lettre_motivation_path: r.try_get("lettre_motivation_path")?,
                created_at: r.try_get("created_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}
