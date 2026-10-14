//! Faux exécutant des appels à Claude : répond par des fixtures figées
//! (`tests/fixtures/llm/`), sans réseau ni clé réelle. Chaque appel est
//! rattaché à une étape d'après le schéma de sortie demandé, compté et
//! enregistré pour les vérifications.

// Importation des modules standard
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use findmyjob_lib::error::AppError;
use findmyjob_lib::llm::{LlmCaller, LlmFuture, LlmRequest, LlmResponse};

/// Étape du pipeline à laquelle un appel appartient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    OfferAnalysis,
    Matching,
    CvLabels,
    LmLabels,
    CvContent,
    LmContent,
    ScrapedOffer,
    TemplatizeCv,
    TemplatizeLm,
}

/// Réponse renvoyée pour une étape.
#[derive(Debug, Clone)]
pub enum Reply {
    /// Contenu d'un fichier de `tests/fixtures/llm/`.
    Fixture(&'static str),
    /// Erreur de l'API simulée (`AppError::Llm`).
    Error(&'static str),
}

/// Appel reçu, pour les vérifications.
///
/// # Attributes
///
/// * `step` - Étape reconnue.
/// * `user_message` - Message utilisateur envoyé.
#[derive(Debug, Clone)]
pub struct RecordedCall {
    pub step: Step,
    pub user_message: String,
}

/// Exécutant factice.
///
/// # Attributes
///
/// * `overrides` - Réponses remplaçant celles par défaut, par étape.
/// * `calls` - Appels reçus, dans l'ordre.
#[derive(Debug, Default)]
pub struct FakeLlm {
    overrides: Mutex<HashMap<Step, Reply>>,
    calls: Mutex<Vec<RecordedCall>>,
}

impl FakeLlm {
    /// Remplace la réponse d'une étape.
    pub fn respond_with(&self, step: Step, reply: Reply) {
        self.overrides.lock().unwrap().insert(step, reply);
    }

    /// Nombre d'appels reçus pour une étape.
    pub fn calls_for(&self, step: Step) -> usize {
        self.calls.lock().unwrap().iter().filter(|c| c.step == step).count()
    }

    /// Nombre total d'appels reçus.
    pub fn total_calls(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Messages utilisateur reçus pour une étape.
    pub fn messages_for(&self, step: Step) -> Vec<String> {
        self.calls.lock().unwrap().iter().filter(|c| c.step == step).map(|c| c.user_message.clone()).collect()
    }

    /// Réponse à renvoyer pour une étape : surcharge, sinon fixture par défaut.
    fn reply_for(&self, step: Step) -> Result<String, AppError> {
        let reply = self.overrides.lock().unwrap().get(&step).cloned();
        match reply {
            Some(Reply::Fixture(name)) => Ok(read_fixture(name)),
            Some(Reply::Error(message)) => Err(AppError::Llm(message.to_string())),
            None => Ok(default_reply(step)),
        }
    }
}

impl LlmCaller for FakeLlm {
    fn call<'a>(&'a self, request: LlmRequest<'a>) -> LlmFuture<'a> {
        let step = step_of(request.output_schema.as_ref());
        self.calls.lock().unwrap().push(RecordedCall { step, user_message: request.user_message.to_string() });
        let reply = self.reply_for(step);
        Box::pin(async move { reply.map(|text| LlmResponse { text, stop_reason: Some("end_turn".to_string()) }) })
    }
}

/// Reconnaît l'étape d'après une propriété propre à son schéma de sortie.
fn step_of(schema: Option<&Value>) -> Step {
    let properties = schema
        .and_then(|s| s.get("properties"))
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("FakeLlm : appel sans schéma de sortie, étape inconnue"));
    let has = |key: &str| properties.contains_key(key);
    if has("template_typ") {
        let is_cv = properties["example_data"]["properties"].get("period_style").is_some();
        return if is_cv { Step::TemplatizeCv } else { Step::TemplatizeLm };
    }
    let candidates = [
        ("job_title", Step::OfferAnalysis),
        ("relevant_experiences", Step::Matching),
        ("months_short", Step::CvLabels),
        ("subject_prefix", Step::LmLabels),
        ("skills_groups", Step::CvContent),
        ("paragraphs", Step::LmContent),
        ("poste", Step::ScrapedOffer),
    ];
    candidates
        .into_iter()
        .find(|(key, _)| has(key))
        .map(|(_, step)| step)
        .unwrap_or_else(|| panic!("FakeLlm : schéma non reconnu, propriétés {:?}", properties.keys().collect::<Vec<_>>()))
}

/// Réponse par défaut d'une étape.
fn default_reply(step: Step) -> String {
    match step {
        Step::OfferAnalysis => read_fixture("offer_analysis.json"),
        Step::Matching => read_fixture("matching.json"),
        Step::CvLabels => read_fixture("cv_labels.json"),
        Step::LmLabels => read_fixture("lm_labels.json"),
        Step::CvContent => read_fixture("cv_content.json"),
        Step::LmContent => read_fixture("lm_content.json"),
        Step::ScrapedOffer => read_fixture("scraped_offer.json"),
        Step::TemplatizeCv => templatize_reply("cv_template.typ", "cv_example_data.json"),
        Step::TemplatizeLm => templatize_reply("lm_template.typ", "lm_example_data.json"),
    }
}

/// Réponse de templatisation : template et données d'exemple de test, les
/// consignes rendues à part en chaîne JSON comme le fait le vrai modèle.
fn templatize_reply(template: &str, example_data: &str) -> String {
    let mut example: Value = serde_json::from_str(&read_reference(example_data)).expect("données d'exemple JSON");
    let guidance = example.as_object_mut().and_then(|map| map.remove("guidance")).unwrap_or_default();
    serde_json::json!({
        "template_typ": read_reference(template),
        "example_data": example,
        "guidance": guidance.to_string(),
    })
    .to_string()
}

/// Dossier des fixtures de test.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

/// Lit une réponse figée de `tests/fixtures/llm/`.
fn read_fixture(name: &str) -> String {
    let path = fixtures_dir().join("llm").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {} illisible : {e}", path.display()))
}

/// Lit un fichier de `tests/fixtures/references/`.
pub fn read_reference(name: &str) -> String {
    let path = fixtures_dir().join("references").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {} illisible : {e}", path.display()))
}
