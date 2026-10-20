//! Ré-exports publics de la couche d'intégration avec l'API Claude : client
//! HTTP générique, pipeline de génération de documents (templatisation des
//! références, analyse d'offre, matching de profil, rédaction de la prose,
//! traduction des libellés et des descriptions du profil) et rédaction de
//! messages de candidature (mail, message court).

pub mod api_key;
pub mod call_record;
pub mod caller;
pub mod candidature_message;
pub mod client;
pub mod cv_writing;
pub mod doc_data_schema;
pub mod label_translation;
pub mod lm_writing;
pub mod matching;
pub mod offer;
pub mod profile_facts;
pub mod profile_translation;
pub mod prose_checks;
pub mod reference_translation;
pub mod schema_size;
pub mod scraping_extract;
pub mod target;
pub mod templatize_example;
pub mod templatize_rules;
pub mod templatize_template;
pub mod validated_call;
pub mod writing_loop;
pub mod writing_rules;

pub use api_key::ApiKey;
pub use call_record::{CallRecorder, LlmCallRecord, TokenUsage};
pub use caller::{LlmCaller, LlmFuture, LlmRequest, LlmResponse};
pub use candidature_message::{generate_application_email, generate_application_message};
pub use client::{call_claude, call_claude_with_tokens, AnthropicCaller};
pub use target::ClaudeTarget;
