//! Ré-exports publics de la couche d'intégration avec l'API Claude : client
//! HTTP générique, pipeline de génération de documents (analyse de style,
//! analyse d'offre, matching de profil, rédaction de contenu, assemblage
//! Typst) et rédaction de messages de candidature (mail, message court).

pub mod assemble;
pub mod candidature_message;
pub mod client;
pub mod content;
pub mod matching;
pub mod offer;
pub mod pipeline;
pub mod style;

pub use candidature_message::{generate_application_email, generate_application_message};
pub use client::{call_claude, call_claude_with_tokens};
pub use pipeline::generate_documents;
pub use style::analyze_style;
