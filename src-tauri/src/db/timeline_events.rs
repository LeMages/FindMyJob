//! Règles métier et persistance de la timeline de suivi d'une candidature :
//! quels événements sont proposables pour le statut courant, et écriture
//! transactionnelle d'un nouvel événement (mise à jour éventuelle du statut,
//! de l'entretien concerné, et de `status_history`).

// Importation des crates externes
use sqlx::{Row, SqlitePool};

// Importation des modules internes
use crate::db::common::normalize;
use crate::db::{interviews, status_history};
use crate::error::AppError;
use crate::models::{AvailableEvent, EventType, NewTimelineEvent};

/// Statuts non terminaux d'une candidature — un événement de timeline ne peut
/// être ajouté que tant que la candidature est dans l'un de ces statuts.
/// `offre` y figure : recevoir une offre ne verrouille pas la candidature,
/// il reste possible de l'accepter, la décliner, ou se désister.
const STATUTS_NON_TERMINAUX: [&str; 5] = ["a_postuler", "postule", "relance", "entretien", "offre"];

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
    if matches!(current_status, "postule" | "relance") {
        push_event(&mut events, EventType::Relance, "Relance", Some("relance"));
    }
    if matches!(current_status, "postule" | "relance" | "entretien") {
        push_event(&mut events, EventType::EntretienPlanifie, "Entretien planifié", Some("entretien"));
    }
    if current_status == "entretien" && has_pending_interviews {
        push_event(&mut events, EventType::EntretienPasse, "Entretien passé", None);
    }
    if current_status == "entretien" {
        push_event(&mut events, EventType::OffreRecue, "Offre reçue", Some("offre"));
    }
    if matches!(current_status, "postule" | "relance" | "entretien") {
        push_event(&mut events, EventType::RefusRecu, "Refus reçu", Some("refus"));
    }
    if matches!(current_status, "postule" | "relance") {
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

/// Ajoute un événement à la timeline d'une candidature : valide qu'il est
/// autorisé pour le statut courant, applique ses effets (création ou mise à
/// jour d'un entretien, changement de statut) et écrit la ligne
/// `status_history`, le tout dans une seule transaction.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `user_id` - Identifiant de l'utilisateur propriétaire.
/// * `candidature_id` - Identifiant de la candidature concernée.
/// * `event` - Événement soumis par le frontend.
///
/// # Returns
///
/// `Ok(())` une fois l'événement enregistré, `AppError::NotFound` si la
/// candidature n'existe pas, `AppError::Validation` si un champ requis
/// manque, ou `AppError::Conflict` si l'événement n'est pas autorisé pour le
/// statut courant.
pub async fn add_event(
    pool: &SqlitePool,
    user_id: i64,
    candidature_id: i64,
    event: NewTimelineEvent,
) -> Result<(), AppError> {
    let event_type = EventType::try_from(event.event_type.as_str())?;

    let mut tx = pool.begin().await?;

    // ------------------------------------------------------------------
    // 1. La candidature doit exister, et l'événement doit être autorisé pour
    //    son statut courant — `available_events` reste la seule source de
    //    vérité, jamais de confiance dans ce qu'envoie le frontend.
    // ------------------------------------------------------------------
    let ancien_status: String = sqlx::query("SELECT status FROM candidatures WHERE id = ? AND user_id = ?")
        .bind(candidature_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Candidature introuvable.".to_string()))?
        .try_get("status")
        .map_err(AppError::from)?;

    let has_pending = interviews::count_pending(&mut *tx, candidature_id).await? > 0;
    let allowed = available_events(&ancien_status, has_pending);
    if !allowed.iter().any(|a| a.event_type == event_type.as_str()) {
        return Err(AppError::Conflict(
            "Cet événement n'est pas disponible pour le statut courant.".to_string(),
        ));
    }

    // ------------------------------------------------------------------
    // 2. Validation des champs requis selon le type d'événement
    // ------------------------------------------------------------------
    let content = normalize(event.content);
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
        EventType::EntretienPlanifie => {
            let data = event
                .interview_data
                .ok_or_else(|| AppError::Validation("Les données de l'entretien sont requises.".to_string()))?;
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
            let id = interviews::insert(
                &mut tx,
                candidature_id,
                &type_entretien,
                normalize(data.date_entretien).as_deref(),
                normalize(data.lieu).as_deref(),
                modalite.as_deref(),
                data.contact_id,
            )
            .await?;
            Some(id)
        }
        EventType::EntretienPasse => {
            let interview_id = event
                .interview_id
                .ok_or_else(|| AppError::Validation("L'entretien concerné est requis.".to_string()))?;
            let issue = normalize(event.interview_issue)
                .ok_or_else(|| AppError::Validation("L'issue de l'entretien est requise.".to_string()))?;
            let issue_lower = issue.to_lowercase();
            if !interviews::ISSUES_VALIDES.contains(&issue_lower.as_str()) {
                return Err(AppError::Validation("Issue d'entretien invalide.".to_string()));
            }
            interviews::mark_passed(&mut tx, interview_id, candidature_id, &issue_lower, content.as_deref())
                .await?;
            Some(interview_id)
        }
        _ => None,
    };

    // ------------------------------------------------------------------
    // 4. Changement de statut, si l'événement en implique un
    // ------------------------------------------------------------------
    let nouveau_status = event_type.changes_status();
    if let Some(target) = nouveau_status {
        sqlx::query("UPDATE candidatures SET status = ? WHERE id = ? AND user_id = ?")
            .bind(target)
            .bind(candidature_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    // ------------------------------------------------------------------
    // 5. Trace de l'événement dans la timeline
    // ------------------------------------------------------------------
    status_history::record(
        &mut tx,
        candidature_id,
        event_type,
        Some(&ancien_status),
        nouveau_status,
        content.as_deref(),
        interview_id,
    )
    .await?;

    tx.commit().await?;

    Ok(())
}
