//! Ré-exports publics de la couche d'intégration avec l'API Claude : client
//! HTTP générique, pipeline de génération de documents (templatisation des
//! références, analyse d'offre, matching de profil, rédaction de la prose,
//! traduction des libellés) et rédaction de messages de candidature (mail,
//! message court).

pub mod candidature_message;
pub mod client;
pub mod cv_writing;
pub mod doc_data_schema;
pub mod label_translation;
pub mod lm_writing;
pub mod matching;
pub mod offer;
pub mod profile_facts;
pub mod scraping_extract;
pub mod templatize;
pub mod writing_loop;
pub mod writing_rules;

pub use candidature_message::{generate_application_email, generate_application_message};
pub use client::{call_claude, call_claude_with_tokens};
pub use templatize::{templatize_reference, TemplatizeFeedback, TemplatizeOutput};
