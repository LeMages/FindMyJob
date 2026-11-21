//! Prompt à copier pour faire convertir un CV ou une lettre au contrat
//! FindMyJob par Claude : consignes de conversion, contrat complet, exemple
//! conforme collé en entier et lien vers la documentation.
//!
//! Les textes viennent de `docs/reference-contract/` (source unique,
//! embarquée dans le binaire) ; ce module ne fait que les assembler.

// Importation des modules internes
use crate::documents::doc_type::parse_doc_type;
use crate::error::AppError;
use crate::models::doc_lang::is_supported;
use crate::models::reference_doc_links::ReferenceDocLinks;

/// Base des liens vers la documentation du contrat sur GitHub.
pub const CONTRACT_DOC_BASE_URL: &str = "https://github.com/LeMages/FindMyJob/blob/main/docs/reference-contract/";

/// Emplacement du contrat dans un modèle de prompt.
const CONTRACT_PLACEHOLDER: &str = "{{CONTRACT}}";
/// Emplacement de l'exemple dans un modèle de prompt.
const EXAMPLE_PLACEHOLDER: &str = "{{EXAMPLE}}";
/// Emplacement du lien vers le contrat dans un modèle de prompt.
const DOC_URL_PLACEHOLDER: &str = "{{DOC_URL}}";

// ======================================================================
// Textes embarqués
// ======================================================================

const CONTRACT_FR: &str = include_str!("../../../docs/reference-contract/contract.fr.md");
const CONTRACT_EN: &str = include_str!("../../../docs/reference-contract/contract.en.md");
const CV_EXAMPLE: &str = include_str!("../../../docs/reference-contract/cv_exemple.typ");
const LM_EXAMPLE: &str = include_str!("../../../docs/reference-contract/lm_exemple.typ");
const PROMPT_CV_FR: &str = include_str!("../../../docs/reference-contract/prompt_cv.fr.md");
const PROMPT_CV_EN: &str = include_str!("../../../docs/reference-contract/prompt_cv.en.md");
const PROMPT_LM_FR: &str = include_str!("../../../docs/reference-contract/prompt_lm.fr.md");
const PROMPT_LM_EN: &str = include_str!("../../../docs/reference-contract/prompt_lm.en.md");

/// Lien vers le contrat dans une langue.
///
/// # Arguments
///
/// * `lang` - Langue de la documentation (`"fr"` ou `"en"`).
///
/// # Returns
///
/// URL GitHub de `contract.<lang>.md`.
pub fn contract_doc_url(lang: &str) -> String {
    format!("{CONTRACT_DOC_BASE_URL}contract.{lang}.md")
}

/// Liens du contrat (dans une langue) et des deux exemples conformes.
///
/// # Arguments
///
/// * `lang` - Langue de l'interface (`"fr"` ou `"en"`).
///
/// # Returns
///
/// Les liens GitHub, ou `AppError::Validation` pour une langue inconnue.
pub fn reference_doc_links(lang: &str) -> Result<ReferenceDocLinks, AppError> {
    ensure_supported(lang)?;
    Ok(ReferenceDocLinks {
        contract: contract_doc_url(lang),
        cv_example: format!("{CONTRACT_DOC_BASE_URL}cv_exemple.typ"),
        lm_example: format!("{CONTRACT_DOC_BASE_URL}lm_exemple.typ"),
    })
}

/// Refuse une langue que l'application ne prend pas en charge.
fn ensure_supported(lang: &str) -> Result<(), AppError> {
    if is_supported(lang) {
        return Ok(());
    }
    Err(AppError::Validation(format!("Langue non prise en charge : « {lang} ».")))
}

/// Assemble le prompt de conversion d'une référence.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `lang` - Langue de l'interface, donc du prompt (`"fr"` ou `"en"`).
///
/// # Returns
///
/// Le prompt complet, ou `AppError::Validation` pour un type ou une
/// langue inconnus.
pub fn reference_prompt(doc_type: &str, lang: &str) -> Result<String, AppError> {
    let doc_type = parse_doc_type(doc_type)?;
    ensure_supported(lang)?;
    let english = lang == "en";
    let (template, example) = match (doc_type, english) {
        ("cv", false) => (PROMPT_CV_FR, CV_EXAMPLE),
        ("cv", true) => (PROMPT_CV_EN, CV_EXAMPLE),
        (_, false) => (PROMPT_LM_FR, LM_EXAMPLE),
        (_, true) => (PROMPT_LM_EN, LM_EXAMPLE),
    };
    let contract = if english { CONTRACT_EN } else { CONTRACT_FR };
    // Le lien d'abord, les textes collés ensuite : un `{{…}}` présent dans
    // le contrat ou l'exemple n'est jamais remplacé à son tour.
    Ok(template
        .replace(DOC_URL_PLACEHOLDER, &contract_doc_url(lang))
        .replace(CONTRACT_PLACEHOLDER, contract.trim_end())
        .replace(EXAMPLE_PLACEHOLDER, example.trim_end()))
}
