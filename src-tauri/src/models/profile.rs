//! DTO agrégeant l'ensemble des informations du profil utilisateur.

// Importation des modules internes
use crate::models::document::DocumentReferenceView;
use crate::models::experience::ExperienceView;
use crate::models::formation::FormationView;
use crate::models::lang::LangView;
use crate::models::projet::ProjetView;
use crate::models::skill::SkillView;
use crate::models::user::UserView;

#[derive(serde::Serialize)]
pub struct ProfileDto {
    pub user: UserView,
    pub experiences: Vec<ExperienceView>,
    pub formations: Vec<FormationView>,
    pub projets: Vec<ProjetView>,
    pub skills: Vec<SkillView>,
    pub langues: Vec<LangView>,
    pub cv_reference: Option<DocumentReferenceView>,
    pub lettre_motivation_reference: Option<DocumentReferenceView>,
}
