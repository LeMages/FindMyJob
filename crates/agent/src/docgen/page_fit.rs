//! Compilation d'un document généré avec son template, et ajustement au
//! nombre de pages de la référence : si le PDF dépasse, la rédaction est
//! relancée avec des budgets réduits de 15 % (cumulés), deux fois au
//! maximum, puis le document est conservé avec un avertissement.

// Importation des modules standard
use std::future::Future;
use std::path::Path;

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::documents::typst::compile_template;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::writing_loop::{AttemptReport, WritingOutcome};

/// Nombre maximal de relances de la rédaction pour tenir dans les pages.
const MAX_PAGE_RETRIES: u8 = 2;
/// Facteur appliqué aux budgets à chaque relance.
const BUDGET_REDUCTION: f64 = 0.85;

/// Document compilé, avec l'historique qui y a mené.
///
/// # Attributes
///
/// * `pdf` - Octets du PDF.
/// * `data_json` - Données passées au template (`data.json`).
/// * `page_count` - Nombre de pages du PDF.
/// * `page_retries` - Nombre de relances dues au dépassement de pages.
/// * `warning` - Avertissement si le document dépasse encore ses pages.
/// * `writing` - Rédaction finalement retenue.
/// * `rounds` - Essais de chaque rédaction (initiale puis relances).
pub struct FittedDocument<C> {
    pub pdf: Vec<u8>,
    pub data_json: String,
    pub page_count: usize,
    pub page_retries: u8,
    pub warning: Option<String>,
    pub writing: WritingOutcome<C>,
    pub rounds: Vec<Vec<AttemptReport>>,
}

/// Ce qui est compilé et contre quoi.
///
/// # Attributes
///
/// * `label` - Nom du document, pour les messages.
/// * `template` - Source du template.
/// * `root` - Dossier racine de compilation.
/// * `fonts_dir` - Dossier des polices importées.
/// * `max_pages` - Nombre de pages à ne pas dépasser.
pub struct CompileTarget<'a> {
    pub label: &'a str,
    pub template: &'a str,
    pub root: &'a Path,
    pub fonts_dir: &'a Path,
    pub max_pages: usize,
}

/// Compile des données avec le template (hors du runtime async).
async fn compile(target: &CompileTarget<'_>, data_json: &str) -> Result<(Vec<u8>, usize), AppError> {
    let (template, data) = (target.template.to_string(), data_json.as_bytes().to_vec());
    let (root, fonts) = (target.root.to_path_buf(), target.fonts_dir.to_path_buf());
    let compiled = tokio::task::spawn_blocking(move || compile_template(template, data, root, fonts))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne de compilation Typst : {e}")))?
        .map_err(|e| AppError::Validation(format!("Compilation du {} impossible : {e}", target.label)))?;
    Ok((compiled.pdf, compiled.page_count))
}

/// Compile un document et l'ajuste au nombre de pages de la référence.
///
/// # Arguments
///
/// * `target` - Template, racine, polices et plafond de pages.
/// * `initial` - Rédaction initiale.
/// * `build` - Construit les données du template depuis un contenu rédigé.
/// * `rewrite` - Relance la rédaction avec un facteur de réduction des budgets.
///
/// # Returns
///
/// `FittedDocument`.
pub async fn fit_to_pages<C, D, Fut>(
    target: &CompileTarget<'_>,
    initial: WritingOutcome<C>,
    build: impl Fn(&C) -> D,
    rewrite: impl Fn(f64) -> Fut,
) -> Result<FittedDocument<C>, AppError>
where
    D: Serialize,
    Fut: Future<Output = Result<WritingOutcome<C>, AppError>>,
{
    let mut writing = initial;
    let mut rounds = vec![writing.attempts.clone()];
    let mut page_retries = 0u8;

    loop {
        let data_json = serde_json::to_string_pretty(&build(&writing.content))
            .map_err(|e| AppError::Validation(format!("Données du {} non sérialisables : {e}", target.label)))?;
        let (pdf, page_count) = compile(target, &data_json).await?;

        let fits = page_count <= target.max_pages;
        if fits || page_retries >= MAX_PAGE_RETRIES {
            let warning = (!fits).then(|| {
                format!(
                    "Le {} fait {page_count} page(s) pour {} attendue(s), malgré {MAX_PAGE_RETRIES} réductions des budgets.",
                    target.label, target.max_pages
                )
            });
            if let Some(w) = &warning {
                eprintln!("[page_fit] {w}");
            }
            return Ok(FittedDocument { pdf, data_json, page_count, page_retries, warning, writing, rounds });
        }

        page_retries += 1;
        let factor = BUDGET_REDUCTION.powi(i32::from(page_retries));
        eprintln!(
            "[page_fit] {} : {page_count} page(s) pour {} attendue(s), relance avec budgets × {factor:.2}",
            target.label, target.max_pages
        );
        writing = rewrite(factor).await?;
        rounds.push(writing.attempts.clone());
    }
}
