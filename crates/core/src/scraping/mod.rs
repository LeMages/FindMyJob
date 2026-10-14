//! Ré-exports publics du module de scraping : validation de l'URL d'une
//! offre, récupération de son HTML (`fetch`), raisons d'échec typées, et
//! cascade d'extraction — JSON-LD (`json_ld`) puis texte de la page pour le
//! LLM (`extract`) — avec le nettoyage final commun (`finalize`).

pub mod content_check;
pub mod contract_type;
pub mod extract;
pub mod failure;
pub mod fetch;
pub mod fetch_classification;
pub mod finalize;
pub mod ip_policy;
pub mod json_ld;
pub mod page_hints;
pub mod safe_resolver;
pub mod salary_format;
pub mod url_validation;
