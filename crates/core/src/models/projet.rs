//! DTOs liés aux projets personnels du profil.

use chrono::NaiveDate;

#[derive(serde::Serialize)]
pub struct ProjetView {
    pub id: i64,
    pub titre: String,
    pub description: Option<String>,
    pub stack: Option<String>,
    pub url: Option<String>,
    pub pour_qui: Option<String>,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
}

#[derive(serde::Deserialize)]
pub struct CreateProjetInput {
    pub titre: String,
    pub description: Option<String>,
    pub stack: Option<String>,
    pub url: Option<String>,
    pub pour_qui: Option<String>,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
}
