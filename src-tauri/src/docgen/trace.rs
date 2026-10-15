//! Traçabilité d'une génération : tout ce qui a mené aux documents
//! (analyse d'offre, matching, sélection, budgets, contenus rédigés,
//! rapports du linter, essais), écrit dans un dossier de debug.

// Importation des crates externes
use serde::Serialize;
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::budgets::{CvBudget, LmBudget};
use crate::docgen::lint::Violation;
use crate::error::AppError;
use crate::llm::writing_loop::AttemptReport;
use crate::models::{CvContent, LmContent, MatchingResult};

/// Trace d'un document généré.
///
/// # Attributes
///
/// * `content` - Contenu rédigé retenu.
/// * `rounds` - Essais de chaque rédaction (initiale, puis relances pour les pages).
/// * `residual_violations` - Violations du linter conservées.
/// * `page_count` - Nombre de pages du PDF.
/// * `page_retries` - Relances dues au dépassement de pages.
/// * `warning` - Avertissement éventuel (pages dépassées).
/// * `labels_translated` - Vrai si les libellés ont été traduits.
#[derive(Serialize)]
pub struct DocumentTrace<C> {
    pub content: C,
    pub rounds: Vec<Vec<AttemptReport>>,
    pub residual_violations: Vec<Violation>,
    pub page_count: usize,
    pub page_retries: u8,
    pub warning: Option<String>,
    pub labels_translated: bool,
}

/// Trace complète d'une génération.
///
/// # Attributes
///
/// * `offer_analysis` - Analyse de l'offre (JSON).
/// * `matching` - Résultat du matching.
/// * `experience_ids`, `project_ids` - Entrées retenues, dans l'ordre d'affichage.
/// * `cv_budget`, `lm_budget` - Budgets initiaux.
/// * `cv`, `lm` - Trace de chaque document.
pub struct GenerationTrace {
    pub offer_analysis: Value,
    pub matching: MatchingResult,
    pub experience_ids: Vec<i64>,
    pub project_ids: Vec<i64>,
    pub cv_budget: CvBudget,
    pub lm_budget: LmBudget,
    pub cv: DocumentTrace<CvContent>,
    pub lm: DocumentTrace<LmContent>,
}

/// Résumé des essais d'un document, pour `attempts.json`.
fn attempts_of<C>(doc: &DocumentTrace<C>) -> Value {
    json!({
        "writing_rounds": doc.rounds,
        "writing_calls": doc.rounds.iter().map(Vec::len).sum::<usize>(),
        "page_retries": doc.page_retries,
        "page_count": doc.page_count,
        "warning": doc.warning,
        "labels_translated": doc.labels_translated,
    })
}

impl GenerationTrace {
    /// Fichiers du dossier de debug : nom et contenu JSON.
    ///
    /// # Returns
    ///
    /// Liste `(nom de fichier, contenu)`.
    pub fn files(&self) -> Result<Vec<(&'static str, String)>, AppError> {
        let entries: Vec<(&'static str, Value)> = vec![
            ("offer_analysis.json", self.offer_analysis.clone()),
            ("matching.json", to_value(&self.matching)?),
            ("selection.json", json!({ "experience_ids": self.experience_ids, "project_ids": self.project_ids })),
            ("budgets.json", json!({ "cv": to_value(&self.cv_budget)?, "lm": to_value(&self.lm_budget)? })),
            ("cv_content.json", to_value(&self.cv.content)?),
            ("lm_content.json", to_value(&self.lm.content)?),
            (
                "lint_report.json",
                json!({ "cv": to_value(&self.cv.residual_violations)?, "lm": to_value(&self.lm.residual_violations)? }),
            ),
            ("attempts.json", json!({ "cv": attempts_of(&self.cv), "lm": attempts_of(&self.lm) })),
        ];
        entries
            .into_iter()
            .map(|(name, value)| {
                serde_json::to_string_pretty(&value)
                    .map(|text| (name, text))
                    .map_err(|e| AppError::Validation(format!("Trace {name} non sérialisable : {e}")))
            })
            .collect()
    }
}

/// Convertit une valeur sérialisable en JSON.
fn to_value<T: Serialize>(value: &T) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(|e| AppError::Validation(format!("Trace non sérialisable : {e}")))
}
