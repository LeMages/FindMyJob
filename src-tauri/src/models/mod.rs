//! Ré-exports publics des DTOs utilisés par les commandes Tauri.

pub mod candidature;
pub mod contact;
pub mod document;
pub mod experience;
pub mod formation;
pub mod lang;
pub mod profile;
pub mod projet;
pub mod session;
pub mod skill;
pub mod status_history;
pub mod user;

pub use candidature::{CandidatureDetail, CandidatureView, CreateCandidatureInput, UpdateCandidatureInput};
pub use contact::ContactView;
pub use document::{Document, DocumentFileInfo, DocumentReferenceView, DocumentSaveProgress};
pub use experience::{CreateExperienceInput, ExperienceView};
pub use formation::{CreateFormationInput, FormationView};
pub use lang::{CreateLangInput, LangView};
pub use profile::ProfileDto;
pub use projet::{CreateProjetInput, ProjetView};
pub use session::{CreateSessionInput, SessionView, UpdateSessionInput};
pub use skill::{CreateSkillInput, SkillView};
pub use status_history::StatusHistoryView;
pub use user::{CreateUserInput, UpdateUserInfoInput, UserView};
