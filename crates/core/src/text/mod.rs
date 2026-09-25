//! Conversion déterministe (sans LLM, sans I/O) du texte des offres
//! d'emploi — HTML ou Markdown selon la source — en texte brut lisible.

mod emoji_strip;
mod html_walk;
mod markdown_cleanup;
mod whitespace_normalize;

pub mod html_to_plain;
pub mod offer_description;

pub use html_to_plain::html_to_plain;
pub use offer_description::assemble_offer_description;
