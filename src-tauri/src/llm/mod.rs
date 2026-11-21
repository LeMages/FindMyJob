//! Ré-exports publics de la couche d'intégration avec l'API Claude : client
//! HTTP générique, pipeline de génération de documents (analyse d'offre,
//! matching de profil, rédaction de la prose, traduction des descriptions
//! du profil) et rédaction de
//! messages de candidature (mail, message court).

pub mod api_key;
pub mod call_record;
pub mod caller;
pub mod candidature_message;
pub mod client;
pub mod cv_writing;
pub mod detailed_context_block;
pub mod doc_data_schema;
pub mod lm_writing;
pub mod matching;
pub mod offer;
pub mod profile_facts;
pub mod profile_prompt;
pub mod profile_translation;
pub mod prose_checks;
pub mod schema_size;
pub mod scraping_extract;
pub mod target;
#[cfg(test)]
mod test_profile;
pub mod validated_call;
pub mod writing_loop;
pub mod writing_rules;

pub use api_key::ApiKey;
pub use call_record::{CallRecorder, LlmCallRecord, TokenUsage};
pub use caller::{LlmCaller, LlmFuture, LlmRequest, LlmResponse};
pub use candidature_message::{generate_application_email, generate_application_message};
pub use client::{call_claude, call_claude_with_tokens, AnthropicCaller};
pub use target::ClaudeTarget;
