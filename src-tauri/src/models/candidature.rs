//! DTOs liés aux candidatures.

#[derive(serde::Serialize)]
pub struct CandidatureView {
    pub id: i64,
    pub session_id: Option<i64>,
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub status: String,
    pub source: Option<String>,
    pub type_contrat: Option<String>,
    pub date_candidature: Option<String>,
    pub next_relance: Option<String>,
    pub cv_path: Option<String>,
    pub lettre_motivation_path: Option<String>,
    pub created_at: String,
}

#[derive(serde::Deserialize)]
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
