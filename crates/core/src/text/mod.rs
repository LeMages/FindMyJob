//! Conversion déterministe (sans LLM, sans I/O) du texte des offres
//! d'emploi — HTML ou Markdown selon la source — en texte brut lisible.

mod emoji_strip;
pub(crate) mod html_walk;
mod markdown_cleanup;
pub(crate) mod whitespace_normalize;

pub mod html_to_plain;

pub use html_to_plain::html_to_plain;
