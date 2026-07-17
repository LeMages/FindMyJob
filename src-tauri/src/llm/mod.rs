//! Ré-exports publics de la couche d'intégration avec l'API Claude : client
//! HTTP générique et pipeline de génération de documents (analyse de style,
//! analyse d'offre, matching de profil, rédaction de contenu, assemblage
//! Typst).

pub mod assemble;
pub mod client;
pub mod content;
pub mod matching;
pub mod offer;
pub mod pipeline;
pub mod style;

pub use client::{call_claude, call_claude_with_tokens};
pub use pipeline::generate_documents;
pub use style::analyze_style;
