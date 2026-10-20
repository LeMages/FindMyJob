//! Balisage des références : lecture déterministe des consignes générales
//! et des zones `[FIXE]` / `[VARIABLE]` / `[ADAPTABLE]` écrites en
//! commentaires, et rattachement des zones aux champs de prose.

pub mod attach;
pub mod model;
pub mod parse;
pub mod source_events;

pub use attach::attach_zones;
pub use model::{describe_markup_errors, MarkupError, MarkupZone, ReferenceMarkup, VisibleText};
pub use parse::parse_markup;
