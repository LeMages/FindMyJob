//! Validation déterministe d'une référence au contrat FindMyJob, sans
//! aucun appel LLM. Ordre des contrôles :
//!
//! 1. fichier : extension, taille, encodage ; constructions interdites ;
//! 2. compilation seule (valeurs d'exemple) : erreurs situées, polices ;
//! 3. métadonnée `<findmyjob>` : unique, contrat, version, langues,
//!    format des périodes ; exemple conforme au schéma ; emplacements ;
//! 4. rendus de contrôle (`render_data`) : chaque langue (pages, tirets),
//!    données vides, sentinelles (valeurs écrites en dur), champs
//!    secondaires vides (séparateurs), listes vides (titres orphelins),
//!    symbole de liste, destinataire réduit.
//!
//! Tous les problèmes trouvés sont rapportés ensemble ; une même erreur de
//! compilation n'est citée qu'une fois.

// Importation des modules standard
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::example_facts::{cv_information_values, lm_information_values};
use crate::docgen::text_diff::orphan_separators;
use crate::documents::reference_contract::example_schema::{shape_for, shape_issues};
use crate::documents::reference_contract::metadata::read_metadata;
use crate::documents::reference_contract::render_checks::{
    dash_issue, hardcoded_issues, list_marker_issue, orphan_heading_issues, recipient_issue,
};
use crate::documents::reference_contract::render_data::{control_renders, single_field_cleared, RenderKind};
use crate::documents::reference_contract::slots::check_slots;
use crate::documents::reference_contract::source_rules::{file_issues, forbidden_constructs};
use crate::documents::typst::{CompileFailure, CompiledDocument, ContractEngine};
use crate::error::AppError;
use crate::models::doc_lang::is_supported;
use crate::models::{CvData, LmData, RawMetadata, ReferenceIssue, ReferenceIssueCode, Slot, StoredContract};
use crate::trace_log;

/// Référence acceptée.
///
/// # Attributes
///
/// * `contract` - Contrat extrait, à enregistrer avec la version.
/// * `pdf` - PDF du rendu de l'exemple (aperçu de la version).
/// * `warnings` - Avertissements (l'import réussit malgré eux).
#[derive(Debug)]
pub struct ValidatedReference {
    pub contract: StoredContract<Value>,
    pub pdf: Vec<u8>,
    pub warnings: Vec<ReferenceIssue>,
}

/// Problèmes collectés, avec les erreurs de compilation déjà citées.
#[derive(Default)]
struct Report {
    issues: Vec<ReferenceIssue>,
    seen_diagnostics: HashSet<(Option<usize>, String)>,
}

impl Report {
    /// Ajoute les problèmes d'un échec de compilation, sauf ceux déjà cités.
    fn compile_failure(&mut self, failure: &CompileFailure, code: ReferenceIssueCode, render: Option<RenderKind>) {
        let at_render = |issue: ReferenceIssue| match render {
            Some(kind) => issue.at_path(kind.name()),
            None => issue,
        };
        match failure {
            CompileFailure::Source(diagnostics) => {
                for diagnostic in diagnostics {
                    if self.seen_diagnostics.insert((diagnostic.line, diagnostic.message.clone())) {
                        let issue = ReferenceIssue::new(code).at_line(diagnostic.line).with_detail(&diagnostic.message);
                        self.issues.push(at_render(issue));
                    }
                }
            }
            CompileFailure::MissingFonts(families) => {
                for family in families {
                    if self.seen_diagnostics.insert((None, family.clone())) {
                        self.issues.push(ReferenceIssue::new(ReferenceIssueCode::MissingFont).with_detail(family));
                    }
                }
            }
            CompileFailure::Other(message) => self.issues.push(at_render(ReferenceIssue::new(code).with_detail(message))),
        }
    }

    /// Vrai si un problème bloquant a été relevé.
    fn has_blocking(&self) -> bool {
        self.issues.iter().any(ReferenceIssue::is_blocking)
    }

    /// Rejet de la référence avec tous ses problèmes.
    fn reject(self) -> AppError {
        AppError::InvalidReference(self.issues)
    }
}

/// Valide une référence et en extrait le contrat.
///
/// # Arguments
///
/// * `filename` - Nom du fichier importé.
/// * `content` - Source Typst.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
///
/// # Returns
///
/// `ValidatedReference`, ou `AppError::InvalidReference` avec tous les
/// problèmes relevés (avertissements compris).
pub async fn validate_reference(
    filename: &str,
    content: &str,
    doc_type: &str,
    fonts_dir: &Path,
) -> Result<ValidatedReference, AppError> {
    // ------------------------------------------------------------------
    // 1. Fichier et constructions interdites : rien n'est compilé avant
    // ------------------------------------------------------------------
    let mut report = Report::default();
    report.issues.extend(file_issues(filename, content));
    if report.issues.is_empty() {
        report.issues.extend(forbidden_constructs(content));
    }
    if report.has_blocking() {
        return Err(report.reject());
    }

    // ------------------------------------------------------------------
    // 2. Compilation seule, puis métadonnée, exemple et emplacements
    // ------------------------------------------------------------------
    let (template, fonts) = (content.to_string(), fonts_dir.to_path_buf());
    let (engine, base) = blocking(move || {
        let engine = ContractEngine::new(&template, &fonts);
        let base = engine.compile(None);
        (engine, base)
    })
    .await?;
    let base = match base {
        Ok(base) => base,
        Err(failure) => {
            report.compile_failure(&failure, ReferenceIssueCode::CompileError, None);
            return Err(report.reject());
        }
    };
    let reading = read_metadata(&base.metadata, doc_type);
    report.issues.extend(reading.issues);
    let Some(metadata) = reading.metadata else {
        return Err(report.reject());
    };
    let example_issues = example_issues(&metadata.example, doc_type);
    if !example_issues.is_empty() {
        report.issues.extend(example_issues);
        return Err(report.reject());
    }
    let (slots, slot_issues) = check_slots(&metadata.slots, doc_type, &metadata.example);
    report.issues.extend(slot_issues);

    // ------------------------------------------------------------------
    // 3. Rendus de contrôle
    // ------------------------------------------------------------------
    let renders = control_renders(&metadata.example, doc_type, &base.text)?;
    let (engine, compiled) = blocking(move || {
        let compiled: Vec<(RenderKind, Result<CompiledDocument, CompileFailure>)> =
            renders.into_iter().map(|(kind, data)| (kind, engine.compile(Some(&data)))).collect();
        (engine, compiled)
    })
    .await?;
    let checks = RenderChecks { base: &base, example: &metadata.example, doc_type };
    let mut example_text = BTreeMap::new();
    let mut sparse_has_orphans = false;
    for (kind, rendered) in &compiled {
        match rendered {
            Ok(doc) => {
                if let RenderKind::Language(lang) = kind {
                    example_text.insert(lang.to_string(), doc.text.clone());
                }
                sparse_has_orphans |= *kind == RenderKind::Sparse && !orphan_separators(&doc.text).is_empty();
                report.issues.extend(checks.issues(*kind, doc));
            }
            Err(failure) => report.compile_failure(failure, failure_code(*kind), Some(*kind)),
        }
    }
    if sparse_has_orphans {
        report.issues.extend(separator_issues(engine, &metadata.example).await?);
    }

    // ------------------------------------------------------------------
    // 4. Verdict
    // ------------------------------------------------------------------
    if report.has_blocking() {
        return Err(report.reject());
    }
    let contract = stored_contract(metadata, slots, base.page_count, example_text, doc_type);
    Ok(ValidatedReference { contract, pdf: base.pdf, warnings: report.issues })
}

/// Exécute une compilation hors du runtime async.
async fn blocking<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static) -> Result<T, AppError> {
    tokio::task::spawn_blocking(job)
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne de compilation Typst : {e}")))
}

/// Problèmes de l'exemple : schéma, puis langue prise en charge.
fn example_issues(example: &Value, doc_type: &str) -> Vec<ReferenceIssue> {
    let issues = shape_issues(example, &shape_for(doc_type), "example");
    if !issues.is_empty() {
        return issues;
    }
    let lang = example["lang"].as_str().unwrap_or_default();
    if is_supported(lang) {
        return Vec::new();
    }
    vec![ReferenceIssue::new(ReferenceIssueCode::ExampleInvalid)
        .at_path("example.lang")
        .with_detail(format!("unsupported language: {lang}"))]
}

/// Code d'un échec de compilation selon le rendu.
fn failure_code(kind: RenderKind) -> ReferenceIssueCode {
    match kind {
        RenderKind::Language(_) => ReferenceIssueCode::LanguageCompileError,
        RenderKind::Blank => ReferenceIssueCode::EmptyDataCompileError,
        _ => ReferenceIssueCode::CompileError,
    }
}

/// Contexte commun des contrôles sur les rendus.
struct RenderChecks<'a> {
    base: &'a CompiledDocument,
    example: &'a Value,
    doc_type: &'a str,
}

impl RenderChecks<'_> {
    /// Problèmes d'un rendu de contrôle réussi.
    fn issues(&self, kind: RenderKind, doc: &CompiledDocument) -> Vec<ReferenceIssue> {
        match kind {
            RenderKind::Language(lang) => {
                let mut issues: Vec<ReferenceIssue> = dash_issue(lang, &doc.text).into_iter().collect();
                if doc.page_count > self.base.page_count {
                    issues.push(
                        ReferenceIssue::new(ReferenceIssueCode::PageCountMismatch)
                            .with_detail(format!("{lang}: {} > {}", doc.page_count, self.base.page_count)),
                    );
                }
                issues
            }
            RenderKind::Sentinel => hardcoded_issues(doc, &self.information_values()),
            RenderKind::EmptyLists => orphan_heading_issues(doc),
            RenderKind::MarkerSentinel(marker) => list_marker_issue(doc, self.example, marker).into_iter().collect(),
            RenderKind::SparseRecipient => recipient_issue(self.base, self.example, doc).into_iter().collect(),
            RenderKind::Blank | RenderKind::Sparse => Vec::new(),
        }
    }

    /// Valeurs d'exemple d'identité et du profil (jamais écrites en dur).
    fn information_values(&self) -> Vec<String> {
        // L'exemple suit déjà le schéma : la désérialisation ne peut échouer
        // que sur un bug, tracé ; aucune valeur n'est alors retenue (rien
        // n'est signalé à tort).
        let values = if self.doc_type == "cv" {
            serde_json::from_value::<CvData>(self.example.clone()).map(|d| cv_information_values(&d))
        } else {
            serde_json::from_value::<LmData>(self.example.clone()).map(|d| lm_information_values(&d))
        };
        values.unwrap_or_else(|e| {
            trace_log::log(&format!("[reference_contract] exemple conforme mais illisible : {e}"));
            Vec::new()
        })
    }
}

/// Séparateurs orphelins, champ secondaire par champ secondaire.
async fn separator_issues(engine: ContractEngine, example: &Value) -> Result<Vec<ReferenceIssue>, AppError> {
    let data_sets = single_field_cleared(example)?;
    let found = blocking(move || {
        data_sets
            .into_iter()
            .filter_map(|(field, data)| {
                // Le rendu avec tous les champs vides compile : un échec ici
                // ne désigne pas un séparateur orphelin, le champ est ignoré.
                let rendered = engine.compile(Some(&data)).ok()?;
                let orphans = orphan_separators(&rendered.text);
                (!orphans.is_empty()).then_some((field, orphans))
            })
            .collect::<Vec<_>>()
    })
    .await?;
    if found.is_empty() {
        // Orphelins qui n'apparaissent qu'en vidant plusieurs champs à la fois.
        return Ok(vec![ReferenceIssue::new(ReferenceIssueCode::OrphanSeparator)]);
    }
    Ok(found
        .into_iter()
        .map(|(field, orphans)| {
            let mut unique: Vec<String> = Vec::new();
            for orphan in orphans {
                if !unique.contains(&orphan) {
                    unique.push(orphan);
                }
            }
            ReferenceIssue::new(ReferenceIssueCode::OrphanSeparator).at_path(field).with_detail(unique.join("  "))
        })
        .collect())
}

/// Contrat à enregistrer avec la version.
fn stored_contract(
    metadata: RawMetadata,
    slots: Vec<Slot>,
    page_count: usize,
    example_text: BTreeMap<String, String>,
    doc_type: &str,
) -> StoredContract<Value> {
    StoredContract {
        contract_version: metadata.version,
        languages: metadata.languages,
        example: metadata.example,
        guidance: metadata.guidance.trim().to_string(),
        slots,
        // Le format des périodes ne concerne que le CV.
        period: if doc_type == "cv" { metadata.period } else { None },
        page_count,
        example_text,
    }
}
