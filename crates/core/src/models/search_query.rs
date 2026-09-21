//! DTOs liés aux requêtes de recherche d'une session.

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchQueryView {
    pub id: i64,
    pub session_id: i64,
    pub keywords: String,
    pub location: Option<String>,
    pub country_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSearchQueryInput {
    pub keywords: String,
    pub location: Option<String>,
    pub country_code: Option<String>,
}
