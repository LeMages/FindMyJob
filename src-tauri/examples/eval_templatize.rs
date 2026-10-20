//! Évaluation manuelle de la templatisation : lance N templatisations
//! réelles (API Claude, modèle imposé) d'une référence et rapporte, pour
//! chacune, les essais, la durée, les tokens et le coût estimé, puis le
//! taux de réussite.
//!
//! Hors suite de tests (appels réels payants) :
//!
//! ```text
//! cargo run -p findmyjob --example eval_templatize -- <reference.typ> <cv|lettre_motivation> [runs] [debug_dir]
//! ```
//!
//! La clé est lue dans `ANTHROPIC_API_KEY` (le `.env` du dépôt est chargé).
//! Avec `debug_dir`, chaque run garde ses essais et ses appels comme en DEV.

// Importation des modules standard
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Importation des modules internes
use findmyjob_lib::documents::templatize::{templatize, TemplatizeInput};
use findmyjob_lib::documents::typst::compile_document;
use findmyjob_lib::llm::{AnthropicCaller, ApiKey, ClaudeTarget, LlmCaller, LlmFuture, LlmRequest, TokenUsage};

/// Tarif de `TEMPLATIZE_MODEL` (Claude Sonnet 4.6), en dollars par million
/// de tokens d'entrée et de sortie.
const PRICE_PER_MILLION: (f64, f64) = (3.0, 15.0);
/// Nombre de runs par défaut.
const DEFAULT_RUNS: usize = 5;

/// Exécutant réel qui additionne les tokens consommés.
#[derive(Debug, Default)]
struct CountingCaller {
    usage: Mutex<Vec<TokenUsage>>,
}

impl CountingCaller {
    /// Retire et additionne les tokens comptés depuis le dernier appel.
    fn take(&self) -> (u64, u64, usize) {
        let usage = std::mem::take(&mut *self.usage.lock().expect("verrou des tokens"));
        let input = usage.iter().map(|u| u64::from(u.input_tokens)).sum();
        let output = usage.iter().map(|u| u64::from(u.output_tokens)).sum();
        (input, output, usage.len())
    }
}

impl LlmCaller for CountingCaller {
    fn call<'a>(&'a self, request: LlmRequest<'a>) -> LlmFuture<'a> {
        Box::pin(async move {
            let response = AnthropicCaller.call(request).await?;
            if let Some(usage) = response.usage {
                self.usage.lock().expect("verrou des tokens").push(usage);
            }
            Ok(response)
        })
    }
}

/// Arguments de la ligne de commande.
struct Args {
    reference: PathBuf,
    doc_type: String,
    runs: usize,
    debug_dir: Option<PathBuf>,
}

/// Lit les arguments.
fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage : eval_templatize <reference.typ> <cv|lettre_motivation> [runs] [debug_dir]";
    let reference = args.first().ok_or(usage)?.into();
    let doc_type = args.get(1).ok_or(usage)?.clone();
    if doc_type != "cv" && doc_type != "lettre_motivation" {
        return Err(usage.to_string());
    }
    let runs = match args.get(2) {
        Some(raw) => raw.parse().map_err(|_| format!("nombre de runs invalide : {raw}"))?,
        None => DEFAULT_RUNS,
    };
    Ok(Args { reference, doc_type, runs, debug_dir: args.get(3).map(PathBuf::from) })
}

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

/// Déroule l'évaluation.
fn run() -> Result<(), String> {
    let args = parse_args()?;
    // Un .env absent n'est pas une erreur : la clé peut venir de l'environnement.
    if let Err(e) = dotenvy::dotenv() {
        eprintln!(".env non chargé : {e}");
    }
    let key = std::env::var("ANTHROPIC_API_KEY").ok().and_then(|raw| ApiKey::parse(&raw)).ok_or("ANTHROPIC_API_KEY absente")?;
    let source = std::fs::read_to_string(&args.reference).map_err(|e| format!("{} illisible : {e}", args.reference.display()))?;
    let root = std::env::temp_dir().join(format!("findmyjob-eval-{}", std::process::id()));
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let fonts = root.join("polices");
    let rendered = compile_document(source.clone(), root.clone(), fonts.clone())?;
    let caller = Arc::new(CountingCaller::default());
    let target = ClaudeTarget { api_key: key, model: String::new(), caller: caller.clone(), recorder: None };
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?;

    let mut accepted_within_two = 0;
    let mut accepted = 0;
    for run in 1..=args.runs {
        let input = TemplatizeInput { source: &source, rendered: &rendered, doc_type: &args.doc_type, root: &root, fonts_dir: &fonts };
        let debug = args.debug_dir.as_ref().map(|dir| dir.join(format!("run_{run}")));
        let started = Instant::now();
        let result = runtime.block_on(templatize(&input, &target, debug.as_deref()));
        let seconds = started.elapsed().as_secs_f64();
        let (input_tokens, output_tokens, calls) = caller.take();
        let cost = cost(input_tokens, output_tokens);
        match result {
            Ok(validated) => {
                accepted += 1;
                if validated.template_attempts <= 2 {
                    accepted_within_two += 1;
                }
                println!(
                    "run {run} : accepté — données {} essai(s), template {} essai(s), {calls} appels, {seconds:.0} s, \
                     {input_tokens} / {output_tokens} tokens, {cost:.3} $",
                    validated.example_attempts, validated.template_attempts
                );
            }
            Err(e) => println!(
                "run {run} : ÉCHEC — {calls} appels, {seconds:.0} s, {input_tokens} / {output_tokens} tokens, {cost:.3} $\n  {e}"
            ),
        }
    }
    println!(
        "\n{} : {accepted}/{} acceptés, dont {accepted_within_two} en au plus 2 essais de template.",
        display(&args.reference),
        args.runs
    );
    std::fs::remove_dir_all(&root).map_err(|e| e.to_string())
}

/// Coût estimé en dollars.
fn cost(input_tokens: u64, output_tokens: u64) -> f64 {
    // Conversion pour un affichage approché : la perte de précision au-delà
    // de 2^52 tokens est sans objet.
    #[allow(clippy::cast_precision_loss)]
    let (input, output) = (input_tokens as f64, output_tokens as f64);
    (input * PRICE_PER_MILLION.0 + output * PRICE_PER_MILLION.1) / 1_000_000.0
}

/// Nom de fichier lisible.
fn display(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}
