//! DTOs du carnet de contacts recruteur : un contact existe indépendamment
//! des candidatures et peut être lié à plusieurs.

use chrono::{DateTime, Utc};

/// Contact du carnet.
///
/// # Attributes
///
/// * `nom` - Nom du contact (obligatoire).
/// * `role` - Rôle du contact dans l'entreprise.
/// * `candidature_count` - Nombre de candidatures auxquelles le contact est lié.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ContactView {
    pub id: i64,
    pub nom: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub notes: Option<String>,
    pub candidature_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Saisie d'un contact, en création comme en modification. Le texte est
/// normalisé côté backend (trim, chaîne vide → absent) ; le nom est requis.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ContactInput {
    pub nom: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub notes: Option<String>,
}

/// Contact à lier à une candidature au moment de sa création : un contact
/// existant du carnet, ou un nouveau contact à créer.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CandidatureContactInput {
    /// Contact déjà présent dans le carnet.
    Existing { contact_id: i64 },
    /// Nouveau contact, créé dans la même transaction que la candidature.
    New { contact: ContactInput },
}
