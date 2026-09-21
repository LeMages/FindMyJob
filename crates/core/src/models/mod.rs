//! Ré-exports publics des DTOs utilisés par les commandes Tauri.

pub mod candidature;
pub mod contact;
pub mod document;
pub mod event_type;
pub mod experience;
pub mod font;
pub mod formation;
pub mod interview;
pub mod lang;
pub mod offer;
pub mod profile;
pub mod projet;
pub mod search_query;
pub mod session;
pub mod skill;
pub mod status_history;
pub mod timeline_event;
pub mod user;

pub use candidature::{CandidatureDetail, CandidatureView, CreateCandidatureInput, UpdateCandidatureInput};
pub use contact::ContactView;
pub use document::{Document, DocumentFileInfo, DocumentReferenceView, DocumentSaveProgress};
pub use event_type::EventType;
pub use experience::{CreateExperienceInput, ExperienceView};
pub use font::FontFamilyView;
pub use formation::{CreateFormationInput, FormationView};
pub use interview::{InterviewView, PlanningInterviewView};
pub use lang::{CreateLangInput, LangView};
pub use offer::OfferView;
pub use profile::ProfileDto;
pub use projet::{CreateProjetInput, ProjetView};
pub use search_query::{CreateSearchQueryInput, SearchQueryView};
pub use session::{CreateSessionInput, SessionView, UpdateSessionInput};
pub use skill::{CreateSkillInput, SkillView};
pub use status_history::StatusHistoryView;
pub use timeline_event::{AvailableEvent, NewInterviewData, NewTimelineEvent};
pub use user::{CreateUserInput, UpdateUserInfoInput, UserView};
