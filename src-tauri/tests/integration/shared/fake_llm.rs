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
    ProfileTranslation,
    ScrapedOffer,
    ReferenceTranslation,
    ExampleCv,
    ExampleLm,
    TemplateCv,
    TemplateLm,
}

/// Réponse renvoyée pour une étape.
#[derive(Debug, Clone)]
pub enum Reply {
    /// Contenu d'un fichier de `tests/fixtures/llm/`.
    Fixture(&'static str),
    /// Fichiers de `tests/fixtures/llm/` renvoyés dans l'ordre des appels
    /// de l'étape ; le dernier est répété au-delà.
    Sequence(&'static [&'static str]),
    /// Erreur de l'API simulée (`AppError::Llm`).
    Error(&'static str),
    /// Fichiers de `tests/fixtures/` (chemins relatifs) renvoyés tels quels
    /// dans l'ordre des appels de l'étape, le dernier répété au-delà ; un
    /// fichier `.typ` est entouré d'un bloc ```typst```, comme une réponse
    /// de l'étape B de la templatisation.
    Files(&'static [&'static str]),
}

/// Appel reçu, pour les vérifications.
///
/// # Attributes
///
/// * `step` - Étape reconnue.
/// * `user_message` - Message utilisateur envoyé.
/// * `model` - Modèle demandé.
#[derive(Debug, Clone)]
pub struct RecordedCall {
    pub step: Step,
    pub user_message: String,
    pub model: String,
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

    /// Modèles demandés pour une étape, dans l'ordre des appels.
    pub fn models_for(&self, step: Step) -> Vec<String> {
        self.calls.lock().unwrap().iter().filter(|c| c.step == step).map(|c| c.model.clone()).collect()
    }

    /// Messages utilisateur reçus pour une étape.
    pub fn messages_for(&self, step: Step) -> Vec<String> {
        self.calls.lock().unwrap().iter().filter(|c| c.step == step).map(|c| c.user_message.clone()).collect()
    }

    /// Réponse à renvoyer pour une étape : surcharge, sinon fixture par défaut.
    /// `call_index` est le rang (0-based) de l'appel parmi ceux de l'étape.
    fn reply_for(&self, step: Step, call_index: usize) -> Result<String, AppError> {
        let reply = self.overrides.lock().unwrap().get(&step).cloned();
        match reply {
            Some(Reply::Fixture(name)) => Ok(read_fixture(name)),
            Some(Reply::Sequence(names)) => {
                let name = names.get(call_index).or(names.last()).expect("séquence de fixtures non vide");
                Ok(read_fixture(name))
            }
            Some(Reply::Error(message)) => Err(AppError::Llm(message.to_string())),
            Some(Reply::Files(names)) => {
                let name = names.get(call_index).or(names.last()).expect("séquence de fichiers non vide");
                Ok(raw_file(name))
            }
            None => Ok(default_reply(step)),
        }
    }
}

impl LlmCaller for FakeLlm {
    fn call<'a>(&'a self, request: LlmRequest<'a>) -> LlmFuture<'a> {
        let step = step_of(request.system, request.output_schema.as_ref());
        let call_index = {
            let mut calls = self.calls.lock().unwrap();
            let index = calls.iter().filter(|c| c.step == step).count();
            calls.push(RecordedCall {
                step,
                user_message: request.user_message.to_string(),
                model: request.model.to_string(),
            });
            index
        };
        let reply = match step {
            // Sans surcharge, la traduction de la référence reprend chaque
            // élément demandé (mêmes identifiants), préfixé de « [en] ».
            Step::ReferenceTranslation if !self.overrides.lock().unwrap().contains_key(&step) => {
                Ok(echo_translation(request.user_message))
            }
            _ => self.reply_for(step, call_index),
        };
        Box::pin(async move { reply.map(|text| LlmResponse { text, stop_reason: Some("end_turn".to_string()), usage: None, request_id: None }) })
    }
}

/// Reconnaît l'étape d'après une propriété propre à son schéma de sortie ;
/// l'étape B de la templatisation, sans schéma, d'après son prompt système.
fn step_of(system: &str, schema: Option<&Value>) -> Step {
    if schema.is_none() && system.starts_with("Tu écris le template Typst") {
        return if system.contains("(CV)") { Step::TemplateCv } else { Step::TemplateLm };
    }
    let properties = schema
        .and_then(|s| s.get("properties"))
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("FakeLlm : appel sans schéma de sortie, étape inconnue"));
    let has = |key: &str| properties.contains_key(key);
    if has("identity") {
        return if has("period_style") { Step::ExampleCv } else { Step::ExampleLm };
    }
    let candidates = [
        ("items", Step::ReferenceTranslation),
        ("job_title", Step::OfferAnalysis),
        ("summary_strategy", Step::Matching),
        ("months_short", Step::CvLabels),
        ("subject_prefix", Step::LmLabels),
        ("skills_groups", Step::CvContent),
        ("paragraphs", Step::LmContent),
        ("formations", Step::ProfileTranslation),
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
        Step::ProfileTranslation => read_fixture("profile_translation.json"),
        Step::ScrapedOffer => read_fixture("scraped_offer.json"),
        Step::ReferenceTranslation => unreachable!("réponse construite depuis la demande"),
        Step::ExampleCv => read_reference("cv_example_data.json"),
        Step::ExampleLm => read_reference("lm_example_data.json"),
        Step::TemplateCv => typst_block(&read_reference("cv_template.typ")),
        Step::TemplateLm => typst_block(&read_reference("lm_template.typ")),
    }
}

/// Traduction factice des textes de la référence : chaque élément du
/// message, préfixé de « [en] ».
fn echo_translation(user_message: &str) -> String {
    let json = user_message.split_once("TEXTES À TRADUIRE :\n").map(|(_, rest)| rest).unwrap_or_default();
    let json = json.split("\n\nTA RÉPONSE PRÉCÉDENTE").next().unwrap_or_default();
    let mut request: Value = serde_json::from_str(json).expect("éléments à traduire en JSON");
    for item in request["items"].as_array_mut().expect("liste d'éléments") {
        let text = item["text"].as_str().unwrap_or_default().to_string();
        item["text"] = Value::String(format!("[en] {text}"));
    }
    request.to_string()
}

/// Template entouré d'un bloc ```typst```, comme une réponse de l'étape B.
fn typst_block(template: &str) -> String {
    format!("Voici le template.
```typst
{template}
```
")
}

/// Lit un fichier de `tests/fixtures/` ; un `.typ` est rendu comme une
/// réponse de l'étape B.
fn raw_file(name: &str) -> String {
    let path = fixtures_dir().join(name);
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {} illisible : {e}", path.display()));
    if name.ends_with(".typ") {
        typst_block(&content)
    } else {
        content
    }
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
