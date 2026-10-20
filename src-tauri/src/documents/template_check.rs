//! Validation d'un template (étape B de la templatisation) par
//! compilation. Un template est accepté s'il :
//! - lit ses données depuis `data.json` et n'évalue ni ne calcule rien
//!   (`eval`, `datetime.today`, `read` interdits) ;
//! - compilé avec les données d'exemple, n'a pas plus de pages que la
//!   référence et rend exactement son texte (seule une date calculée à la
//!   compilation peut différer), paragraphes de la lettre compris ;
//! - compile avec des données vides ;
//! - compilé avec les champs secondaires vides, ne laisse aucun séparateur
//!   orphelin ;
//! - compilé avec un marqueur unique dans chaque chaîne (rendu sentinelle),
//!   n'affiche plus aucun texte écrit en dur : tout texte visible vient d'un
//!   champ, d'un libellé ou d'un texte de la référence (`data.texts`).
//!
//! Tous les contrôles s'exécutent à chaque essai ; une même erreur de
//! compilation n'est citée qu'une fois.

// Importation des modules standard
use std::collections::BTreeMap;
use std::path::Path;

// Importation des modules internes
use crate::clock;
use crate::docgen::date_format::format_long_date;
use crate::docgen::example_facts::word_keys;
use crate::docgen::sentinel_data::{hardcoded_runs, hardcoded_values, is_month_word, sentinel_cv_data, sentinel_lm_data};
use crate::docgen::text_diff::{diff_hunks, normalize_quotes, DiffHunk};
use crate::documents::example_check::CheckedExample;
use crate::documents::markup::hard_texts::add_text;
use crate::documents::paragraph_check::paragraph_problems;
use crate::documents::sparse_check::{cleared_data, sparse_problems, SPARSE_FIELDS};
use crate::documents::template_problems::{code_rendered_as_text, compile_problems, describe_hunks, NamedRender};
use crate::documents::typst::{compile_template, CompiledDocument};
use crate::error::AppError;
use crate::models::{CvData, LmData};

/// Constructions interdites dans un template : elles évalueraient une valeur
/// comme du code ou rendraient le document dépendant de la date de compilation.
const FORBIDDEN_CONSTRUCTS: [&str; 3] = ["eval(", "datetime.today", "read("];

/// Ce que la validation doit savoir de la référence et de la compilation.
///
/// # Attributes
///
/// * `reference` - Rendu de la référence.
/// * `computes_date` - Vrai si la référence calcule sa date (`datetime.today`).
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `root` - Dossier racine de compilation.
/// * `fonts_dir` - Dossier des polices importées par l'utilisateur.
pub struct TemplateContext<'a> {
    pub reference: &'a CompiledDocument,
    pub computes_date: bool,
    pub doc_type: &'a str,
    pub root: &'a Path,
    pub fonts_dir: &'a Path,
}

/// Verdict sur un template.
///
/// # Attributes
///
/// * `problems` - Tous les problèmes relevés (vide si accepté).
/// * `blocking` - Nombre de problèmes bloquants (tous sauf les séparateurs
///   orphelins, tolérés au dernier essai).
/// * `page_count` - Pages du rendu avec les données d'exemple, s'il compile.
/// * `texts` - Textes en dur, complétés de ceux découverts dans le rendu
///   sentinelle (pour l'essai suivant).
pub struct TemplateVerdict {
    pub problems: Vec<String>,
    pub blocking: usize,
    pub page_count: Option<usize>,
    pub texts: BTreeMap<String, String>,
}

impl TemplateVerdict {
    /// Score de l'essai (plus petit = meilleur) : problèmes bloquants, puis total.
    pub fn score(&self) -> (usize, usize) {
        (self.blocking, self.problems.len())
    }
}

/// Valide un template avec des données d'exemple validées.
///
/// # Arguments
///
/// * `template` - Source du template.
/// * `checked` - Données d'exemple validées (étape A).
/// * `context` - Référence et dossiers de compilation.
///
/// # Returns
///
/// `TemplateVerdict`.
pub async fn check_template(
    template: &str,
    checked: &CheckedExample,
    context: &TemplateContext<'_>,
) -> Result<TemplateVerdict, AppError> {
    // ------------------------------------------------------------------
    // 1. Rendus avec les données d'exemple, vides, clairsemées et
    //    sentinelles (compilation bloquante, hors du runtime async)
    // ------------------------------------------------------------------
    let example = example_for_rendering(checked, context)?;
    let data_sets = [example.clone(), blank_data(checked, context.doc_type)?, cleared_data(&example, &SPARSE_FIELDS)?, sentinel_data(checked, context.doc_type)?];
    let (owned, root, fonts_dir) = (template.to_string(), context.root.to_path_buf(), context.fonts_dir.to_path_buf());
    let [with_example, with_blank, with_sparse, with_sentinel] = tokio::task::spawn_blocking(move || {
        data_sets.map(|data| compile_template(owned.clone(), data, root.clone(), fonts_dir.clone()))
    })
    .await
    .map_err(|e| AppError::Validation(format!("Erreur interne de compilation Typst : {e}")))?;

    // ------------------------------------------------------------------
    // 2. Contrôles statiques et erreurs de compilation, dédoublonnées
    // ------------------------------------------------------------------
    let mut problems = static_problems(template);
    let texts_of = |r: &Result<CompiledDocument, String>| r.as_ref().map(|d| d.text.clone()).map_err(Clone::clone);
    let results = [texts_of(&with_example), texts_of(&with_blank), texts_of(&with_sparse), texts_of(&with_sentinel)];
    let names = ["exemple", "vides", "champs secondaires vides", "un marqueur par champ"];
    let renders: Vec<NamedRender> =
        names.iter().zip(&results).map(|(name, result)| NamedRender { name, result }).collect();
    problems.extend(compile_problems(&renders));

    // ------------------------------------------------------------------
    // 3. Fidélité à la référence, puis texte resté en dur
    // ------------------------------------------------------------------
    let page_count = with_example.as_ref().ok().map(|rendered| rendered.page_count);
    if let Ok(rendered) = &with_example {
        problems.extend(compare_with_reference(context, rendered));
        problems.extend(paragraph_problems(&context.reference.lines, &rendered.lines, &checked.paragraphs));
    }
    let mut texts = checked.texts.clone();
    if let Ok(rendered) = &with_sentinel {
        problems.extend(hardcoded_problems(&rendered.text, checked, &context.reference.text, &mut texts));
    }

    // ------------------------------------------------------------------
    // 4. Séparateurs orphelins (seul défaut toléré en dernier recours)
    // ------------------------------------------------------------------
    let blocking = problems.len();
    if with_sparse.is_ok() {
        let (root, fonts_dir) = (context.root.to_path_buf(), context.fonts_dir.to_path_buf());
        problems.extend(sparse_problems(with_sparse, template, &example, root, fonts_dir).await?);
    }
    Ok(TemplateVerdict { problems, blocking, page_count, texts })
}

/// Contrôles statiques : chargement de `data.json` et constructions interdites.
fn static_problems(template: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !template.contains("json(\"data.json\")") {
        problems.push("Le template doit charger ses données avec `#let data = json(\"data.json\")`.".to_string());
    }
    for construct in FORBIDDEN_CONSTRUCTS {
        if template.contains(construct) {
            problems.push(format!("Le template utilise `{construct}`, interdit."));
        }
    }
    problems
}

/// Compare le rendu du template avec celui de la référence : pas plus de
/// pages, exactement les mêmes mots, pas de code affiché.
fn compare_with_reference(context: &TemplateContext<'_>, rendered: &CompiledDocument) -> Vec<String> {
    let mut problems = Vec::new();
    if rendered.page_count > context.reference.page_count {
        problems.push(format!(
            "Rendu avec les données d'exemple, le template fait {} page(s), plus que l'original ({}).",
            rendered.page_count, context.reference.page_count
        ));
    }
    problems.extend(code_rendered_as_text(&rendered.text, &context.reference.text));
    match diff_hunks(&context.reference.text, &rendered.text) {
        Ok(hunks) => {
            let hunks: Vec<DiffHunk> =
                hunks.into_iter().filter(|h| !(context.computes_date && is_date_hunk(h))).collect();
            problems.extend(describe_hunks(&context.reference.text, &hunks));
            problems.extend(glued_words(&hunks));
        }
        Err(e) => problems.push(e),
    }
    problems
}

/// Vrai si un écart ne porte que sur une date (calculée à la compilation).
fn is_date_hunk(hunk: &DiffHunk) -> bool {
    let is_date_word = |word: &String| {
        let cleaned = word.trim_matches(|c: char| c == '.' || c == ',');
        (!cleaned.is_empty() && cleaned.chars().all(|c| c.is_ascii_digit())) || is_month_word(word)
    };
    hunk.missing.iter().chain(&hunk.extra).all(is_date_word)
}

/// Signale chaque mot en trop qui colle deux mots manquants consécutifs :
/// deux textes rendus sans séparateur.
fn glued_words(hunks: &[DiffHunk]) -> Vec<String> {
    hunks
        .iter()
        .flat_map(|hunk| {
            hunk.extra.iter().filter_map(|word| {
                let glued = normalize_quotes(word);
                let pair = hunk
                    .missing
                    .windows(2)
                    .find(|pair| normalize_quotes(&format!("{}{}", pair[0], pair[1])) == glued)?;
                Some(format!(
                    "Le mot rendu « {word} » colle « {} » et « {} » : deux textes sont rendus sans séparateur. Si ce \
                     sont la fin d'un paragraphe et le début du suivant, chaque entrée de paragraphs doit être rendue \
                     comme un paragraphe distinct.",
                    pair[0], pair[1]
                ))
            })
        })
        .collect()
}

/// Problèmes du rendu sentinelle : valeurs d'un champ écrites en dur, et
/// textes de la référence écrits en dur au lieu d'être lus dans
/// `data.texts`. Un texte en dur de l'original encore sans identifiant en
/// reçoit un (`texts`), pour l'essai suivant.
fn hardcoded_problems(
    sentinel_text: &str,
    checked: &CheckedExample,
    reference_text: &str,
    texts: &mut BTreeMap<String, String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    let values = hardcoded_values(sentinel_text, &checked.values);
    if !values.is_empty() {
        let quoted: Vec<String> = values.iter().map(|v| format!("« {v} »")).collect();
        problems.push(format!(
            "Ces valeurs de champs sont écrites en dur dans le template : {}. Une valeur qui a un champ vient \
             toujours de data, même au milieu d'une phrase recopiée.",
            quoted.join(", ")
        ));
    }
    let reference_keys = word_keys(reference_text);
    for run in hardcoded_runs(sentinel_text) {
        let keys = word_keys(&run);
        let is_value = values.iter().any(|v| contains(&word_keys(v), &keys) || contains(&keys, &word_keys(v)));
        if is_value || run.split_whitespace().all(is_month_word) {
            continue;
        }
        if !contains(&reference_keys, &keys) {
            problems.push(format!("Le template affiche « {run} », qui n'est pas dans l'original."));
            continue;
        }
        let id = add_text(texts, &run);
        problems.push(format!(
            "Le texte « {run} » est écrit en dur dans le template : lis-le dans data.texts.{id} (les textes de \
             l'original sont traduits avec le document)."
        ));
    }
    problems
}

/// Vrai si `run` apparaît, à la suite, dans `haystack`.
fn contains(haystack: &[String], run: &[String]) -> bool {
    !run.is_empty() && haystack.windows(run.len()).any(|window| window == run)
}

/// Données d'exemple sérialisées pour le rendu de validation. Une lettre
/// dont la référence calcule sa date n'a pas de date dans ses données : la
/// date du jour y est placée, comme à la génération.
fn example_for_rendering(checked: &CheckedExample, context: &TemplateContext<'_>) -> Result<Vec<u8>, AppError> {
    let mut example = checked.example.clone();
    let date_is_empty = example.get("date").and_then(|v| v.as_str()).is_some_and(str::is_empty);
    if context.computes_date && date_is_empty {
        example["date"] = serde_json::Value::String(format_long_date(clock::today_local(), &checked.lang));
    }
    to_bytes(&example)
}

/// Données vides : chaînes et listes vides, mais la langue et les textes
/// de la référence (toujours fournis à la génération).
fn blank_data(checked: &CheckedExample, doc_type: &str) -> Result<Vec<u8>, AppError> {
    match doc_type {
        "cv" => {
            let mut blank = CvData { lang: checked.lang.clone(), texts: checked.texts.clone(), ..Default::default() };
            blank.labels.months_short = vec![String::new(); 12];
            to_bytes(&blank)
        }
        _ => to_bytes(&LmData { lang: checked.lang.clone(), texts: checked.texts.clone(), ..Default::default() }),
    }
}

/// Données sentinelles du type de document.
fn sentinel_data(checked: &CheckedExample, doc_type: &str) -> Result<Vec<u8>, AppError> {
    let ids: Vec<String> = checked.texts.keys().cloned().collect();
    let data = match doc_type {
        "cv" => sentinel_cv_data(&checked.lang, &ids),
        _ => sentinel_lm_data(&checked.lang, &ids),
    }
    .map_err(|e| AppError::Validation(format!("Données sentinelles non sérialisables : {e}")))?;
    to_bytes(&data)
}

/// Sérialise des données de rendu.
fn to_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, AppError> {
    serde_json::to_vec(value).map_err(|e| AppError::Validation(format!("Données de rendu non sérialisables : {e}")))
}
