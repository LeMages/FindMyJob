//! DTOs liés aux offres scrapées et scorées d'une session.

use chrono::{DateTime, Utc};

/// Vue d'une offre scrapée, exposée par l'API agent (onglet « Marché »).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OfferView {
    pub id: i64,
    pub session_id: i64,
    pub source: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub contract_type: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    /// Texte brut de l'annonce — utilisé pour pré-remplir `description_offre`
    /// du formulaire de candidature quand elle est créée depuis le Marché.
    pub description: Option<String>,
    pub status: String,
    pub score: Option<f32>,
    pub created_at: DateTime<Utc>,
    /// `true` si une candidature référence déjà cette offre (`offer_id`).
    pub has_candidature: bool,
}
