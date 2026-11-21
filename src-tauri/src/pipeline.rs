//! Orchestration du pipeline de génération de documents (CV + lettre de
//! motivation) pour une candidature :
//!
//! 1. analyse de l'offre (LLM) ;
//! 2. matching du profil (LLM, choix des projets validé en Rust), puis
//!    sélection des entrées (Rust) et, si la langue du CV diffère de celle
//!    du profil, traduction fidèle de leurs descriptions (LLM, validée) ;
//! 3. rédaction de la prose du CV (titre, résumé, compétences) et de la
//!    lettre en parallèle (LLM, avec linter et retry), dans le style de la
//!    référence et sous budgets ;
//! 4. assemblage déterministe des données (Rust), normalisation des tirets
//!    de toutes leurs chaînes et compilation de chaque référence avec ses
//!    données (`sys.inputs.data`), avec ajustement au nombre de pages.
//!
//! Les libellés et textes fixes d'une référence sont écrits dans son
//! template, dans chaque langue prise en charge : seule la langue (`data.lang`)
//! les choisit, sans traduction. Aucun LLM ne produit de Typst pendant la
//! génération. Rien n'est écrit sur disque ni en base ici : c'est à
//! l'appelant de persister le résultat.

// Importation des modules standard
use std::path::Path;
use std::sync::Arc;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::clock;
use crate::db;
use crate::services::api_key::ApiKeyAccess;
use crate::services::claude_target::ClaudeTargetSource;
use crate::docgen::build_cv_data::{build_cv_data, CvLayout};
use crate::docgen::build_lm_data::{build_lm_data, LmLayout};
use crate::docgen::contact_fields::{missing_contacts, ContactField};
use crate::docgen::identity::{validate_sender_name, SenderLocation};
use crate::documents::page_fit::{fit_to_pages, CompileTarget, FittedDocument};
use crate::docgen::recipient::build_recipient;
use crate::documents::reference_template::{load_reference, ReferenceTemplate};
use crate::documents::generation_trace::{DocumentTrace, GenerationTrace};
use crate::docgen::budgets::{cv_budget, lm_budget, CvBudget, LmBudget};
use crate::docgen::description_list::list_marker;
use crate::docgen::entry_texts::{source_texts, CvEntryTexts};
use crate::docgen::period_format::PeriodFormat;
use crate::docgen::profile_categories::{missing_categories, ProfileCategory};
use crate::docgen::selection::{expected_project_count, select, Selection};
use crate::docgen::verbatim_check::verbatim_violations;
use crate::error::AppError;
use crate::llm::detailed_context_block::letter_contexts;
use crate::llm::cv_writing::{write_cv, CvWritingInput};
use crate::llm::lm_writing::{write_lm, LmWritingInput};
use crate::llm::matching::match_profile;
use crate::llm::offer::analyze_offer;
use crate::llm::profile_translation::translate_entry_texts;
use crate::llm::writing_loop::WritingOutcome;
use crate::llm::{CallRecorder, ClaudeTarget, LlmCaller};
use crate::models::doc_lang::is_supported;
use crate::trace_log;
use crate::models::{
    CvContent, CvData, LmContent, LmData, MatchingResult, PeriodSpec, ProfileDto, ProseGuidance, Recipient,
    StoredContract,
};

/// Nombre total d'étapes de `generate_documents`, communiqué à l'appelant
/// pour calculer une progression (`current_step`/`total_steps`). Les
/// relances (linter, pages) restent sur leur étape.
pub const GENERATION_STEPS_TOTAL: u8 = 5;

/// Document généré, prêt à être sauvegardé.
///
/// # Attributes
///
/// * `pdf` - Octets du PDF compilé.
/// * `data_json` - Données passées au template (`sys.inputs.data`).
/// * `reference_version_id` - Version de référence (et de template) utilisée.
pub struct GeneratedDocument {
    pub pdf: Vec<u8>,
    pub data_json: String,
    pub reference_version_id: i64,
}

/// Documents générés pour une candidature, avec leur trace.
///
/// # Attributes
///
/// * `cv`, `lm` - Documents à sauvegarder.
/// * `warnings` - Avertissements pour l'utilisateur (pages dépassées…).
/// * `trace` - Trace de la génération (dossier de debug).
pub struct GeneratedDocuments {
    pub cv: GeneratedDocument,
    pub lm: GeneratedDocument,
    pub warnings: Vec<String>,
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

/// Construit le destinataire de la lettre depuis la candidature, avec
/// l'intitulé exact du poste.
async fn load_recipient(pool: &SqlitePool, candidature_id: i64) -> Result<(Recipient, String), AppError> {
    let candidature = db::candidatures::fetch_detail(pool, candidature_id).await?;
    Ok((build_recipient(&candidature), candidature.poste.trim().to_string()))
}

/// Langue d'un document : celle du corps de l'offre si elle est prise en
/// charge, sinon (offre « inconnue » ou illisible) celle de sa référence.
fn target_language(offer_analysis: &str, reference_lang: &str) -> String {
    serde_json::from_str::<serde_json::Value>(offer_analysis)
        .ok()
        .and_then(|v| v.get("language").and_then(|l| l.as_str()).map(str::to_string))
        .filter(|lang| is_supported(lang))
        .unwrap_or_else(|| reference_lang.to_string())
}

/// Convertit un document ajusté en trace et en document à sauvegarder.
fn finish<C>(fitted: FittedDocument<C>, reference_version_id: i64) -> (GeneratedDocument, DocumentTrace<C>) {
    let trace = DocumentTrace {
        content: fitted.writing.content,
        rounds: fitted.rounds,
        residual_violations: fitted.writing.residual_violations,
        page_count: fitted.page_count,
        page_retries: fitted.page_retries,
        warning: fitted.warning,
        rendered_dash_marks: fitted.rendered_dash_marks,
        typst_warnings: fitted.typst_warnings,
    };
    (GeneratedDocument { pdf: fitted.pdf, data_json: fitted.data_json, reference_version_id }, trace)
}

/// Ce que la rédaction d'un document reprend de sa référence et de sa
/// langue.
///
/// # Attributes
///
/// * `lang` - Langue du document.
/// * `entry_texts` - Textes du profil dans cette langue.
/// * `guidance` - Consignes et emplacements déclarés par la référence.
/// * `reference_text` - Texte rendu de la référence avec son exemple, dans
///   la langue du document (parties fixes comprises), sinon dans celle de
///   l'exemple.
struct DocumentLanguage {
    lang: String,
    entry_texts: CvEntryTexts,
    guidance: ProseGuidance,
    reference_text: String,
}

impl DocumentLanguage {
    /// Prépare la langue et les consignes d'un document.
    ///
    /// # Arguments
    ///
    /// * `lang` - Langue du document.
    /// * `entry_texts` - Textes du profil dans cette langue.
    /// * `contract` - Contrat enregistré de la référence.
    /// * `example_lang` - Langue des valeurs d'exemple.
    fn new<E: serde::Serialize>(
        lang: String,
        entry_texts: CvEntryTexts,
        contract: &StoredContract<E>,
        example_lang: &str,
    ) -> Result<Self, AppError> {
        let example = serde_json::to_value(&contract.example)
            .map_err(|e| AppError::Validation(format!("Exemple de la référence non sérialisable : {e}")))?;
        let guidance = ProseGuidance::from_slots(&contract.guidance, &contract.slots, &example);
        // Le validateur rend chaque langue déclarée : la langue du document
        // est attendue, celle de l'exemple n'est qu'un filet.
        let reference_text = contract
            .example_text
            .get(&lang)
            .or_else(|| contract.example_text.get(example_lang))
            .cloned()
            .ok_or_else(|| AppError::Validation(format!("Rendu de la référence absent pour la langue « {lang} ».")))?;
        Ok(Self { lang, entry_texts, guidance, reference_text })
    }
}

/// Tout ce que la rédaction et l'assemblage partagent, préparé une fois.
///
/// # Attributes
///
/// * `profile` - Profil complet du candidat.
/// * `target` - Clé API et modèle Claude configuré.
/// * `cv_ref`, `lm_ref` - Références validées.
/// * `recipient` - Destinataire de la lettre.
/// * `job_title` - Intitulé exact du poste de la candidature.
/// * `offer_analysis` - Analyse de l'offre (JSON).
/// * `matching`, `selection` - Matching et entrées retenues.
/// * `cv_budget`, `lm_budget` - Budgets initiaux.
/// * `missing` - Catégories du CV retirées car vides dans le profil.
/// * `missing_contacts` - Coordonnées affichées par une référence mais vides dans le profil.
/// * `cv`, `lm` - Langue, textes du profil et consignes de chaque document.
/// * `period` - Format des périodes déclaré par la référence du CV.
/// * `list_marker` - Symbole des éléments de liste du CV (format de la session).
struct GenerationContext {
    profile: ProfileDto,
    target: ClaudeTarget,
    cv_ref: ReferenceTemplate<CvData>,
    lm_ref: ReferenceTemplate<LmData>,
    recipient: Recipient,
    job_title: String,
    location: SenderLocation,
    offer_analysis: String,
    matching: MatchingResult,
    selection: Selection,
    cv_budget: CvBudget,
    lm_budget: LmBudget,
    missing: Vec<ProfileCategory>,
    missing_contacts: Vec<ContactField>,
    cv: DocumentLanguage,
    lm: DocumentLanguage,
    period: PeriodSpec,
    list_marker: &'static str,
}

impl GenerationContext {
    /// Entrées de la rédaction du CV pour un budget donné.
    fn cv_input(&self, budget: CvBudget) -> CvWritingInput<'_> {
        CvWritingInput {
            offer_analysis: &self.offer_analysis,
            matching: &self.matching,
            selection: &self.selection,
            profile: &self.profile,
            entry_texts: &self.cv.entry_texts,
            example: &self.cv_ref.contract.example,
            reference_text: &self.cv.reference_text,
            guidance: &self.cv.guidance,
            budget,
            target_lang: &self.cv.lang,
            example_lang: &self.cv_ref.contract.example.lang,
        }
    }

    /// Entrées de la rédaction de la lettre pour un budget donné.
    fn lm_input(&self, budget: LmBudget) -> LmWritingInput<'_> {
        LmWritingInput {
            offer_analysis: &self.offer_analysis,
            matching: &self.matching,
            selection: &self.selection,
            profile: &self.profile,
            entry_texts: &self.lm.entry_texts,
            example: &self.lm_ref.contract.example,
            reference_text: &self.lm.reference_text,
            guidance: &self.lm.guidance,
            budget,
            company: &self.recipient.company,
            job_title: &self.job_title,
            target_lang: &self.lm.lang,
            example_lang: &self.lm_ref.contract.example.lang,
        }
    }
}

/// Étapes 1 et 2 : profil, références, destinataire, analyse de l'offre,
/// matching, sélection, langues, traductions et budgets.
async fn prepare(
    pool: &SqlitePool,
    request: &GenerationRequest<'_>,
    on_progress: &impl Fn(u8, &str),
) -> Result<GenerationContext, AppError> {
    // ------------------------------------------------------------------
    // 1. Profil, modèle, références validées, destinataire
    // ------------------------------------------------------------------
    let profile = db::profile::build_profile(pool).await?;
    // Avant tout appel LLM : sans prénom ni nom, aucun document valable ne peut être produit.
    validate_sender_name(&profile.user)?;
    // Tous les appels de la génération sont journalisés pour `llm_calls.json`.
    let target = ClaudeTargetSource { pool, api_key: request.api_key, llm: request.llm }
        .resolve()
        .await?
        .with_recorder(Arc::new(CallRecorder::default()));
    let cv_ref: ReferenceTemplate<CvData> =
        load_reference(pool, request.data_dir, request.session_id, "cv", "CV").await?;
    // Le validateur refuse un CV sans format de période : une absence ici
    // vient d'un contrat modifié à la main.
    let period = cv_ref
        .contract
        .period
        .clone()
        .ok_or_else(|| AppError::Validation("Le contrat du CV de référence n'a pas de format de période.".to_string()))?;
    let lm_ref: ReferenceTemplate<LmData> =
        load_reference(pool, request.data_dir, request.session_id, "lettre_motivation", "lettre de motivation").await?;
    let (recipient, job_title) = load_recipient(pool, request.candidature_id).await?;
    // Le lieu de l'expéditeur vient de la session, jamais de la référence.
    let session = db::sessions::fetch_view(pool, request.session_id).await?;
    let location = SenderLocation::new(session.ville.as_ref(), session.zip_code.as_ref());
    // Le symbole des listes du CV vient aussi de la session, lu une fois au
    // démarrage : un changement pendant la génération vaut pour la suivante.
    let list_marker = list_marker(&session.format_liste);

    // ------------------------------------------------------------------
    // 2. Analyse de l'offre, matching et sélection
    // ------------------------------------------------------------------
    on_progress(1, "Analyse de l'offre…");
    let offer_analysis = analyze_offer(request.description_offre, &target).await?;

    on_progress(2, "Matching profil / offre…");
    let project_count = expected_project_count(&profile, cv_ref.contract.example.projects.len());
    let matching = match_profile(&offer_analysis, &profile, project_count, &target).await?;
    let selection = select(&profile, &matching.project_ids);

    // ------------------------------------------------------------------
    // 3. Langue de chaque document, puis textes du profil dans cette
    //    langue (aucun appel quand les langues coïncident). Les textes de
    //    la référence sont déjà dans chaque langue, dans son template.
    // ------------------------------------------------------------------
    let cv_lang = target_language(&offer_analysis, &cv_ref.contract.example.lang);
    let lm_lang = target_language(&offer_analysis, &lm_ref.contract.example.lang);
    let source = source_texts(&profile, &selection);
    let profile_lang = profile.user.langue_profil.as_str();
    let (cv_entries, lm_entries) = tokio::try_join!(
        translate_entry_texts(&source, profile_lang, &cv_lang, &target),
        // Une seule traduction du profil quand les deux documents partagent
        // leur langue (le cas courant).
        async {
            if lm_lang == cv_lang {
                Ok(None)
            } else {
                translate_entry_texts(&source, profile_lang, &lm_lang, &target).await.map(Some)
            }
        },
    )?;
    let lm_entries = lm_entries.unwrap_or_else(|| cv_entries.clone());

    // ------------------------------------------------------------------
    // 4. Budgets, catégories et coordonnées, dérivés de l'exemple
    // ------------------------------------------------------------------
    let mut cv_budget = cv_budget(&cv_ref.contract.example, cv_ref.page_count);
    // Pas plus de groupes de compétences que de domaines dans le profil,
    // sinon le modèle doit en inventer un.
    let mut domains: Vec<&str> = profile.skills.iter().map(|s| s.domaine.trim()).collect();
    domains.sort_unstable();
    domains.dedup();
    cv_budget.skills_group_count = cv_budget.skills_group_count.min(domains.len());
    let lm_budget = lm_budget(&lm_ref.contract.example, lm_ref.page_count);
    // Calcul déterministe : l'unité du CV est retirée et signalée, jamais reprise de la référence.
    let missing = missing_categories(&cv_ref.contract.example, &profile);
    // Coordonnées vides : ligne retirée et signalée une seule fois pour les deux documents.
    let missing_contacts =
        missing_contacts(&[&cv_ref.contract.example.identity, &lm_ref.contract.example.identity], &profile.user);

    let cv = DocumentLanguage::new(cv_lang, cv_entries, &cv_ref.contract, &cv_ref.contract.example.lang)?;
    let lm = DocumentLanguage::new(lm_lang, lm_entries, &lm_ref.contract, &lm_ref.contract.example.lang)?;
    Ok(GenerationContext {
        profile,
        target,
        cv_ref,
        lm_ref,
        recipient,
        job_title,
        location,
        offer_analysis,
        matching,
        selection,
        cv_budget,
        lm_budget,
        missing,
        missing_contacts,
        cv,
        lm,
        period,
        list_marker,
    })
}

/// Assemble, compile et ajuste le CV au nombre de pages de la référence.
async fn produce_cv(
    ctx: &GenerationContext,
    writing: WritingOutcome<CvContent>,
    fonts_dir: &Path,
) -> Result<FittedDocument<CvContent>, AppError> {
    let period_format = PeriodFormat::new(&ctx.period, &ctx.cv.lang);
    let layout = CvLayout {
        location: &ctx.location,
        granularity: &ctx.period.granularity,
        period_format: &period_format,
        lang: &ctx.cv.lang,
        list_marker: ctx.list_marker,
    };
    let target = CompileTarget {
        label: "CV",
        template: &ctx.cv_ref.template,
        fonts_dir,
        max_pages: ctx.cv_ref.page_count,
    };
    fit_to_pages(
        &target,
        writing,
        |content| build_cv_data(&ctx.profile, &ctx.selection, &ctx.cv.entry_texts, content, &layout),
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
    let layout = LmLayout { location: &ctx.location, title, date: clock::today_local(), lang: &ctx.lm.lang };
    let target = CompileTarget {
        label: "lettre",
        template: &ctx.lm_ref.template,
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

/// Avertissement si la lettre finale recopie encore un passage du contexte
/// détaillé : la revue relance la rédaction, mais ne bloque jamais pour ce
/// motif. Contrôle fait sur la lettre retenue, après les relances de pages.
fn copied_context_warning(lm: &LmContent, selection: &Selection) -> Option<String> {
    let contexts: Vec<String> = letter_contexts(selection).into_iter().map(|c| c.text).collect();
    if verbatim_violations(&lm.paragraphs, &contexts).is_empty() {
        return None;
    }
    trace_log::log("[pipeline] lettre : passage du contexte détaillé recopié après le dernier essai");
    Some(
        "La lettre reprend mot pour mot un passage de votre contexte détaillé. Relisez-la avant de l'envoyer."
            .to_string(),
    )
}

/// Construit le résultat et sa trace.
fn into_result(
    ctx: GenerationContext,
    cv_fitted: FittedDocument<CvContent>,
    lm_fitted: FittedDocument<LmContent>,
) -> GeneratedDocuments {
    let (cv, cv_trace) = finish(cv_fitted, ctx.cv_ref.version_id);
    let (lm, lm_trace) = finish(lm_fitted, ctx.lm_ref.version_id);
    let offer_analysis = serde_json::from_str(&ctx.offer_analysis)
        .unwrap_or_else(|_| serde_json::Value::String(ctx.offer_analysis.clone()));
    let trace = GenerationTrace {
        offer_analysis,
        experience_ids: ctx.selection.experience_ids(),
        project_ids: ctx.selection.project_ids(),
        matching: ctx.matching,
        cv_budget: ctx.cv_budget,
        lm_budget: ctx.lm_budget,
        missing_categories: ctx.missing.iter().map(|c| c.french_name().to_string()).collect(),
        cv: cv_trace,
        lm: lm_trace,
        llm_calls: ctx.target.recorder.as_ref().map(|r| r.snapshot()).unwrap_or_default(),
    };
    let mut warnings: Vec<String> = ctx.missing.iter().map(|c| c.warning()).collect();
    warnings.extend(ctx.missing_contacts.iter().map(|c| c.warning()));
    warnings.extend([&trace.cv.warning, &trace.lm.warning].into_iter().flatten().cloned());
    warnings.extend(copied_context_warning(&trace.lm.content, &ctx.selection));
    GeneratedDocuments { cv, lm, warnings, trace }
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
