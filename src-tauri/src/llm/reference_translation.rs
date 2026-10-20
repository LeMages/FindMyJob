//! Traduction fidèle des textes d'une référence (textes en dur, zones
//! `[FIXE]`, consignes) dans la langue d'un document, en un appel validé
//! (`docgen::reference_texts_translation`), avec relances.

// Importation des crates externes
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::reference_texts_translation::{
    apply_translation, translatable_items, translation_problems, TranslationItem,
};
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::object;
use crate::llm::target::ClaudeTarget;
use crate::llm::validated_call::call_until_valid;
use crate::llm::writing_loop::WritingFeedback;
use crate::llm::writing_rules::with_feedback;
use crate::models::doc_lang::direction;
use crate::models::ReferenceTexts;

/// Version du prompt de traduction des textes de la référence : un cache
/// écrit avec une autre version est retraduit. À incrémenter à chaque
/// modification du prompt.
pub const REFERENCE_TEXTS_PROMPT_VERSION: u32 = 1;

/// Sortie de la traduction.
///
/// # Attributes
///
/// * `items` - Éléments traduits, mêmes identifiants que l'original.
#[derive(Serialize, Deserialize)]
struct TranslatedItems {
    items: Vec<TranslationItem>,
}

/// Schéma de sortie de la traduction des textes de la référence.
pub fn reference_translation_schema() -> Value {
    let item = object(&[("id", json!({ "type": "string" })), ("text", json!({ "type": "string" }))]);
    object(&[("items", json!({ "type": "array", "items": item }))])
}

/// Prompt système de la traduction.
fn system_prompt(from: &str, to: &str) -> String {
    format!(
        "Tu traduis {} les textes fixes d'un document de candidature (CV ou lettre de motivation) et les consignes \
         qui servent à le rédiger.\n\
         Traduis chaque texte fidèlement : même sens, même registre, rien d'ajouté ni de retiré. Garde tels quels \
         les noms propres, les noms officiels (intitulé de poste, entreprise, diplôme, école, titre de projet), les \
         chiffres, les sigles et les noms de technologies. Pour une formule usuelle (formule de politesse, mention \
         « Références disponibles sur demande », « P.J. »), utilise son équivalent usuel dans la langue cible.\n\
         Les éléments dont l'identifiant commence par « zone_fixe: » sont des passages recopiés tels quels dans le \
         document : traduis-les mot à mot autant que la langue le permet.\n\
         Les consignes (« consignes », « zone_consigne: … ») sont des instructions de rédaction : traduis-les \
         aussi, en gardant les balises [FIXE], [VARIABLE] et [ADAPTABLE] telles quelles.\n\
         Réponds avec exactement les mêmes identifiants, chacun une fois.",
        direction(from, to)
    )
}

/// Appelle le LLM une fois.
async fn call_once(
    system: &str,
    source: &str,
    feedback: Option<WritingFeedback>,
    target: &ClaudeTarget,
) -> Result<TranslatedItems, AppError> {
    let message = with_feedback(format!("TEXTES À TRADUIRE :\n{source}"), feedback.as_ref());
    let raw = call_claude("traduction:référence", system, &message, Some(reference_translation_schema()), target).await?;
    serde_json::from_str(raw.trim()).map_err(|e| {
        AppError::Llm(format!("La traduction des textes de la référence n'a pas retourné le JSON attendu : {e}"))
    })
}

/// Traduit les textes d'une référence dans une langue.
///
/// # Arguments
///
/// * `texts` - Textes de la référence, dans sa langue.
/// * `to` - Langue cible.
/// * `target` - Clé API et modèle Claude.
///
/// # Returns
///
/// Les textes traduits ; les textes d'origine, sans appel, si la langue est
/// déjà la bonne ou s'il n'y a rien à traduire ; `AppError::Llm` si la
/// traduction reste invalide après 3 essais.
pub async fn translate_reference_texts(
    texts: &ReferenceTexts,
    to: &str,
    target: &ClaudeTarget,
) -> Result<ReferenceTexts, AppError> {
    let items = translatable_items(texts);
    if texts.lang == to || items.is_empty() {
        return Ok(ReferenceTexts { lang: to.to_string(), ..texts.clone() });
    }
    let source = serde_json::to_string_pretty(&TranslatedItems { items: items.clone() })
        .map_err(|e| AppError::Llm(format!("Textes de la référence non sérialisables : {e}")))?;
    let system = system_prompt(&texts.lang, to);
    let translated = call_until_valid(
        "La traduction des textes de la référence",
        |feedback| call_once(&system, &source, feedback, target),
        |output: &TranslatedItems| translation_problems(&items, &output.items),
    )
    .await?;
    Ok(apply_translation(texts, &translated.items, to))
}
