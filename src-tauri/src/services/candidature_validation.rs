//! Validation et normalisation des champs d'une candidature, communes à la
//! création et à la modification.

// Importation des modules internes
use crate::db::candidatures::CandidatureFields;
use crate::db::common::normalize;
use crate::db::contacts::{self, ContactToLink};
use crate::error::AppError;
use crate::models::{CreateCandidatureInput, UpdateCandidatureInput};

const TYPES_CONTRAT_VALIDES: [&str; 6] = ["stage", "alternance", "cdi", "cdd", "interim", "freelance"];

/// Candidature à créer, champs validés.
///
/// # Attributes
///
/// * `session_id` - Session de rattachement.
/// * `fields` - Champs éditables validés.
/// * `description_offre` - Contenu de l'offre.
/// * `contacts` - Contacts à lier, validés.
pub struct NewCandidature {
    pub session_id: i64,
    pub fields: CandidatureFields,
    pub description_offre: String,
    pub contacts: Vec<ContactToLink>,
}

/// Champs obligatoires partagés par la création et la modification.
///
/// # Attributes
///
/// * `poste`, `entreprise`, `localisation`, `source` - Valeurs trimées, non vides.
struct RequiredFields {
    poste: String,
    entreprise: String,
    localisation: String,
    source: String,
}

/// Exige une valeur non vide après normalisation.
///
/// # Arguments
///
/// * `value` - Valeur reçue.
/// * `message` - Message d'erreur si elle est absente.
///
/// # Returns
///
/// Valeur trimée, ou `AppError::Validation`.
fn required(value: &Option<String>, message: &str) -> Result<String, AppError> {
    normalize(value.clone()).ok_or_else(|| AppError::Validation(message.to_string()))
}

/// Valide les champs obligatoires communs, dans l'ordre des messages
/// d'erreur affichés à l'utilisateur.
///
/// # Arguments
///
/// * `poste` - Intitulé du poste.
/// * `entreprise`, `localisation`, `source` - Valeurs reçues.
///
/// # Returns
///
/// `RequiredFields`, ou la première `AppError::Validation` rencontrée.
fn required_fields(
    poste: &str,
    entreprise: &Option<String>,
    localisation: &Option<String>,
    source: &Option<String>,
) -> Result<RequiredFields, AppError> {
    let poste = poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation("L'intitulé du poste est requis.".to_string()));
    }
    Ok(RequiredFields {
        poste,
        entreprise: required(entreprise, "L'entreprise est requise.")?,
        localisation: required(localisation, "Le lieu est requis.")?,
        source: required(source, "La plateforme de l'offre est requise.")?,
    })
}

/// Valide le type de contrat et le passe en minuscules.
///
/// # Arguments
///
/// * `type_contrat` - Valeur reçue.
///
/// # Returns
///
/// Type de contrat normalisé, ou `AppError::Validation`.
fn type_contrat(type_contrat: &Option<String>) -> Result<String, AppError> {
    let value = required(type_contrat, "Le type de contrat est requis.")?.to_lowercase();
    if !TYPES_CONTRAT_VALIDES.contains(&value.as_str()) {
        return Err(AppError::Validation("Type de contrat invalide.".to_string()));
    }
    Ok(value)
}

/// Valide une demande de création de candidature.
///
/// # Arguments
///
/// * `input` - Données envoyées par l'interface.
///
/// # Returns
///
/// `NewCandidature`, ou `AppError::Validation` (champ manquant ou invalide,
/// session absente).
pub fn validate_create(input: &CreateCandidatureInput) -> Result<NewCandidature, AppError> {
    let session_id = input
        .session_id
        .ok_or_else(|| AppError::Validation("La session de la candidature est requise.".to_string()))?;
    let common = required_fields(&input.poste, &input.entreprise, &input.localisation, &input.source)?;
    let description_offre = required(&input.description_offre, "Le contenu de l'offre est requis.")?;
    let type_contrat = type_contrat(&input.type_contrat)?;
    let contacts = contacts::validate_for_candidature(&input.contacts)?;

    Ok(NewCandidature {
        session_id,
        fields: CandidatureFields {
            poste: common.poste,
            entreprise: common.entreprise,
            localisation: common.localisation,
            type_contrat,
            source: common.source,
            remuneration: normalize(input.remuneration.clone()),
            url_offre: normalize(input.url_offre.clone()),
            notes: normalize(input.notes.clone()),
        },
        description_offre,
        contacts,
    })
}

/// Valide une demande de modification de candidature.
///
/// # Arguments
///
/// * `input` - Données envoyées par l'interface.
///
/// # Returns
///
/// `CandidatureFields`, ou `AppError::Validation`.
pub fn validate_update(input: &UpdateCandidatureInput) -> Result<CandidatureFields, AppError> {
    let common = required_fields(&input.poste, &input.entreprise, &input.localisation, &input.source)?;
    let type_contrat = type_contrat(&input.type_contrat)?;

    Ok(CandidatureFields {
        poste: common.poste,
        entreprise: common.entreprise,
        localisation: common.localisation,
        type_contrat,
        source: common.source,
        remuneration: normalize(input.remuneration.clone()),
        url_offre: normalize(input.url_offre.clone()),
        notes: normalize(input.notes.clone()),
    })
}
