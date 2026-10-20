//! Étape B de la templatisation : le template Typst, écrit en texte libre
//! (bloc ```typst```, sans grammaire imposée), en connaissant les données
//! d'exemple et les textes en dur de la référence.
//!
//! Après un essai refusé, le modèle peut renvoyer le template complet ou
//! des remplacements ciblés (`<<<<<<< AVANT` / `=======` / `>>>>>>> APRÈS`)
//! appliqués par l'appelant sur le template précédent.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude_with_tokens;
use crate::llm::target::ClaudeTarget;
use crate::llm::templatize_rules::{doc_type_label, CV_TEMPLATE_FIELDS, LM_TEMPLATE_FIELDS, TEMPLATE_RULES};

/// Plafond de tokens de sortie : un template complet de CV dépasse le
/// plafond par défaut d'une réponse JSON.
const TEMPLATE_MAX_TOKENS: u32 = 16384;
/// Ouverture d'un remplacement ciblé.
const EDIT_BEFORE: &str = "<<<<<<< AVANT";
/// Séparation entre l'ancien et le nouveau texte d'un remplacement.
const EDIT_SEPARATOR: &str = "=======";
/// Fermeture d'un remplacement ciblé.
const EDIT_AFTER: &str = ">>>>>>> APRÈS";

/// Ce qu'il faut pour écrire le template.
///
/// # Attributes
///
/// * `reference_typ` - Source Typst de la référence.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `example_data_json` - Données d'exemple validées (étape A), JSON indenté.
/// * `texts` - Textes en dur de la référence, par identifiant.
pub struct TemplateRequest<'a> {
    pub reference_typ: &'a str,
    pub doc_type: &'a str,
    pub example_data_json: &'a str,
    pub texts: &'a BTreeMap<String, String>,
}

/// Retour sur le meilleur template refusé jusqu'ici.
///
/// # Attributes
///
/// * `previous_template` - Template à corriger.
/// * `problems` - Problèmes relevés.
pub struct TemplateFeedback {
    pub previous_template: String,
    pub problems: Vec<String>,
}

/// Remplacement ciblé dans le template précédent.
///
/// # Attributes
///
/// * `before` - Texte exact à remplacer.
/// * `after` - Texte de remplacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateEdit {
    pub before: String,
    pub after: String,
}

/// Réponse de l'étape B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateReply {
    /// Template complet.
    Full(String),
    /// Remplacements à appliquer au template précédent.
    Edits(Vec<TemplateEdit>),
}

/// Écrit (ou corrige) le template d'une référence.
///
/// # Arguments
///
/// * `request` - Référence, données d'exemple et textes en dur.
/// * `feedback` - Template à corriger et problèmes, `None` au premier essai.
/// * `target` - Clé API et modèle Claude.
///
/// # Returns
///
/// `TemplateReply`, ou `AppError::Llm` si la réponse ne contient ni bloc
/// ```typst``` ni remplacement.
pub async fn write_template(
    request: &TemplateRequest<'_>,
    feedback: Option<&TemplateFeedback>,
    target: &ClaudeTarget,
) -> Result<TemplateReply, AppError> {
    let fields = if request.doc_type == "cv" { CV_TEMPLATE_FIELDS } else { LM_TEMPLATE_FIELDS };
    let label = doc_type_label(request.doc_type);
    let system = format!(
        "Tu écris le template Typst d'un document ({label}) : le même document, dont tout le texte visible est \
         lu depuis un fichier JSON (`data`), et tout le reste recopié tel quel.\n\n{TEMPLATE_RULES}\n\n{fields}\n\n\
         Réponds uniquement par le template complet, dans un seul bloc ```typst … ```."
    );
    let user = user_message(request, feedback);
    let raw =
        call_claude_with_tokens("templatisation:template", &system, &user, None, TEMPLATE_MAX_TOKENS, target).await?;
    parse_template_reply(&raw).map_err(AppError::Llm)
}

/// Message utilisateur : original, données, textes, puis correction demandée.
fn user_message(request: &TemplateRequest<'_>, feedback: Option<&TemplateFeedback>) -> String {
    let texts: Vec<String> = request.texts.iter().map(|(id, text)| format!("- data.texts.{id} : « {text} »")).collect();
    let texts = if texts.is_empty() { "(aucun)".to_string() } else { texts.join("\n") };
    let mut message = format!(
        "DOCUMENT ORIGINAL :\n```typst\n{}\n```\n\nDONNÉES (data, telles que lues depuis data.json) :\n```json\n{}\n```\n\n\
         TEXTES DE LA RÉFÉRENCE (à lire dans data.texts, jamais en dur) :\n{texts}",
        request.reference_typ, request.example_data_json
    );
    if let Some(feedback) = feedback {
        let problems: Vec<String> = feedback.problems.iter().map(|p| format!("- {p}")).collect();
        message.push_str(&format!(
            "\n\nLe template ci-dessous a été refusé. Corrige seulement ce qui pose problème.\n\nPROBLÈMES :\n{}\n\n\
             TEMPLATE À CORRIGER (lignes numérotées pour te repérer ; les numéros ne font pas partie du template) :\n\
             ```\n{}\n```\n\n\
             Réponds soit par le template complet corrigé dans un bloc ```typst … ```, soit, si les corrections sont \
             localisées, uniquement par des remplacements de cette forme, sans numéros de ligne :\n\
             {EDIT_BEFORE}\n(lignes exactes du template à remplacer)\n{EDIT_SEPARATOR}\n(nouvelles lignes)\n{EDIT_AFTER}\n\
             Le texte entre {EDIT_BEFORE} et {EDIT_SEPARATOR} doit apparaître une seule fois, à l'identique, dans le \
             template à corriger.",
            problems.join("\n"),
            numbered(&feedback.previous_template)
        ));
    }
    message
}

/// Template avec ses lignes numérotées.
fn numbered(template: &str) -> String {
    template.lines().enumerate().map(|(i, line)| format!("{:>4} | {line}", i + 1)).collect::<Vec<_>>().join("\n")
}

/// Lit la réponse de l'étape B : bloc ```typst``` ou remplacements.
///
/// # Arguments
///
/// * `raw` - Texte de la réponse.
///
/// # Returns
///
/// `TemplateReply`, ou un message si la réponse n'a ni l'un ni l'autre.
pub fn parse_template_reply(raw: &str) -> Result<TemplateReply, String> {
    let edits = parse_edits(raw);
    if !edits.is_empty() && !raw.contains("```typst") {
        return Ok(TemplateReply::Edits(edits));
    }
    extract_typst_block(raw).map(TemplateReply::Full).ok_or_else(|| {
        let start: String = raw.chars().take(200).collect();
        format!("La réponse ne contient ni bloc ```typst``` ni remplacement. Début de la réponse : {start}")
    })
}

/// Contenu du bloc ```typst``` (ou du premier bloc ``` à défaut). Un bloc
/// non refermé (réponse coupée) est refusé.
fn extract_typst_block(raw: &str) -> Option<String> {
    let start = raw.find("```typst").map(|i| i + "```typst".len()).or_else(|| raw.find("```").map(|i| i + 3))?;
    let body = &raw[start..];
    let body = body.strip_prefix('\n').or_else(|| body.strip_prefix("\r\n")).unwrap_or(body);
    let end = body.rfind("```")?;
    let block = body[..end].trim_end();
    (!block.trim().is_empty()).then(|| format!("{block}\n"))
}

/// Remplacements ciblés de la réponse, dans l'ordre.
fn parse_edits(raw: &str) -> Vec<TemplateEdit> {
    let mut edits = Vec::new();
    let mut rest = raw;
    while let Some(open) = rest.find(EDIT_BEFORE) {
        let after_open = &rest[open + EDIT_BEFORE.len()..];
        let Some(separator) = after_open.find(EDIT_SEPARATOR) else {
            break;
        };
        let Some(close) = after_open[separator..].find(EDIT_AFTER).map(|i| i + separator) else {
            break;
        };
        let before = trim_block(&after_open[..separator]);
        let after = trim_block(&after_open[separator + EDIT_SEPARATOR.len()..close]);
        edits.push(TemplateEdit { before, after });
        rest = &after_open[close + EDIT_AFTER.len()..];
    }
    edits
}

/// Retire le saut de ligne qui suit un marqueur et celui qui précède le suivant.
fn trim_block(block: &str) -> String {
    let block = block.strip_prefix("\r\n").or_else(|| block.strip_prefix('\n')).unwrap_or(block);
    let block = block.strip_suffix("\r\n").or_else(|| block.strip_suffix('\n')).unwrap_or(block);
    block.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloc_typst_extrait() {
        let reply = parse_template_reply("Voici :\n```typst\n#let data = json(\"data.json\")\nTexte\n```\nFin.");
        assert_eq!(reply, Ok(TemplateReply::Full("#let data = json(\"data.json\")\nTexte\n".to_string())));
    }

    #[test]
    fn remplacements_extraits() {
        let raw = "<<<<<<< AVANT\nligne 1\nligne 2\n=======\nnouvelle\n>>>>>>> APRÈS\n\n<<<<<<< AVANT\nx\n=======\ny\n>>>>>>> APRÈS";
        let expected = vec![
            TemplateEdit { before: "ligne 1\nligne 2".into(), after: "nouvelle".into() },
            TemplateEdit { before: "x".into(), after: "y".into() },
        ];
        assert_eq!(parse_template_reply(raw), Ok(TemplateReply::Edits(expected)));
    }

    #[test]
    fn reponse_sans_template_refusee() {
        assert!(parse_template_reply("Je ne peux pas.").is_err());
        assert!(parse_template_reply("```typst\ncoupé").is_err(), "bloc non refermé");
    }
}
