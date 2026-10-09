//! Cascade d'extraction des champs d'une offre à partir de son HTML brut,
//! partagée par la récupération par lien (`POST /api/scraping/url`) et la
//! fenêtre de capture du desktop (`POST /api/scraping/html`), ainsi que
//! l'extraction depuis un texte collé (`POST /api/scraping/text`).
//!
//! Étage A : JSON-LD `JobPosting`, sans LLM. Étage B, si A échoue : texte
//! isolé de la page + indices de titre, analysés par le LLM. Les champs
//! renvoyés passent toujours par le nettoyage final de `core`.

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::llm::scraping_extract::extract_offer_fields;
use findmyjob_core::models::{ExtractionStage, OfferExtraction};
use findmyjob_core::scraping::extract::{extract_page_text, truncate_chars, MAX_CHARS};
use findmyjob_core::scraping::finalize::finalize_fields;
use findmyjob_core::scraping::json_ld::extract_job_posting_fields;
use findmyjob_core::scraping::page_hints::PageHints;

/// Extrait les champs du formulaire de candidature depuis le HTML d'une page
/// d'offre, en essayant le JSON-LD avant le LLM.
///
/// # Arguments
///
/// * `html` - HTML brut de la page (récupéré par l'agent ou capturé).
/// * `model` - Modèle Claude à utiliser pour l'étage LLM.
///
/// # Returns
///
/// Les champs nettoyés et l'étage utilisé, ou l'erreur typée de l'étage
/// LLM (contenu insuffisant, erreur d'appel).
pub async fn run_cascade(html: &str, model: &str) -> Result<OfferExtraction, AppError> {
    // ------------------------------------------------------------------
    // 1. Étage A — JSON-LD (le HTML brut doit encore contenir ses scripts)
    // ------------------------------------------------------------------
    if let Some(fields) = extract_job_posting_fields(html) {
        return Ok(OfferExtraction {
            fields,
            stage: ExtractionStage::JsonLd,
        });
    }

    // ------------------------------------------------------------------
    // 2. Étage B — texte isolé de la page + LLM
    // ------------------------------------------------------------------
    let page = extract_page_text(html)?;
    let fields = extract_offer_fields(&page.text, &page.hints, model).await?;
    Ok(OfferExtraction {
        fields: finalize_fields(fields),
        stage: ExtractionStage::Llm,
    })
}

/// Extrait les champs d'une offre depuis un texte collé par l'utilisateur.
///
/// # Arguments
///
/// * `text` - Texte de l'offre, tronqué à `MAX_CHARS` avant l'appel LLM.
/// * `model` - Modèle Claude à utiliser.
///
/// # Returns
///
/// Les champs nettoyés (étage `llm`), ou l'erreur de l'appel LLM.
pub async fn extract_from_text(text: &str, model: &str) -> Result<OfferExtraction, AppError> {
    let text = truncate_chars(text, MAX_CHARS);
    let fields = extract_offer_fields(&text, &PageHints::default(), model).await?;
    Ok(OfferExtraction {
        fields: finalize_fields(fields),
        stage: ExtractionStage::Llm,
    })
}
