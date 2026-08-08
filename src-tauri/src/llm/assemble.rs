//! Assemblage du code Typst final via l'API Claude — étapes 4a/4b du
//! pipeline de génération de documents, et correction en cas d'échec de
//! compilation (`fix_typst`, utilisé en retry par `llm::pipeline`).
//!
//! Contrairement aux autres étapes du pipeline, la sortie attendue n'est pas
//! du JSON mais du code Typst brut : pas de structured output ici, et un
//! plafond de tokens élevé (un CV Typst complet peut dépasser 4096 tokens).

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::{call_claude_with_tokens, call_claude_with_tokens_meta};

/// Plafond de tokens de sortie pour l'assemblage Typst — un document complet
/// (CV ou lettre) peut largement dépasser le plafond par défaut des étapes JSON.
const ASSEMBLE_MAX_TOKENS: u32 = 8192;

/// Libellé lisible d'un type de document, utilisé dans les prompts.
fn doc_type_label(doc_type: &str) -> &str {
    match doc_type {
        "cv" => "CV / résumé",
        "lettre_motivation" => "lettre de motivation / cover letter",
        _ => doc_type,
    }
}

/// Valide que la réponse du LLM ressemble à du code Typst compilable et non
/// à du texte enrobé (markdown, explications).
///
/// # Arguments
///
/// * `code` - Réponse brute (déjà `trim()`ée) à valider.
///
/// # Returns
///
/// `Ok(())` si la réponse semble être du Typst brut, sinon une erreur
/// explicite incluant le début de la réponse pour diagnostic.
fn validate_typst_output(code: &str) -> Result<(), AppError> {
    if code.contains("```") {
        return Err(AppError::Llm(format!(
            "Le code Typst généré contient des balises markdown non attendues.\nDébut de la réponse: {}",
            code.chars().take(200).collect::<String>()
        )));
    }
    let starts_valid = code.starts_with('#') || code.starts_with("//") || code.starts_with("import");
    if !starts_valid {
        return Err(AppError::Llm(format!(
            "Le code Typst généré ne commence pas par un pattern valide (#, // ou import).\nDébut de la réponse: {}",
            code.chars().take(200).collect::<String>()
        )));
    }
    Ok(())
}

/// Assemble le code Typst final d'un document (CV ou lettre de motivation) à
/// partir de son contenu rédigé, de l'analyse de style du document de
/// référence de la session, et du document Typst original lui-même.
///
/// # Arguments
///
/// * `content_json` - JSON de contenu produit par `llm::content` (CV ou lettre).
/// * `style_analysis` - JSON d'analyse de style du document de référence (voir `llm::style`).
/// * `original_typst` - Code Typst du document de référence, utilisé comme référence de style et de syntaxe.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Code Typst brut, prêt à être compilé.
pub async fn assemble_typst(
    content_json: &str,
    style_analysis: &str,
    original_typst: &str,
    doc_type: &str,
) -> Result<String, AppError> {
    let label = doc_type_label(doc_type);

    let system_prompt = format!(
        r#"Tu es un expert Typst. Tu génères un document {label} en code Typst.

Tu reçois :
1. Le CONTENU à utiliser (JSON) — c'est ce qui doit apparaître dans le document
2. L'ANALYSE DE STYLE — les conventions de structure et de design à reproduire
3. Le DOCUMENT ORIGINAL en Typst — c'est ta référence de style et de syntaxe

Règles :
- Reproduis fidèlement la structure, le design et le style Typst du document original
- Utilise UNIQUEMENT le contenu fourni dans le JSON — n'invente rien
- Le code Typst doit compiler sans erreur
- Conserve les imports, les fonctions custom, et les packages du document original
- Conserve les polices, couleurs, et spacing du document original
- Retourne UNIQUEMENT le code Typst, sans backticks, sans commentaires avant/après"#
    );

    let user_message = format!(
        "CONTENU :\n{content_json}\n\nANALYSE DE STYLE :\n{style_analysis}\n\nDOCUMENT ORIGINAL :\n{original_typst}"
    );

    let is_lm = doc_type == "lettre_motivation";

    // [LM-DEBUG] Point 3 — juste avant l'appel d'assemblage, pour la LM uniquement.
    if is_lm {
        eprintln!(
            "[LM-DEBUG] assemble_typst(lettre_motivation): longueur du content_json injecté={} caractères",
            content_json.len()
        );
        eprintln!(
            "[LM-DEBUG] assemble_typst(lettre_motivation): 500 premiers caractères du content_json:\n{}",
            content_json.chars().take(500).collect::<String>()
        );
        eprintln!(
            "[LM-DEBUG] assemble_typst(lettre_motivation): document Typst de référence -> longueur={} caractères",
            original_typst.len()
        );
    }

    let (raw_response, stop_reason) =
        call_claude_with_tokens_meta(&system_prompt, &user_message, None, ASSEMBLE_MAX_TOKENS).await?;
    let code = raw_response.trim().to_string();

    // [LM-DEBUG] Point 4 — réponse de l'assemblage Typst, pour la LM uniquement.
    if is_lm {
        eprintln!(
            "[LM-DEBUG] assemble_typst(lettre_motivation): stop_reason={:?}, longueur du Typst retourné={} caractères",
            stop_reason,
            code.len()
        );
        eprintln!(
            "[LM-DEBUG] assemble_typst(lettre_motivation): 300 premiers caractères du Typst retourné:\n{}",
            code.chars().take(300).collect::<String>()
        );
    }

    validate_typst_output(&code)?;
    Ok(code)
}

/// Corrige du code Typst dont la compilation a échoué, à partir des erreurs
/// remontées par le compilateur et du document original (référence de
/// syntaxe). Utilisé par `llm::pipeline` en retry, une seule fois par document.
///
/// # Arguments
///
/// * `broken_code` - Code Typst dont la compilation a échoué.
/// * `compiler_errors` - Message d'erreur retourné par `documents::compile_to_pdf`.
/// * `original_typst` - Code Typst du document de référence, pour la syntaxe.
///
/// # Returns
///
/// Code Typst corrigé, prêt à être recompilé.
pub async fn fix_typst(
    broken_code: &str,
    compiler_errors: &str,
    original_typst: &str,
) -> Result<String, AppError> {
    let system_prompt = "Corrige ce code Typst. Voici les erreurs du compilateur. Voici le document \
        original pour référence de syntaxe. Retourne uniquement le code corrigé, sans backticks, \
        sans commentaires avant/après.";

    let user_message = format!(
        "CODE À CORRIGER :\n{broken_code}\n\nERREURS DU COMPILATEUR :\n{compiler_errors}\n\nDOCUMENT ORIGINAL (référence de syntaxe) :\n{original_typst}"
    );

    let raw_response = call_claude_with_tokens(system_prompt, &user_message, None, ASSEMBLE_MAX_TOKENS).await?;
    let code = raw_response.trim().to_string();
    validate_typst_output(&code)?;
    Ok(code)
}
