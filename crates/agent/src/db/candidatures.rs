//! Accès et logique de persistance pour les candidatures.

// Importation des crates externes
use sqlx::{PgPool, Row};

// Importation des modules internes
use crate::db::common::{map_db_err, normalize};
use crate::db::status_history;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{
    CandidatureDetail, CandidatureView, CreateCandidatureInput, EventType, UpdateCandidatureInput,
};

const TYPES_CONTRAT_VALIDES: [&str; 6] = ["stage", "alternance", "cdi", "cdd", "interim", "freelance"];

/// Crée une candidature (sans les chemins de documents, renseignés séparément
/// une fois les fichiers finalisés sur le disque — voir `set_documents`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `input` - Données de la candidature à créer.
///
/// # Returns
///
/// `CandidatureView` fraîchement créée (chemins de documents à `None`).
pub async fn insert(
    pool: &PgPool,
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
    // 2. Insertion en base et entrée initiale d'historique, dans une même
    //    transaction — la candidature ne doit jamais exister sans que son
    //    passage au statut « à postuler » soit tracé.
    // ------------------------------------------------------------------
    let mut tx = pool.begin().await.map_err(map_db_err)?;

    let row = sqlx::query(
        "INSERT INTO search.candidatures \
           (poste, entreprise, localisation, type, source, remuneration, \
            url_offre, description_offre, notes, session_id, offer_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
         RETURNING id, session_id, offer_id, poste, entreprise, localisation, status, source, \
                   type AS type_contrat, date_candidature, created_at",
    )
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
    .bind(input.offer_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_err)?;

    // Reconstruction du DTO — cv_path / lettre_motivation_path ne sont pas
    // dans la clause RETURNING : ils sont renseignés après coup, une fois
    // les fichiers finalisés (voir set_documents).
    let view = CandidatureView {
        id: row.try_get("id").map_err(map_db_err)?,
        session_id: row.try_get("session_id").map_err(map_db_err)?,
        offer_id: row.try_get("offer_id").map_err(map_db_err)?,
        poste: row.try_get("poste").map_err(map_db_err)?,
        entreprise: row.try_get("entreprise").map_err(map_db_err)?,
        localisation: row.try_get("localisation").map_err(map_db_err)?,
        status: row.try_get("status").map_err(map_db_err)?,
        source: row.try_get("source").map_err(map_db_err)?,
        type_contrat: row.try_get("type_contrat").map_err(map_db_err)?,
        date_candidature: row.try_get("date_candidature").map_err(map_db_err)?,
        // À la création, l'unique entrée d'historique (insérée juste après)
        // partage le même horodatage que `created_at`.
        derniere_maj_statut: row.try_get("created_at").map_err(map_db_err)?,
        cv_path: None,
        lettre_motivation_path: None,
        created_at: row.try_get("created_at").map_err(map_db_err)?,
    };

    status_history::record(
        &mut tx,
        view.id,
        EventType::Creation,
        None,
        Some(&view.status),
        None,
        None,
    )
    .await?;

    // Candidature créée depuis une offre du Marché (bouton "Postuler") :
    // l'offre passe au statut "applied" dans la même transaction, pour ne
    // jamais désynchroniser candidature et offre source.
    if let Some(offer_id) = input.offer_id {
        crate::db::offers::mark_applied_tx(&mut tx, offer_id).await?;
    }

    tx.commit().await.map_err(map_db_err)?;

    Ok(view)
}

/// Met à jour les champs éditables d'une candidature (détails du poste et
/// notes), uniquement autorisé tant que son statut est `a_postuler` — au-delà,
/// la candidature est considérée comme engagée et ne doit plus être modifiée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature à modifier.
/// * `input` - Nouvelles valeurs des champs éditables.
///
/// # Returns
///
/// `CandidatureDetail` mise à jour, `AppError::NotFound` si la candidature
/// n'existe pas, ou `AppError::Conflict` si son statut ne permet plus l'édition.
pub async fn update(
    pool: &PgPool,
    id: i64,
    input: &UpdateCandidatureInput,
) -> Result<CandidatureDetail, AppError> {
    // ------------------------------------------------------------------
    // 1. La candidature doit exister et être encore au statut « à postuler »
    // ------------------------------------------------------------------
    let status: String = sqlx::query("SELECT status FROM search.candidatures WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?
        .try_get("status")
        .map_err(map_db_err)?;

    if status != "a_postuler" {
        return Err(AppError::Conflict(
            "Seule une candidature au statut « À postuler » peut être modifiée.".to_string(),
        ));
    }

    // ------------------------------------------------------------------
    // 2. Validation des champs requis (mêmes règles qu'à la création)
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
    let type_contrat = normalize(input.type_contrat.clone())
        .ok_or_else(|| AppError::Validation("Le type de contrat est requis.".to_string()))?;
    let type_contrat_lower = type_contrat.to_lowercase();
    if !TYPES_CONTRAT_VALIDES.contains(&type_contrat_lower.as_str()) {
        return Err(AppError::Validation("Type de contrat invalide.".to_string()));
    }

    // ------------------------------------------------------------------
    // 3. Mise à jour en base
    // ------------------------------------------------------------------
    let row = sqlx::query(
        "UPDATE search.candidatures \
         SET poste = $1, entreprise = $2, localisation = $3, type = $4, source = $5, \
             remuneration = $6, url_offre = $7, notes = $8 \
         WHERE id = $9 \
         RETURNING id, session_id, offer_id, poste, entreprise, localisation, status, source, \
                   type AS type_contrat, remuneration, url_offre, description_offre, notes, \
                   date_candidature, cv_path, lettre_motivation_path, created_at, \
                   (SELECT version FROM search.document_versions WHERE id = cv_reference_version_id) AS cv_reference_version, \
                   (SELECT version FROM search.document_versions WHERE id = lm_reference_version_id) AS lm_reference_version",
    )
    .bind(&poste)
    .bind(&entreprise)
    .bind(&localisation)
    .bind(&type_contrat_lower)
    .bind(&source)
    .bind(normalize(input.remuneration.clone()))
    .bind(normalize(input.url_offre.clone()))
    .bind(normalize(input.notes.clone()))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Ok(CandidatureDetail {
        id: row.try_get("id").map_err(map_db_err)?,
        session_id: row.try_get("session_id").map_err(map_db_err)?,
        offer_id: row.try_get("offer_id").map_err(map_db_err)?,
        poste: row.try_get("poste").map_err(map_db_err)?,
        entreprise: row.try_get("entreprise").map_err(map_db_err)?,
        localisation: row.try_get("localisation").map_err(map_db_err)?,
        status: row.try_get("status").map_err(map_db_err)?,
        source: row.try_get("source").map_err(map_db_err)?,
        type_contrat: row.try_get("type_contrat").map_err(map_db_err)?,
        remuneration: row.try_get("remuneration").map_err(map_db_err)?,
        url_offre: row.try_get("url_offre").map_err(map_db_err)?,
        description_offre: row.try_get("description_offre").map_err(map_db_err)?,
        notes: row.try_get("notes").map_err(map_db_err)?,
        date_candidature: row.try_get("date_candidature").map_err(map_db_err)?,
        cv_path: row.try_get("cv_path").map_err(map_db_err)?,
        lettre_motivation_path: row.try_get("lettre_motivation_path").map_err(map_db_err)?,
        cv_reference_version: row.try_get("cv_reference_version").map_err(map_db_err)?,
        lm_reference_version: row.try_get("lm_reference_version").map_err(map_db_err)?,
        created_at: row.try_get("created_at").map_err(map_db_err)?,
    })
}

/// Renseigne les chemins des documents finalisés (CV / lettre de motivation)
/// d'une candidature, avec les versions de référence utilisées pour les générer.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature.
/// * `cv_path` - Chemin final du CV, si fourni.
/// * `lettre_motivation_path` - Chemin final de la lettre de motivation, si fournie.
/// * `reference_versions` - Ids des versions de référence (CV, lettre) ayant
///   servi à la génération, `None` pour une référence antérieure au versioning.
///
/// # Returns
///
/// `Ok(())` une fois la mise à jour effectuée.
pub async fn set_documents(
    pool: &PgPool,
    id: i64,
    cv_path: &Option<String>,
    lettre_motivation_path: &Option<String>,
    reference_versions: (Option<i64>, Option<i64>),
) -> Result<(), AppError> {
    let (cv_version_id, lm_version_id) = reference_versions;
    sqlx::query(
        "UPDATE search.candidatures SET cv_path = $1, lettre_motivation_path = $2, \
         cv_reference_version_id = $3, lm_reference_version_id = $4 WHERE id = $5",
    )
    .bind(cv_path)
    .bind(lettre_motivation_path)
    .bind(cv_version_id)
    .bind(lm_version_id)
    .bind(id)
    .execute(pool)
    .await
    .map_err(map_db_err)?;
    Ok(())
}

/// Supprime définitivement une candidature, sans condition de statut.
///
/// Usage restreint : appelé uniquement par
/// `api::candidatures::spawn_document_generation` quand la génération des
/// documents échoue juste après la création. Le bouton « Supprimer » de la
/// page de détail passe par `delete_a_postuler`, qui vérifie le statut.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature à supprimer.
///
/// # Returns
///
/// `Ok(())` une fois la suppression effectuée (silencieux si la candidature n'existe déjà plus).
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(map_db_err)?;
    delete_in_tx(&mut tx, id).await?;
    tx.commit().await.map_err(map_db_err)?;
    Ok(())
}

/// Supprime définitivement une candidature encore au statut « à postuler »
/// (bouton « Supprimer » de la page de détail). Au-delà de ce statut, la
/// candidature est engagée et passe par la corbeille (désistement, refus...),
/// jamais par une suppression.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature à supprimer.
///
/// # Returns
///
/// Identifiant de la session de la candidature supprimée (pour le nettoyage
/// de ses documents sur disque), `AppError::NotFound` si elle n'existe pas,
/// ou `AppError::Conflict` si elle n'est plus au statut « à postuler ».
pub async fn delete_a_postuler(pool: &PgPool, id: i64) -> Result<i64, AppError> {
    let mut tx = pool.begin().await.map_err(map_db_err)?;

    // ------------------------------------------------------------------
    // 1. Vérification du statut — FOR UPDATE pour qu'un « Postuler »
    //    concurrent ne puisse pas s'intercaler entre le contrôle et le DELETE
    // ------------------------------------------------------------------
    let status: String = sqlx::query("SELECT status FROM search.candidatures WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?
        .try_get("status")
        .map_err(map_db_err)?;

    if status != "a_postuler" {
        return Err(AppError::Conflict(
            "Seule une candidature au statut « À postuler » peut être supprimée.".to_string(),
        ));
    }

    // ------------------------------------------------------------------
    // 2. Suppression
    // ------------------------------------------------------------------
    let session_id = delete_in_tx(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?;

    tx.commit().await.map_err(map_db_err)?;
    Ok(session_id)
}

/// Supprime une candidature dans une transaction existante. Les lignes qui la
/// référencent (historique, contacts, entretiens) partent par
/// `ON DELETE CASCADE` ; l'offre source éventuelle repasse `reviewed` pour
/// réapparaître dans le Marché.
///
/// Retourne l'identifiant de session de la candidature supprimée, `None` si
/// elle n'existait pas.
async fn delete_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: i64,
) -> Result<Option<i64>, AppError> {
    let Some(row) = sqlx::query("DELETE FROM search.candidatures WHERE id = $1 RETURNING session_id, offer_id")
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_err)?
    else {
        return Ok(None);
    };

    let session_id: i64 = row.try_get("session_id").map_err(map_db_err)?;
    let offer_id: Option<i64> = row.try_get("offer_id").map_err(map_db_err)?;
    if let Some(offer_id) = offer_id {
        crate::db::offers::revert_applied_tx(tx, offer_id).await?;
    }
    Ok(Some(session_id))
}

/// Marque une candidature comme postulée (statut `postule`), utilisé une fois
/// la candidature effectivement envoyée (mail, message ou export local sans
/// message). La mise à jour du statut et son entrée d'historique sont écrites
/// dans la même transaction, pour ne jamais désynchroniser les deux.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature à marquer comme postulée.
/// * `content` - Message (mail ou message de candidature) envoyé, `None` si
///   la candidature a été postulée sans message.
///
/// # Returns
///
/// `Ok(())` une fois le changement de statut effectué, `AppError::NotFound`
/// si la candidature n'existe pas, ou `AppError::Conflict` si elle n'est plus
/// au statut « à postuler ».
pub async fn mark_postule(
    pool: &PgPool,
    id: i64,
    content: Option<&str>,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(map_db_err)?;

    let ancien_status: String = sqlx::query("SELECT status FROM search.candidatures WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?
        .try_get("status")
        .map_err(map_db_err)?;

    if ancien_status != "a_postuler" {
        return Err(AppError::Conflict(
            "Seule une candidature au statut « À postuler » peut être marquée comme postulée.".to_string(),
        ));
    }

    sqlx::query("UPDATE search.candidatures SET status = 'postule' WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;

    status_history::record(
        &mut tx,
        id,
        EventType::CandidatureEnvoyee,
        Some(&ancien_status),
        Some("postule"),
        content,
        None,
    )
    .await?;

    tx.commit().await.map_err(map_db_err)?;

    Ok(())
}

/// Liste les candidatures d'une session, les plus récentes en premier.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// Candidatures triées par date de création décroissante.
pub async fn list_for_session(
    pool: &PgPool,
    session_id: i64,
) -> Result<Vec<CandidatureView>, AppError> {
    let rows = sqlx::query(
        "SELECT id, session_id, offer_id, poste, entreprise, localisation, status, source, \
                type AS type_contrat, date_candidature, \
                COALESCE( \
                    (SELECT MAX(changed_at) FROM search.status_history \
                     WHERE status_history.candidature_id = candidatures.id), \
                    created_at \
                ) AS derniere_maj_statut, \
                cv_path, lettre_motivation_path, \
                created_at \
         FROM search.candidatures \
         WHERE session_id = $1 \
         ORDER BY created_at DESC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    rows.iter()
        .map(|r| -> Result<CandidatureView, sqlx::Error> {
            Ok(CandidatureView {
                id: r.try_get("id")?,
                session_id: r.try_get("session_id")?,
                offer_id: r.try_get("offer_id")?,
                poste: r.try_get("poste")?,
                entreprise: r.try_get("entreprise")?,
                localisation: r.try_get("localisation")?,
                status: r.try_get("status")?,
                source: r.try_get("source")?,
                type_contrat: r.try_get("type_contrat")?,
                date_candidature: r.try_get("date_candidature")?,
                derniere_maj_statut: r.try_get("derniere_maj_statut")?,
                cv_path: r.try_get("cv_path")?,
                lettre_motivation_path: r.try_get("lettre_motivation_path")?,
                created_at: r.try_get("created_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(map_db_err)
}

/// Récupère le détail complet d'une candidature (tous les champs, pour la
/// page de détail).
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// `CandidatureDetail` correspondante, ou `AppError::NotFound` si elle n'existe pas.
pub async fn fetch_detail(
    pool: &PgPool,
    id: i64,
) -> Result<CandidatureDetail, AppError> {
    let row = sqlx::query(
        "SELECT id, session_id, offer_id, poste, entreprise, localisation, status, source, \
                type AS type_contrat, remuneration, url_offre, description_offre, notes, \
                date_candidature, \
                cv_path, lettre_motivation_path, \
                created_at, \
                (SELECT version FROM search.document_versions WHERE id = cv_reference_version_id) AS cv_reference_version, \
                (SELECT version FROM search.document_versions WHERE id = lm_reference_version_id) AS lm_reference_version \
         FROM search.candidatures \
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?
    .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?;

    Ok(CandidatureDetail {
        id: row.try_get("id").map_err(map_db_err)?,
        session_id: row.try_get("session_id").map_err(map_db_err)?,
        offer_id: row.try_get("offer_id").map_err(map_db_err)?,
        poste: row.try_get("poste").map_err(map_db_err)?,
        entreprise: row.try_get("entreprise").map_err(map_db_err)?,
        localisation: row.try_get("localisation").map_err(map_db_err)?,
        status: row.try_get("status").map_err(map_db_err)?,
        source: row.try_get("source").map_err(map_db_err)?,
        type_contrat: row.try_get("type_contrat").map_err(map_db_err)?,
        remuneration: row.try_get("remuneration").map_err(map_db_err)?,
        url_offre: row.try_get("url_offre").map_err(map_db_err)?,
        description_offre: row.try_get("description_offre").map_err(map_db_err)?,
        notes: row.try_get("notes").map_err(map_db_err)?,
        date_candidature: row.try_get("date_candidature").map_err(map_db_err)?,
        cv_path: row.try_get("cv_path").map_err(map_db_err)?,
        lettre_motivation_path: row.try_get("lettre_motivation_path").map_err(map_db_err)?,
        cv_reference_version: row.try_get("cv_reference_version").map_err(map_db_err)?,
        lm_reference_version: row.try_get("lm_reference_version").map_err(map_db_err)?,
        created_at: row.try_get("created_at").map_err(map_db_err)?,
    })
}

/// Récupère le statut courant d'une candidature.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `id` - Identifiant de la candidature.
///
/// # Returns
///
/// Statut brut de la candidature, ou `AppError::NotFound` si elle n'existe pas.
pub async fn fetch_status(pool: &PgPool, id: i64) -> Result<String, AppError> {
    sqlx::query("SELECT status FROM search.candidatures WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?
        .try_get("status")
        .map_err(map_db_err)
}
