//! Cascade d'extraction des champs d'une offre à partir de son HTML brut,
//! partagée par la récupération par lien et la fenêtre de capture, ainsi
//! que l'extraction depuis un texte collé (voir `services::scraping`).
//!
//! Étage A : JSON-LD `JobPosting`, sans LLM. Étage B, si A échoue : texte
//! isolé de la page + indices de titre, analysés par le LLM. Les champs
//! renvoyés passent toujours par le nettoyage final de `core`.

// Importation des modules internes
use crate::services::claude_target::ClaudeTargetSource;
use crate::error::AppError;
use crate::llm::scraping_extract::extract_offer_fields;
use crate::llm::ClaudeTarget;
use crate::models::{ExtractionStage, OfferExtraction};
use crate::scraping::extract::{extract_page_text, truncate_chars, MAX_CHARS};
use crate::scraping::finalize::finalize_fields;
use crate::scraping::json_ld::extract_job_posting_fields;
use crate::scraping::page_hints::PageHints;

/// Extrait les champs du formulaire de candidature depuis le HTML d'une page
/// d'offre, en essayant le JSON-LD avant le LLM.
///
/// # Arguments
///
/// * `html` - HTML brut de la page (récupéré par lien ou capturé).
/// * `claude` - Source de la clé et du modèle, résolus seulement si l'étage
///   LLM est atteint : un import par JSON-LD n'exige pas de clé.
///
/// # Returns
///
/// Les champs nettoyés et l'étage utilisé, ou l'erreur typée de l'étage
/// LLM (contenu insuffisant, erreur d'appel).
pub async fn run_cascade(html: &str, claude: ClaudeTargetSource<'_>) -> Result<OfferExtraction, AppError> {
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
    let target = claude.resolve().await?;
    let fields = extract_offer_fields(&page.text, &page.hints, &target).await?;
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
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Les champs nettoyés (étage `llm`), ou l'erreur de l'appel LLM.
pub async fn extract_from_text(text: &str, target: &ClaudeTarget) -> Result<OfferExtraction, AppError> {
    let text = truncate_chars(text, MAX_CHARS);
    let fields = extract_offer_fields(&text, &PageHints::default(), target).await?;
    Ok(OfferExtraction {
        fields: finalize_fields(fields),
        stage: ExtractionStage::Llm,
    })
}
