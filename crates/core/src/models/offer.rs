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
    pub status: String,
    pub score: Option<f32>,
    pub created_at: DateTime<Utc>,
}
