//! DTOs liés aux candidatures.

use chrono::{DateTime, NaiveDate, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CandidatureView {
    pub id: i64,
    pub session_id: Option<i64>,
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub status: String,
    pub source: Option<String>,
    pub type_contrat: Option<String>,
    pub date_candidature: Option<NaiveDate>,
    /// Horodatage du dernier changement de statut (issu de `status_history`),
    /// utilisé pour calculer l'inactivité affichée dans la vue pipeline.
    pub derniere_maj_statut: DateTime<Utc>,
    pub cv_path: Option<String>,
    pub lettre_motivation_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Vue complète d'une candidature, utilisée par la page de détail (contient
/// les champs additionnels absents de `CandidatureView`, qui reste léger pour
/// l'affichage en liste/kanban).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CandidatureDetail {
    pub id: i64,
    pub session_id: Option<i64>,
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub status: String,
    pub source: Option<String>,
    pub type_contrat: Option<String>,
    pub remuneration: Option<String>,
    pub url_offre: Option<String>,
    pub description_offre: Option<String>,
    pub notes: Option<String>,
    pub date_candidature: Option<NaiveDate>,
    pub cv_path: Option<String>,
    pub lettre_motivation_path: Option<String>,
    /// Numéro de la version du CV de référence utilisée pour la génération,
    /// `None` si générée avant le versioning ou version supprimée.
    pub cv_reference_version: Option<i32>,
    /// Numéro de la version de la lettre de référence utilisée pour la génération.
    pub lm_reference_version: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateCandidatureInput {
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub type_contrat: Option<String>,
    pub source: Option<String>,
    pub remuneration: Option<String>,
    pub url_offre: Option<String>,
    pub description_offre: Option<String>,
    pub notes: Option<String>,
    pub cv_path: Option<String>,
    pub lettre_motivation_path: Option<String>,
    pub session_id: Option<i64>,
}

/// Champs modifiables depuis la page de détail, uniquement au statut
/// « à_postuler » (voir `db::candidatures::update`).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateCandidatureInput {
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub type_contrat: Option<String>,
    pub source: Option<String>,
    pub remuneration: Option<String>,
    pub url_offre: Option<String>,
    pub notes: Option<String>,
}

/// Candidature existante dont l'URL d'offre correspond (après
/// normalisation) à une URL donnée : sert à l'avertissement non bloquant de
/// doublon à la création, toutes sessions confondues.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DuplicateCandidatureView {
    pub id: i64,
    pub session_id: i64,
    pub session_nom: String,
    pub poste: String,
    pub entreprise: Option<String>,
    pub status: String,
}
