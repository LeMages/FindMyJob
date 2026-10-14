//! Orchestration du pipeline de génération de documents (CV + lettre de
//! motivation) pour une candidature :
//!
//! 1. analyse de l'offre (LLM) ;
//! 2. matching du profil (LLM), puis sélection des entrées (Rust) ;
//! 3. rédaction de la prose du CV et de la lettre en parallèle (LLM, avec
//!    linter et retry), dans le style de la référence et sous budgets ;
//! 4. assemblage déterministe des données (Rust) et compilation du template
//!    de chaque référence, avec ajustement au nombre de pages.
//!
//! Aucun LLM ne produit de Typst pendant la génération. Rien n'est écrit sur
//! disque ni en base ici : c'est à l'appelant de persister le résultat.

// Importation des modules standard
use std::path::Path;
use std::sync::Arc;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::services::api_key::ApiKeyAccess;
use crate::services::claude_target::ClaudeTargetSource;
use crate::docgen::build_cv_data::{build_cv_data, CvLayout};
use crate::docgen::build_lm_data::{build_lm_data, LmLayout};
use crate::docgen::labels_cache::{resolve_cv_labels, resolve_lm_labels, ResolvedLabels};
use crate::docgen::page_fit::{fit_to_pages, CompileTarget, FittedDocument};
use crate::docgen::recipient::build_recipient;
use crate::docgen::reference_template::{load_reference, ReferenceTemplate};
use crate::docgen::trace::{DocumentTrace, GenerationTrace};
use findmyjob_core::docgen::budgets::{cv_budget, lm_budget, CvBudget, LmBudget};
use findmyjob_core::docgen::period_format::detect_separator;
use findmyjob_core::docgen::selection::{select, Selection};
use findmyjob_core::error::AppError;
use findmyjob_core::llm::cv_writing::{write_cv, CvWritingInput};
use findmyjob_core::llm::lm_writing::{write_lm, LmWritingInput};
use findmyjob_core::llm::matching::match_profile;
use findmyjob_core::llm::offer::analyze_offer;
use findmyjob_core::llm::writing_loop::WritingOutcome;
use findmyjob_core::llm::{ClaudeTarget, LlmCaller};
use findmyjob_core::models::{
    CvContent, CvExampleData, CvLabels, LmContent, LmExampleData, LmLabels, MatchingResult, ProfileDto,
    Recipient,
};

/// Nombre total d'étapes de `generate_documents`, communiqué à l'appelant
/// pour calculer une progression (`current_step`/`total_steps`). Les
/// relances (linter, pages) restent sur leur étape.
pub const GENERATION_STEPS_TOTAL: u8 = 5;

/// Langues de document prises en charge ; une autre valeur d'analyse
/// d'offre retombe sur la langue de la référence.
const SUPPORTED_LANGS: [&str; 2] = ["fr", "en"];

/// Document généré, prêt à être sauvegardé.
///
/// # Attributes
///
/// * `pdf` - Octets du PDF compilé.
/// * `data_json` - Données passées au template (`data.json`).
/// * `reference_version_id` - Version de référence (et de template) utilisée.
pub struct GeneratedDocument {
    pub pdf: Vec<u8>,
    pub data_json: String,
    pub reference_version_id: i64,
}

/// Documents générés pour une candidature, avec leur trace.
pub struct GeneratedDocuments {
    pub cv: GeneratedDocument,
    pub lm: GeneratedDocument,
    pub trace: GenerationTrace,
}

/// Paramètres d'une génération.
///
/// # Attributes
///
/// * `data_dir` - Dossier de données de l'application (fichiers des références).
/// * `session_id` - Session de la candidature (fournit les références).
/// * `candidature_id` - Candidature (fournit le destinataire).
/// * `description_offre` - Texte brut de l'offre.
/// * `api_key` - Accès à la clé API Anthropic.
/// * `llm` - Exécutant des appels à Claude.
pub struct GenerationRequest<'a> {
    pub data_dir: &'a Path,
    pub session_id: i64,
    pub candidature_id: i64,
    pub description_offre: &'a str,
    pub api_key: &'a ApiKeyAccess,
    pub llm: &'a Arc<dyn LlmCaller>,
}

/// Construit le destinataire de la lettre depuis la candidature.
async fn load_recipient(pool: &SqlitePool, candidature_id: i64) -> Result<Recipient, AppError> {
    let candidature = db::candidatures::fetch_detail(pool, candidature_id).await?;
    Ok(build_recipient(&candidature))
}

/// Langue du document : celle de l'offre si elle est prise en charge,
/// sinon celle de la référence.
fn target_language(offer_analysis: &str, fallback: &str) -> String {
    serde_json::from_str::<serde_json::Value>(offer_analysis)
        .ok()
        .and_then(|v| v.get("language").and_then(|l| l.as_str()).map(str::to_string))
        .filter(|lang| SUPPORTED_LANGS.contains(&lang.as_str()))
        .unwrap_or_else(|| fallback.to_string())
}

/// Convertit un document ajusté en trace et en document à sauvegarder.
fn finish<C>(
    fitted: FittedDocument<C>,
    reference_version_id: i64,
    labels_translated: bool,
) -> (GeneratedDocument, DocumentTrace<C>) {
    let trace = DocumentTrace {
        content: fitted.writing.content,
        rounds: fitted.rounds,
        residual_violations: fitted.writing.residual_violations,
        page_count: fitted.page_count,
        page_retries: fitted.page_retries,
        warning: fitted.warning,
        labels_translated,
    };
    (GeneratedDocument { pdf: fitted.pdf, data_json: fitted.data_json, reference_version_id }, trace)
}

/// Tout ce que la rédaction et l'assemblage partagent, préparé une fois.
///
/// # Attributes
///
/// * `profile` - Profil complet du candidat.
/// * `target` - Clé API et modèle Claude configuré.
/// * `cv_ref`, `lm_ref` - Références templatisées.
/// * `recipient` - Destinataire de la lettre.
/// * `offer_analysis` - Analyse de l'offre (JSON).
/// * `lang` - Langue des documents.
/// * `matching`, `selection` - Matching et entrées retenues.
/// * `cv_budget`, `lm_budget` - Budgets initiaux.
/// * `cv_labels`, `lm_labels` - Libellés dans la langue des documents.
struct GenerationContext {
    profile: ProfileDto,
    target: ClaudeTarget,
    cv_ref: ReferenceTemplate<CvExampleData>,
    lm_ref: ReferenceTemplate<LmExampleData>,
    recipient: Recipient,
    offer_analysis: String,
    lang: String,
    matching: MatchingResult,
    selection: Selection,
    cv_budget: CvBudget,
    lm_budget: LmBudget,
    cv_labels: ResolvedLabels<CvLabels>,
    lm_labels: ResolvedLabels<LmLabels>,
}

impl GenerationContext {
    /// Entrées de la rédaction du CV pour un budget donné.
    fn cv_input(&self, budget: CvBudget) -> CvWritingInput<'_> {
        CvWritingInput {
            offer_analysis: &self.offer_analysis,
            matching: &self.matching,
            selection: &self.selection,
            profile: &self.profile,
            example: &self.cv_ref.example.data,
            guidance: &self.cv_ref.example.guidance,
            budget,
            target_lang: &self.lang,
            example_lang: &self.cv_ref.example.detected_lang,
        }
    }

    /// Entrées de la rédaction de la lettre pour un budget donné.
    fn lm_input(&self, budget: LmBudget) -> LmWritingInput<'_> {
        LmWritingInput {
            offer_analysis: &self.offer_analysis,
            matching: &self.matching,
            selection: &self.selection,
            profile: &self.profile,
            example: &self.lm_ref.example.data,
            guidance: &self.lm_ref.example.guidance,
            budget,
            company: &self.recipient.company,
            target_lang: &self.lang,
            example_lang: &self.lm_ref.example.detected_lang,
        }
    }
}

/// Étapes 1 et 2 : profil, références, destinataire, analyse de l'offre,
/// matching, sélection, budgets et libellés.
async fn prepare(
    pool: &SqlitePool,
    request: &GenerationRequest<'_>,
    on_progress: &impl Fn(u8, &str),
) -> Result<GenerationContext, AppError> {
    // ------------------------------------------------------------------
    // 1. Profil, modèle, références templatisées, destinataire
    // ------------------------------------------------------------------
    let profile = db::profile::build_profile(pool).await?;
    let target = ClaudeTargetSource { pool, api_key: request.api_key, llm: request.llm }.resolve().await?;
    let cv_ref: ReferenceTemplate<CvExampleData> =
        load_reference(pool, request.data_dir, request.session_id, "cv", "CV").await?;
    let lm_ref: ReferenceTemplate<LmExampleData> =
        load_reference(pool, request.data_dir, request.session_id, "lettre_motivation", "lettre de motivation").await?;
    let recipient = load_recipient(pool, request.candidature_id).await?;

    // ------------------------------------------------------------------
    // 2. Analyse de l'offre, matching, sélection et budgets
    // ------------------------------------------------------------------
    on_progress(1, "Analyse de l'offre…");
    let offer_analysis = analyze_offer(request.description_offre, &target).await?;
    let lang = target_language(&offer_analysis, &cv_ref.example.detected_lang);

    on_progress(2, "Matching profil / offre…");
    let project_count = cv_ref.example.data.projects.len();
    let matching = match_profile(&offer_analysis, &profile, project_count, &target).await?;
    let selection = select(&profile, &matching, project_count);
    let mut cv_budget = cv_budget(&cv_ref.example.data, cv_ref.page_count).with_fixed_words(&cv_ref.example.guidance);
    // Un profil peut avoir moins de projets que la référence : on en attend
    // autant que la sélection en contient, jamais plus.
    cv_budget.project_count = selection.projects.len();
    // Même logique pour les groupes de compétences : pas plus de groupes que
    // de domaines dans le profil, sinon le modèle doit en inventer un.
    let mut domains: Vec<&str> = profile.skills.iter().map(|s| s.domaine.trim()).collect();
    domains.sort_unstable();
    domains.dedup();
    cv_budget.skills_group_count = cv_budget.skills_group_count.min(domains.len());
    let lm_budget = lm_budget(&lm_ref.example.data, lm_ref.page_count).with_fixed_words(&lm_ref.example.guidance);

    // ------------------------------------------------------------------
    // 3. Libellés dans la langue de l'offre (traduits et mis en cache si besoin)
    // ------------------------------------------------------------------
    let (cv_labels, lm_labels) = tokio::try_join!(
        resolve_cv_labels(&cv_ref.example.data.labels, &cv_ref.example.detected_lang, &lang, &cv_ref.dir, &target),
        resolve_lm_labels(&lm_ref.example.data.labels, &lm_ref.example.detected_lang, &lang, &lm_ref.dir, &target),
    )?;

    Ok(GenerationContext {
        profile,
        target,
        cv_ref,
        lm_ref,
        recipient,
        offer_analysis,
        lang,
        matching,
        selection,
        cv_budget,
        lm_budget,
        cv_labels,
        lm_labels,
    })
}

/// Assemble, compile et ajuste le CV au nombre de pages de la référence.
async fn produce_cv(
    ctx: &GenerationContext,
    writing: WritingOutcome<CvContent>,
    fonts_dir: &Path,
) -> Result<FittedDocument<CvContent>, AppError> {
    let separator = detect_separator(ctx.cv_ref.example.data.experiences.iter().map(|e| e.period.as_str()));
    let layout = CvLayout {
        document: &ctx.cv_ref.example.data,
        labels: &ctx.cv_labels.labels,
        period_style: &ctx.cv_ref.example.period_style,
        separator: &separator,
        lang: &ctx.lang,
    };
    let target = CompileTarget {
        label: "CV",
        template: &ctx.cv_ref.template,
        root: &ctx.cv_ref.dir,
        fonts_dir,
        max_pages: ctx.cv_ref.page_count,
    };
    fit_to_pages(
        &target,
        writing,
        |content| build_cv_data(&ctx.profile, &ctx.selection, content, &layout),
        |factor| {
            let input = ctx.cv_input(ctx.cv_budget.scaled(factor));
            async move { write_cv(&input, &ctx.target).await }
        },
    )
    .await
}

/// Assemble, compile et ajuste la lettre au nombre de pages de la référence.
/// `title` est le sous-titre professionnel du CV final, repris tel quel.
async fn produce_lm(
    ctx: &GenerationContext,
    writing: WritingOutcome<LmContent>,
    title: &str,
    fonts_dir: &Path,
) -> Result<FittedDocument<LmContent>, AppError> {
    let layout = LmLayout {
        document: &ctx.lm_ref.example.data,
        labels: &ctx.lm_labels.labels,
        title,
        date: chrono::Local::now().date_naive(),
        lang: &ctx.lang,
    };
    let target = CompileTarget {
        label: "lettre",
        template: &ctx.lm_ref.template,
        root: &ctx.lm_ref.dir,
        fonts_dir,
        max_pages: ctx.lm_ref.page_count,
    };
    fit_to_pages(
        &target,
        writing,
        |content| build_lm_data(&ctx.profile, &ctx.recipient, content, &layout),
        |factor| {
            let input = ctx.lm_input(ctx.lm_budget.scaled(factor));
            async move { write_lm(&input, &ctx.target).await }
        },
    )
    .await
}

/// Construit le résultat et sa trace.
fn into_result(
    ctx: GenerationContext,
    cv_fitted: FittedDocument<CvContent>,
    lm_fitted: FittedDocument<LmContent>,
) -> GeneratedDocuments {
    let (cv, cv_trace) = finish(cv_fitted, ctx.cv_ref.version_id, ctx.cv_labels.translated);
    let (lm, lm_trace) = finish(lm_fitted, ctx.lm_ref.version_id, ctx.lm_labels.translated);
    let offer_analysis = serde_json::from_str(&ctx.offer_analysis)
        .unwrap_or_else(|_| serde_json::Value::String(ctx.offer_analysis.clone()));
    let trace = GenerationTrace {
        offer_analysis,
        experience_ids: ctx.selection.experience_ids(),
        project_ids: ctx.selection.project_ids(),
        matching: ctx.matching,
        cv_budget: ctx.cv_budget,
        lm_budget: ctx.lm_budget,
        cv: cv_trace,
        lm: lm_trace,
    };
    GeneratedDocuments { cv, lm, trace }
}

/// Génère le CV et la lettre de motivation adaptés à une offre.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
/// * `request` - Session, candidature et texte de l'offre.
/// * `on_progress` - Callback appelée à chaque étape (`step` 1-based, libellé).
///
/// # Returns
///
/// `GeneratedDocuments`.
pub async fn generate_documents(
    pool: &SqlitePool,
    fonts_dir: &Path,
    request: GenerationRequest<'_>,
    on_progress: impl Fn(u8, &str),
) -> Result<GeneratedDocuments, AppError> {
    let ctx = prepare(pool, &request, &on_progress).await?;

    on_progress(3, "Rédaction du CV et de la lettre…");
    let (cv_input, lm_input) = (ctx.cv_input(ctx.cv_budget.clone()), ctx.lm_input(ctx.lm_budget.clone()));
    let (cv_writing, lm_writing) =
        tokio::try_join!(write_cv(&cv_input, &ctx.target), write_lm(&lm_input, &ctx.target))?;

    on_progress(4, "Assemblage et compilation du CV…");
    let cv_fitted = produce_cv(&ctx, cv_writing, fonts_dir).await?;

    on_progress(5, "Assemblage et compilation de la lettre…");
    let cv_title = cv_fitted.writing.content.title.clone();
    let lm_fitted = produce_lm(&ctx, lm_writing, &cv_title, fonts_dir).await?;

    Ok(into_result(ctx, cv_fitted, lm_fitted))
}
