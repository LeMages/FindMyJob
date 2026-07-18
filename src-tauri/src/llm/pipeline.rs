//! Orchestration du pipeline de génération de documents (CV + lettre de
//! motivation) pour une candidature : enchaîne les 6 appels LLM du module
//! `llm`, compile le Typst produit en PDF, et retry une fois la compilation
//! en cas d'échec. Tout est synchrone dans l'appelant (pas de queue).

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::documents::compile_to_pdf;
use crate::error::AppError;
use crate::llm::{assemble, content, matching, offer};
use crate::models::Document;

/// Documents générés et compilés pour une candidature, prêts à être
/// sauvegardés sur disque par l'appelant.
pub struct GeneratedDocuments {
    pub cv_typst: String,
    pub cv_pdf: Vec<u8>,
    pub lm_typst: String,
    pub lm_pdf: Vec<u8>,
}

/// Récupère le document de référence d'un type donné pour la session,
/// et vérifie qu'il porte bien une analyse de style déjà calculée
/// (calculée à la sauvegarde de la référence, voir `documents::save_reference`).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `label` - Libellé lisible du type de document, pour le message d'erreur.
///
/// # Returns
///
/// Le `Document` de référence, garanti avec `style_analysis: Some(_)`.
async fn require_reference(
    pool: &SqlitePool,
    session_id: i64,
    doc_type: &str,
    label: &str,
) -> Result<Document, AppError> {
    db::documents::get_by_session_and_type(pool, session_id, doc_type)
        .await?
        .filter(|d| d.style_analysis.is_some())
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Veuillez d'abord importer votre {label} de référence dans les paramètres de session."
            ))
        })
}

/// Compile un document Typst en PDF, et retente une fois après correction
/// automatique (`assemble::fix_typst`) si la première compilation échoue.
///
/// # Arguments
///
/// * `code` - Code Typst à compiler.
/// * `original_typst` - Document Typst de référence, transmis à `fix_typst` comme référence de syntaxe.
/// * `root` - Dossier racine pour la résolution des imports/images relatifs du template.
///
/// # Returns
///
/// Le code Typst final (corrigé ou non) et les octets du PDF compilé.
async fn compile_with_retry(
    code: String,
    original_typst: &str,
    root: PathBuf,
) -> Result<(String, Vec<u8>), AppError> {
    let first_attempt = {
        let code = code.clone();
        let root = root.clone();
        tauri::async_runtime::spawn_blocking(move || compile_to_pdf(code, root))
            .await
            .map_err(|e| AppError::Llm(format!("Erreur interne de compilation Typst : {e}")))?
    };

    match first_attempt {
        Ok(pdf) => Ok((code, pdf)),
        Err(compiler_errors) => {
            let fixed_code = assemble::fix_typst(&code, &compiler_errors, original_typst).await?;

            let retry_result = {
                let code = fixed_code.clone();
                tauri::async_runtime::spawn_blocking(move || compile_to_pdf(code, root))
                    .await
                    .map_err(|e| AppError::Llm(format!("Erreur interne de compilation Typst : {e}")))?
            };

            retry_result.map(|pdf| (fixed_code.clone(), pdf)).map_err(|retry_errors| {
                AppError::Llm(format!(
                    "La compilation Typst a échoué après une tentative de correction.\n\
                     Erreurs du compilateur : {retry_errors}\n\nCode Typst généré :\n{fixed_code}"
                ))
            })
        }
    }
}

/// Génère le CV et la lettre de motivation adaptés à une offre, pour la
/// candidature en cours de création.
///
/// Enchaîne : analyse de l'offre → matching du profil → rédaction du contenu
/// (CV et lettre en parallèle) → assemblage Typst (CV et lettre en
/// parallèle) → compilation PDF (avec un retry de correction par document).
/// Aucune étape n'écrit sur le disque ni en base : c'est à l'appelant de
/// persister le résultat une fois la candidature créée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session (fournit les documents de référence).
/// * `description_offre` - Texte brut de l'offre d'emploi.
///
/// # Returns
///
/// `GeneratedDocuments` avec le Typst et le PDF de chaque document.
pub async fn generate_documents(
    pool: &SqlitePool,
    session_id: i64,
    description_offre: &str,
) -> Result<GeneratedDocuments, AppError> {
    // ------------------------------------------------------------------
    // 1. Profil candidat et documents de référence de la session
    // ------------------------------------------------------------------
    let profile = db::profile::build_profile(pool).await?;

    let cv_doc = require_reference(pool, session_id, "cv", "CV").await?;
    let lm_doc = require_reference(pool, session_id, "lettre_motivation", "lettre de motivation").await?;

    let cv_original = std::fs::read_to_string(&cv_doc.file_path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le CV de référence : {e}")))?;
    let lm_original = std::fs::read_to_string(&lm_doc.file_path).map_err(|e| {
        AppError::Validation(format!("Impossible de lire la lettre de référence : {e}"))
    })?;

    // Garanti Some par `require_reference` (filtre sur style_analysis.is_some()).
    let cv_style = cv_doc.style_analysis.as_deref().expect("style_analysis filtré en amont");
    let lm_style = lm_doc.style_analysis.as_deref().expect("style_analysis filtré en amont");

    let cv_root = doc_dir(&cv_doc.file_path)?;
    let lm_root = doc_dir(&lm_doc.file_path)?;

    // ------------------------------------------------------------------
    // 2. Étapes 1 → 2 : analyse de l'offre, puis matching du profil
    //    (séquentielles, la seconde dépend de la première)
    // ------------------------------------------------------------------
    let offer_analysis = offer::analyze_offer(description_offre).await?;
    let matching_strategy = matching::match_profile(&offer_analysis, &profile).await?;

    // ------------------------------------------------------------------
    // 3. Étapes 3a/3b : rédaction du contenu CV et lettre, en parallèle
    // ------------------------------------------------------------------
    let (cv_content, lm_content) = tokio::try_join!(
        content::generate_cv_content(&matching_strategy, &profile, &offer_analysis),
        content::generate_lm_content(&matching_strategy, &profile, &offer_analysis),
    )?;

    // ------------------------------------------------------------------
    // 4. Étapes 4a/4b : assemblage Typst CV et lettre, en parallèle
    // ------------------------------------------------------------------
    let (cv_typst, lm_typst) = tokio::try_join!(
        assemble::assemble_typst(&cv_content, cv_style, &cv_original, "cv"),
        assemble::assemble_typst(&lm_content, lm_style, &lm_original, "lettre_motivation"),
    )?;

    // ------------------------------------------------------------------
    // 5. Compilation PDF, avec un retry de correction par document
    // ------------------------------------------------------------------
    let (cv_typst, cv_pdf) = compile_with_retry(cv_typst, &cv_original, cv_root).await?;
    let (lm_typst, lm_pdf) = compile_with_retry(lm_typst, &lm_original, lm_root).await?;

    Ok(GeneratedDocuments { cv_typst, cv_pdf, lm_typst, lm_pdf })
}

/// Dérive le dossier racine de compilation à partir du chemin du `.typ` de
/// référence (`.../documents/sessions/<session_id>/<doc_type>/reference.typ`
/// → `.../documents/sessions/<session_id>/<doc_type>/`).
///
/// # Arguments
///
/// * `reference_typ_path` - Chemin absolu vers le fichier `.typ` de référence.
///
/// # Returns
///
/// Le dossier parent, utilisé pour résoudre les imports/images relatifs du template.
fn doc_dir(reference_typ_path: &str) -> Result<PathBuf, AppError> {
    Path::new(reference_typ_path)
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| AppError::Validation("Chemin de document de référence invalide.".to_string()))
}
