//! Ré-exports publics du module de scraping : validation de l'URL d'une
//! offre, récupération de son HTML (`fetch`), raisons d'échec typées et
//! extraction de son texte exploitable (`extract`).

pub mod extract;
pub mod failure;
pub mod fetch;
pub mod fetch_classification;
pub mod ip_policy;
pub mod safe_resolver;
pub mod url_validation;
