//! Ré-exports publics des commandes Tauri, groupées par entité métier.
//! Chaque commande délègue sa logique à l'agent via `api_client::ApiClient` —
//! aucun accès direct à une base de données ici.

pub mod candidatures;
pub mod contacts;
pub mod document_versions;
pub mod documents;
pub mod experiences;
pub mod export;
pub mod fonts;
pub mod formations;
pub mod interviews;
pub mod langues;
pub mod offers;
pub mod projets;
pub mod scraping;
pub mod search_queries;
pub mod sessions;
pub mod settings;
pub mod skills;
pub mod status_history;
pub mod system;
pub mod timeline_events;
pub mod users;

// `pub use module::*` (plutôt que des imports nommés) est nécessaire ici :
// la macro #[tauri::command] génère des items cachés (__cmd__xxx) à côté de
// chaque fonction, que seul un ré-export glob propage jusqu'à `generate_handler!`.
pub use candidatures::*;
pub use contacts::*;
pub use document_versions::*;
pub use documents::*;
pub use experiences::*;
pub use export::*;
pub use fonts::*;
pub use formations::*;
pub use interviews::*;
pub use langues::*;
pub use offers::*;
pub use projets::*;
pub use scraping::*;
pub use search_queries::*;
pub use sessions::*;
pub use settings::*;
pub use skills::*;
pub use status_history::*;
pub use system::*;
pub use timeline_events::*;
pub use users::*;
