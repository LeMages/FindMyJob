//! Traduction fidèle des textes d'entrées du CV (lignes de description des
//! expériences, projets et formations, organisation des projets) quand la
//! langue du CV diffère de celle du profil. Un seul appel pour l'ensemble,
//! validé en Rust (`docgen::translation_check`) avec retry.
//!
//! Les noms officiels (titre de projet, stack, lien, poste, entreprise,
//! diplôme, établissement, lieu) ne sont jamais envoyés : ils restent ceux
//! du profil.

// Importation des crates externes
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::entry_texts::CvEntryTexts;
use crate::docgen::translation_check::{translation_problems, LENGTH_TOLERANCE_PERCENT, SHORT_TEXT_MARGIN_CHARS};
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::object;
use crate::llm::target::ClaudeTarget;
use crate::llm::validated_call::call_until_valid;
use crate::llm::writing_loop::WritingFeedback;
use crate::llm::writing_rules::with_feedback;
use crate::models::doc_lang::direction;

/// Schéma de sortie de la traduction : même forme que `CvEntryTexts`.
pub fn translation_schema() -> Value {
    let id = json!({ "type": "integer" });
    let lines = json!({ "type": "array", "items": { "type": "string" } });
    let entry = object(&[("id", id.clone()), ("lines", lines.clone())]);
    let project = object(&[("id", id.clone()), ("pour_qui", json!({ "type": "string" })), ("lines", lines)]);
    let language = object(&[("id", id), ("nom", json!({ "type": "string" })), ("level", json!({ "type": "string" }))]);
    object(&[
        ("experiences", json!({ "type": "array", "items": entry.clone() })),
        ("projects", json!({ "type": "array", "items": project })),
        ("formations", json!({ "type": "array", "items": entry })),
        ("langues", json!({ "type": "array", "items": language })),
        ("interets", json!({ "type": "array", "items": { "type": "string" } })),
    ])
}

/// Prompt système de la traduction.
fn system_prompt(from: &str, to: &str) -> String {
    format!(
        "Tu traduis les textes d'un CV {}, sans rien rédiger.\n\
         Traduis chaque texte fidèlement : n'ajoute, ne retire et ne reformule aucune information, garde les chiffres, les noms propres, les noms de technologies et les sigles tels quels.\n\
         Chaque ligne de `lines` est traduite en une ligne, dans le même ordre : même nombre de lignes que l'original, sans saut de ligne.\n\
         Une traduction n'est pas plus longue que l'original : au plus {LENGTH_TOLERANCE_PERCENT} % de caractères en plus (ou {SHORT_TEXT_MARGIN_CHARS} caractères pour un texte court). Plus court est accepté.\n\
         `pour_qui` est le nom de l'organisation d'un projet : traduis-le seulement s'il est un nom commun (par exemple une association décrite), garde un nom propre tel quel. Un texte vide reste vide.\n\
         `langues` : traduis le nom de chaque langue et son niveau (« Langue maternelle » devient « Native »), en gardant les certifications et scores tels quels (TOEIC 905). `interets` : un centre d'intérêt traduit par élément, dans le même ordre.\n\
         Garde les mêmes identifiants, une entrée par entrée de l'original.",
        direction(from, to)
    )
}

/// Retire les espaces de bord de chaque texte traduit.
fn trimmed(mut texts: CvEntryTexts) -> CvEntryTexts {
    let trim = |lines: &mut Vec<String>| lines.iter_mut().for_each(|l| *l = l.trim().to_string());
    texts.experiences.iter_mut().chain(texts.formations.iter_mut()).for_each(|e| trim(&mut e.lines));
    for project in texts.projects.iter_mut() {
        project.pour_qui = project.pour_qui.trim().to_string();
        trim(&mut project.lines);
    }
    for langue in texts.langues.iter_mut() {
        langue.nom = langue.nom.trim().to_string();
        langue.level = langue.level.trim().to_string();
    }
    trim(&mut texts.interets);
    texts
}

/// Appelle le LLM une fois.
async fn call_once(
    system_prompt: &str,
    source: &str,
    feedback: Option<WritingFeedback>,
    target: &ClaudeTarget,
) -> Result<CvEntryTexts, AppError> {
    let message = with_feedback(format!("TEXTES À TRADUIRE :\n{source}"), feedback.as_ref());
    let raw = call_claude("traduction:profil", system_prompt, &message, Some(translation_schema()), target).await?;
    let texts: CvEntryTexts = serde_json::from_str(raw.trim()).map_err(|e| {
        AppError::Llm(format!("La traduction du profil n'a pas retourné le JSON attendu : {e}\nRéponse brute : {raw}"))
    })?;
    Ok(trimmed(texts))
}

/// Traduit les textes d'entrées du CV dans la langue cible.
///
/// # Arguments
///
/// * `texts` - Textes dans la langue du profil (`docgen::entry_texts::source_texts`).
/// * `from` - Langue du profil.
/// * `to` - Langue du CV.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Les textes traduits ; les textes d'origine, sans appel, si les langues
/// coïncident ou s'il n'y a rien à traduire ; `AppError::Llm` si la
/// traduction reste invalide après 3 essais.
pub async fn translate_entry_texts(
    texts: &CvEntryTexts,
    from: &str,
    to: &str,
    target: &ClaudeTarget,
) -> Result<CvEntryTexts, AppError> {
    if from == to || texts.is_blank() {
        return Ok(texts.clone());
    }
    let source = serde_json::to_string_pretty(texts)
        .map_err(|e| AppError::Llm(format!("Textes du profil non sérialisables : {e}")))?;
    let prompt = system_prompt(from, to);
    call_until_valid(
        "La traduction du profil",
        |feedback| call_once(&prompt, &source, feedback, target),
        |translated| translation_problems(texts, translated),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_dans_les_deux_sens() {
        assert_eq!(direction("fr", "en"), "du français vers l'anglais");
        assert_eq!(direction("en", "fr"), "de l'anglais vers le français");
        assert!(system_prompt("fr", "en").contains("du français vers l'anglais"));
    }

    #[test]
    fn schema_sans_nom_officiel() {
        let schema = translation_schema();
        let project = &schema["properties"]["projects"]["items"]["properties"];
        assert!(project.get("pour_qui").is_some());
        for field in ["titre", "stack", "url"] {
            assert!(project.get(field).is_none(), "{field} n'est jamais traduit");
        }
        let experience = &schema["properties"]["experiences"]["items"]["properties"];
        for field in ["poste", "entreprise", "lieu"] {
            assert!(experience.get(field).is_none(), "{field} n'est jamais traduit");
        }
    }
}
