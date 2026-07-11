//! Ré-exports publics des commandes Tauri, groupées par entité métier.
//! Chaque commande délègue sa logique à la couche `db::` ou `documents::` —
//! aucune requête SQL directe ici.

pub mod candidatures;
pub mod documents;
pub mod experiences;
pub mod formations;
pub mod langues;
pub mod projets;
pub mod sessions;
pub mod skills;
pub mod system;
pub mod users;

// `pub use module::*` (plutôt que des imports nommés) est nécessaire ici :
// la macro #[tauri::command] génère des items cachés (__cmd__xxx) à côté de
// chaque fonction, que seul un ré-export glob propage jusqu'à `generate_handler!`.
pub use candidatures::*;
pub use documents::*;
pub use experiences::*;
pub use formations::*;
pub use langues::*;
pub use projets::*;
pub use sessions::*;
pub use skills::*;
pub use system::*;
pub use users::*;
