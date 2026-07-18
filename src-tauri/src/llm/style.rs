//! Analyse de style d'un document Typst (CV ou lettre de motivation) via
//! l'API Claude — étape 0 du pipeline IA. Le résultat (JSON) est destiné à
//! être mis en cache dans `documents.style_analysis`.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;

/// Schéma JSON attendu pour l'analyse de style, utilisé comme structured
/// output (`output_config.format`) — l'API garantit alors une réponse
/// strictement conforme, sans besoin de re-valider les champs manuellement.
fn style_analysis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "sections_order": {
                "type": "array",
                "items": { "type": "string" }
            },
            "formatting": {
                "type": "object",
                "properties": {
                    "bullet_style": { "type": "string", "enum": ["bullets", "sentences", "mixed"] },
                    "person": { "type": "string", "enum": ["first", "third", "impersonal"] },
                    "description_length": { "type": "string", "enum": ["short", "medium", "long"] },
                    "tense": { "type": "string", "enum": ["present", "past", "mixed"] }
                },
                "required": ["bullet_style", "person", "description_length", "tense"],
                "additionalProperties": false
            },
            "tone": { "type": "string", "enum": ["formal", "semi_formal", "casual"] },
            "design": {
                "type": "object",
                "properties": {
                    "layout": { "type": "string", "enum": ["single_column", "two_column", "sidebar"] },
                    "has_icons": { "type": "boolean" },
                    "has_separators": { "type": "boolean" },
                    "has_photo": { "type": "boolean" },
                    "color_usage": { "type": "string", "enum": ["monochrome", "accent_color", "colorful"] }
                },
                "required": ["layout", "has_icons", "has_separators", "has_photo", "color_usage"],
                "additionalProperties": false
            },
            "typst_patterns": {
                "type": "object",
                "properties": {
                    "uses_functions": { "type": "boolean" },
                    "custom_commands": { "type": "array", "items": { "type": "string" } },
                    "imported_packages": { "type": "array", "items": { "type": "string" } },
                    "font_families": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["uses_functions", "custom_commands", "imported_packages", "font_families"],
                "additionalProperties": false
            }
        },
        "required": ["sections_order", "formatting", "tone", "design", "typst_patterns"],
        "additionalProperties": false
    })
}

/// Libellé lisible d'un type de document, utilisé dans les prompts.
fn doc_type_label(doc_type: &str) -> &str {
    match doc_type {
        "cv" => "CV / résumé",
        "lettre_motivation" => "lettre de motivation / cover letter",
        _ => doc_type,
    }
}

/// Analyse la structure et le style d'un document Typst via l'API Claude.
///
/// # Arguments
///
/// * `typ_content` - Contenu source Typst du document à analyser.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Le JSON d'analyse (sérialisé de façon canonique), prêt à être stocké
/// dans `documents.style_analysis`. Toute erreur d'appel, de refus ou de
/// format inattendu remonte telle quelle : l'appelant doit alors abandonner
/// la sauvegarde du document.
pub async fn analyze_style(typ_content: &str, doc_type: &str) -> Result<String, AppError> {
    let label = doc_type_label(doc_type);

    let system_prompt = format!(
        r#"Tu es un expert en analyse de documents professionnels. Tu analyses la structure et le style d'un document {label} au format Typst, et tu retournes une analyse structurée.

Description des champs à renseigner :
- sections_order : l'ordre des sections tel qu'elles apparaissent dans le document (ex: "header", "summary", "experience", "education", "skills", "languages", "projects")
- formatting.bullet_style : comment les descriptions sont formatées
- formatting.person : première personne ("j'ai développé"), troisième ("a développé"), ou impersonnel ("développement de")
- formatting.description_length : short = 1-2 lignes, medium = 3-4 lignes, long = 5+ lignes par item
- formatting.tense : temps verbal dominant dans les descriptions
- tone : registre général du document
- design : caractéristiques visuelles du layout Typst (colonnes, icônes, séparateurs, photo, usage de couleur)
- typst_patterns : patterns techniques Typst utilisés (fonctions custom, packages importés, polices)"#
    );

    let user_message =
        format!("Analyse ce document {label} au format Typst :\n\n```typst\n{typ_content}\n```");

    let raw_response =
        call_claude(&system_prompt, &user_message, Some(style_analysis_schema())).await?;

    // Le structured output garantit normalement un JSON conforme au schéma ;
    // ce parsing reste un filet de sécurité plutôt qu'une validation de forme.
    let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "L'analyse de style n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })?;

    // Re-sérialisation canonique (élimine le whitespace superflu) avant stockage DB.
    serde_json::to_string(&parsed)
        .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")))
}
