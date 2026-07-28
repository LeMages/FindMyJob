//! DTO représentant un entretien rattaché à une candidature.

#[derive(serde::Serialize)]
pub struct InterviewView {
    pub id: i64,
    pub candidature_id: i64,
    #[serde(rename = "type")]
    pub type_entretien: Option<String>,
    pub date_entretien: Option<String>,
    pub lieu: Option<String>,
    pub issue: String,
    pub notes: Option<String>,
    pub contact_id: Option<i64>,
}
