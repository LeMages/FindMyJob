//! DTOs liés aux sessions de candidature.

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionView {
    pub id: i64,
    pub nom: String,
    pub contexte: Option<String>,
    pub statut: String,
    pub zone: Option<String>,
    pub min_score: f32,
    pub candidature_count: i64,
    pub has_accepted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSessionInput {
    pub nom: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSessionInput {
    pub nom: String,
    pub contexte: Option<String>,
    pub statut: String,
    pub zone: Option<String>,
    pub min_score: f32,
}
