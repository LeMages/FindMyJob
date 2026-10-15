//! Règles métier et persistance de la timeline de suivi d'une candidature :
//! quels événements sont proposables pour le statut courant, et écriture
//! transactionnelle d'un nouvel événement (mise à jour éventuelle du statut,
//! de l'entretien concerné, et de `status_history`).

// Importation des crates externes
use sqlx::{SqliteConnection, SqlitePool};

// Importation des modules internes
use crate::clock::format_timestamp;
use crate::db::candidatures;
use crate::db::common::{map_db_err, normalize};
use crate::db::contacts;
use crate::db::interviews::{self, NewInterview};
use crate::db::status_history::{self, NewHistoryEntry};
use crate::db::transaction::begin_write;
use crate::error::AppError;
use crate::models::{AvailableEvent, EventType, NewInterviewData, NewTimelineEvent};

/// Statuts non terminaux d'une candidature — un événement de timeline ne peut
/// être ajouté que tant que la candidature est dans l'un de ces statuts.
/// `offre` y figure : recevoir une offre ne verrouille pas la candidature,
/// il reste possible de l'accepter, la décliner, ou se désister.
const STATUTS_NON_TERMINAUX: [&str; 6] = ["a_postuler", "postule", "relance", "entretien", "entretien_passe", "offre"];

/// Construit la liste des événements que l'utilisateur peut ajouter à la
/// timeline pour le statut courant d'une candidature.
///
/// # Arguments
///
/// * `current_status` - Statut brut courant de la candidature.
/// * `has_pending_interviews` - `true` si au moins un entretien `en_attente`
///   existe pour cette candidature (conditionne `entretien_passe`).
///
/// # Returns
///
/// Événements disponibles, dans l'ordre où ils doivent être proposés à
/// l'utilisateur. `creation` et `candidature_envoyee` ne sont jamais inclus :
/// ils sont gérés par les flux existants (création, `postuler_*`).
pub fn available_events(current_status: &str, has_pending_interviews: bool) -> Vec<AvailableEvent> {
    if !STATUTS_NON_TERMINAUX.contains(&current_status) {
        return Vec::new();
    }

    let mut events = Vec::new();

    // ------------------------------------------------------------------
    // 1. Événements sans changement de statut, disponibles dès qu'une
    //    candidature est active (y compris « à postuler »).
    // ------------------------------------------------------------------
    push_event(&mut events, EventType::MessageEnvoye, "Message envoyé", None);
    push_event(&mut events, EventType::MessageRecu, "Message reçu", None);
    push_event(&mut events, EventType::Appel, "Appel", None);
    push_event(&mut events, EventType::Note, "Note", None);

    // ------------------------------------------------------------------
    // 2. Événements conditionnés au statut courant
    // ------------------------------------------------------------------
    if matches!(current_status, "postule" | "relance" | "entretien_passe") {
        push_event(&mut events, EventType::Relance, "Relance", Some("relance"));
    }
    if matches!(current_status, "postule" | "relance" | "entretien" | "entretien_passe") {
        push_event(&mut events, EventType::EntretienPlanifie, "Entretien planifié", Some("entretien"));
    }
    if matches!(current_status, "entretien" | "entretien_passe") && has_pending_interviews {
        push_event(&mut events, EventType::EntretienPasse, "Entretien passé", Some("entretien_passe"));
    }
    if matches!(current_status, "entretien" | "entretien_passe") {
        push_event(&mut events, EventType::OffreRecue, "Offre reçue", Some("offre"));
    }
    if matches!(current_status, "postule" | "relance" | "entretien" | "entretien_passe") {
        push_event(&mut events, EventType::RefusRecu, "Refus reçu", Some("refus"));
    }
    if matches!(current_status, "postule" | "relance" | "entretien_passe") {
        push_event(&mut events, EventType::Ghosting, "Ghosting", Some("ghosting"));
    }
    if current_status == "offre" {
        push_event(&mut events, EventType::AccepterOffre, "Accepter l'offre", Some("accepte"));
        push_event(&mut events, EventType::DeclinerOffre, "Décliner l'offre", Some("offre_refusee"));
    }
    push_event(&mut events, EventType::Desistement, "Se désister", Some("abandon"));

    events
}

/// Ajoute un `AvailableEvent` à la liste, à partir d'un `EventType` et de son
/// éventuel statut cible.
fn push_event(events: &mut Vec<AvailableEvent>, event_type: EventType, label: &str, target_status: Option<&str>) {
    events.push(AvailableEvent {
        event_type: event_type.as_str().to_string(),
        label: label.to_string(),
        changes_status: target_status.is_some(),
        target_status: target_status.map(str::to_string),
    });
}

/// Liste les événements proposables pour une candidature, d'après son
/// statut courant et ses entretiens en attente.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// Événements disponibles, ou `AppError::NotFound` si la candidature n'existe pas.
pub async fn available_for_candidature(pool: &SqlitePool, candidature_id: i64) -> Result<Vec<AvailableEvent>, AppError> {
    let status = candidatures::fetch_status(pool, candidature_id).await?;
    let has_pending = interviews::count_pending(pool, candidature_id).await? > 0;
    Ok(available_events(&status, has_pending))
}

/// Vérifie qu'un événement est autorisé pour le statut courant —
/// `available_events` reste la seule source de vérité, jamais de confiance
/// dans ce qu'envoie le frontend.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `candidature_id` - Identifiant de la candidature.
/// * `event_type` - Événement demandé.
///
/// # Returns
///
/// Statut courant, `AppError::NotFound` si la candidature n'existe pas, ou
/// `AppError::Conflict` si l'événement n'est pas autorisé.
async fn ensure_allowed(
    conn: &mut SqliteConnection,
    candidature_id: i64,
    event_type: EventType,
) -> Result<String, AppError> {
    let ancien_status = candidatures::fetch_status(&mut *conn, candidature_id).await?;
    let has_pending = interviews::count_pending(&mut *conn, candidature_id).await? > 0;
    let allowed = available_events(&ancien_status, has_pending);
    if !allowed.iter().any(|a| a.event_type == event_type.as_str()) {
        return Err(AppError::Conflict(
            "Cet événement n'est pas disponible pour le statut courant.".to_string(),
        ));
    }
    Ok(ancien_status)
}

/// Crée l'entretien d'un événement `entretien_planifie`.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `candidature_id` - Identifiant de la candidature.
/// * `data` - Données de l'entretien soumises par le frontend.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// Identifiant de l'entretien créé, ou `AppError::Validation`.
async fn plan_interview(
    conn: &mut SqliteConnection,
    candidature_id: i64,
    data: Option<NewInterviewData>,
    now: &str,
) -> Result<i64, AppError> {
    let data = data.ok_or_else(|| AppError::Validation("Les données de l'entretien sont requises.".to_string()))?;
    let type_entretien = data.type_entretien.to_lowercase();
    if !interviews::TYPES_ENTRETIEN_VALIDES.contains(&type_entretien.as_str()) {
        return Err(AppError::Validation("Type d'entretien invalide.".to_string()));
    }
    let modalite = normalize(data.modalite).map(|m| m.to_lowercase());
    if let Some(m) = &modalite {
        if !interviews::MODALITES_VALIDES.contains(&m.as_str()) {
            return Err(AppError::Validation("Modalité d'entretien invalide.".to_string()));
        }
    }
    let contact_ids = linked_contact_ids(conn, candidature_id, &data.contact_ids).await?;
    let date_entretien = data.date_entretien.map(format_timestamp);
    let lieu = normalize(data.lieu);
    let new = NewInterview {
        candidature_id,
        type_entretien: &type_entretien,
        date_entretien: date_entretien.as_deref(),
        lieu: lieu.as_deref(),
        modalite: modalite.as_deref(),
        contact_ids: &contact_ids,
    };
    interviews::insert(conn, &new, now).await
}

/// Vérifie les interlocuteurs choisis pour un entretien : chacun doit être
/// un contact lié à la candidature (sinon le frontend pourrait rattacher le
/// contact d'une autre candidature, ou un id inexistant qui remonterait en
/// erreur SQLite brute).
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `candidature_id` - Identifiant de la candidature de l'entretien.
/// * `contact_ids` - Interlocuteurs soumis par le frontend.
///
/// # Returns
///
/// Identifiants dédoublonnés, dans l'ordre reçu, ou `AppError::Validation`.
async fn linked_contact_ids(
    conn: &mut SqliteConnection,
    candidature_id: i64,
    contact_ids: &[i64],
) -> Result<Vec<i64>, AppError> {
    let mut unique: Vec<i64> = Vec::with_capacity(contact_ids.len());
    for &contact_id in contact_ids {
        if unique.contains(&contact_id) {
            continue;
        }
        if !contacts::is_linked_to_candidature(&mut *conn, contact_id, candidature_id).await? {
            return Err(AppError::Validation(
                "Un interlocuteur choisi n'est pas un contact de cette candidature.".to_string(),
            ));
        }
        unique.push(contact_id);
    }
    Ok(unique)
}

/// Marque passé l'entretien d'un événement `entretien_passe`.
///
/// # Arguments
///
/// * `conn` - Connexion de la transaction d'écriture en cours.
/// * `candidature_id` - Identifiant de la candidature.
/// * `interview_id` - Entretien concerné, soumis par le frontend.
/// * `interview_issue` - Issue de l'entretien, soumise par le frontend.
/// * `content` - Contenu normalisé de l'événement, enregistré comme notes.
///
/// # Returns
///
/// Identifiant de l'entretien, `AppError::Validation` ou `AppError::NotFound`.
async fn close_interview(
    conn: &mut SqliteConnection,
    candidature_id: i64,
    interview_id: Option<i64>,
    interview_issue: Option<String>,
    content: Option<&str>,
) -> Result<i64, AppError> {
    let interview_id =
        interview_id.ok_or_else(|| AppError::Validation("L'entretien concerné est requis.".to_string()))?;
    let issue = normalize(interview_issue)
        .ok_or_else(|| AppError::Validation("L'issue de l'entretien est requise.".to_string()))?
        .to_lowercase();
    if !interviews::ISSUES_VALIDES.contains(&issue.as_str()) {
        return Err(AppError::Validation("Issue d'entretien invalide.".to_string()));
    }
    interviews::mark_passed(conn, interview_id, candidature_id, &issue, content).await?;
    Ok(interview_id)
}

/// Ajoute un événement à la timeline d'une candidature : valide qu'il est
/// autorisé pour le statut courant, applique ses effets (création ou mise à
/// jour d'un entretien, changement de statut) et écrit la ligne
/// `status_history`, le tout dans une seule transaction et au même horodatage.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `candidature_id` - Identifiant de la candidature concernée.
/// * `event` - Événement soumis par le frontend.
/// * `now` - Horodatage de la transaction.
///
/// # Returns
///
/// `Ok(())` une fois l'événement enregistré, `AppError::NotFound` si la
/// candidature n'existe pas, `AppError::Validation` si un champ requis
/// manque, ou `AppError::Conflict` si l'événement n'est pas autorisé pour le
/// statut courant.
pub async fn add_event(
    pool: &SqlitePool,
    candidature_id: i64,
    event: NewTimelineEvent,
    now: &str,
) -> Result<(), AppError> {
    let NewTimelineEvent { event_type, content, interview_data, interview_id, interview_issue } = event;
    let event_type = EventType::try_from(event_type.as_str())?;

    let mut tx = begin_write(pool).await?;

    // ------------------------------------------------------------------
    // 1. Candidature existante et événement autorisé pour son statut
    // ------------------------------------------------------------------
    let ancien_status = ensure_allowed(&mut tx, candidature_id, event_type).await?;

    // ------------------------------------------------------------------
    // 2. Validation des champs requis selon le type d'événement
    // ------------------------------------------------------------------
    let content = normalize(content);
    if matches!(
        event_type,
        EventType::Relance | EventType::MessageEnvoye | EventType::MessageRecu | EventType::Note
    ) && content.is_none()
    {
        return Err(AppError::Validation("Le contenu de cet événement est requis.".to_string()));
    }

    // ------------------------------------------------------------------
    // 3. Effets spécifiques : création ou mise à jour d'un entretien
    // ------------------------------------------------------------------
    let interview_id = match event_type {
        EventType::EntretienPlanifie => Some(plan_interview(&mut tx, candidature_id, interview_data, now).await?),
        EventType::EntretienPasse => {
            Some(close_interview(&mut tx, candidature_id, interview_id, interview_issue, content.as_deref()).await?)
        }
        _ => None,
    };

    // ------------------------------------------------------------------
    // 4. Changement de statut, si l'événement en implique un
    // ------------------------------------------------------------------
    let nouveau_status = event_type.changes_status();
    if let Some(target) = nouveau_status {
        sqlx::query("UPDATE candidatures SET status = $1 WHERE id = $2")
            .bind(target)
            .bind(candidature_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_err)?;
    }

    // ------------------------------------------------------------------
    // 5. Trace de l'événement dans la timeline
    // ------------------------------------------------------------------
    let entry = NewHistoryEntry {
        candidature_id,
        event_type,
        ancien_status: Some(&ancien_status),
        nouveau_status,
        content: content.as_deref(),
        interview_id,
    };
    status_history::record(&mut tx, &entry, now).await?;

    tx.commit().await.map_err(map_db_err)?;
    Ok(())
}
