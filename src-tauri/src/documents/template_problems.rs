//! Formulation des problèmes d'un template refusé, pour le feedback du
//! LLM : écarts de texte localisés (phrase citée, position), code Typst
//! affiché comme du texte, erreurs de compilation dédoublonnées avec un
//! indice sur le mode markup / code.

// Importation des modules internes
use crate::docgen::text_diff::{normalize_quotes, words, DiffHunk};

/// Nombre maximal d'écarts de texte décrits ; au-delà, un résumé.
const MAX_HUNKS_REPORTED: usize = 8;
/// Mots cités au début d'une phrase.
const SENTENCE_HEAD_WORDS: usize = 6;
/// Mots cités à la fin d'une phrase.
const SENTENCE_TAIL_WORDS: usize = 4;
/// Mots cités dans un écart.
const MAX_HUNK_WORDS: usize = 12;
/// Indices d'un texte rendu qui ne peut être que du code Typst affiché.
const CODE_MARKERS: [&str; 9] = ["data.", "#let", "#if", "#for", "=>", "!=", "==", ".join(", ".len()"];
/// Messages Typst signalant un `#` en mode code.
const CODE_MODE_ERRORS: [&str; 2] = ["already in code mode", "the character `#` is not valid in code"];

/// Rendu d'un template, nommé pour les messages.
pub struct NamedRender<'a> {
    pub name: &'a str,
    pub result: &'a Result<String, String>,
}

/// Décrit les écarts de texte entre la référence et le rendu du template,
/// chacun situé dans sa phrase de la référence.
///
/// # Arguments
///
/// * `reference_text` - Texte rendu de la référence.
/// * `hunks` - Écarts (`text_diff::diff_hunks`).
///
/// # Returns
///
/// Un problème, ou `None` sans écart.
pub fn describe_hunks(reference_text: &str, hunks: &[DiffHunk]) -> Option<String> {
    if hunks.is_empty() {
        return None;
    }
    let normalized = normalize_quotes(reference_text);
    let reference = words(&normalized);
    let mut lines: Vec<String> = hunks.iter().take(MAX_HUNKS_REPORTED).map(|h| describe_hunk(&reference, h)).collect();
    if hunks.len() > MAX_HUNKS_REPORTED {
        lines.push(format!("… et {} autres écarts.", hunks.len() - MAX_HUNKS_REPORTED));
    }
    Some(format!(
        "Rendu avec les données d'exemple, le texte diffère de l'original ({} écart(s)). Rien de l'original n'est \
         retiré ni ajouté :\n{}",
        hunks.len(),
        lines.join("\n")
    ))
}

/// Décrit un écart dans sa phrase de la référence.
fn describe_hunk(reference: &[&str], hunk: &DiffHunk) -> String {
    let position = hunk.expected_start.min(reference.len().saturating_sub(1));
    let sentence = sentence_around(reference, position);
    let mut parts = Vec::new();
    if !hunk.missing.is_empty() {
        parts.push(format!("absent du rendu : « {} »", quote(&hunk.missing)));
    }
    if !hunk.extra.is_empty() {
        parts.push(format!("en trop dans le rendu : « {} »", quote(&hunk.extra)));
    }
    format!(
        "  - mot {} sur {} de l'original, dans « {} » : {}.",
        hunk.expected_start + 1,
        reference.len(),
        sentence,
        parts.join(" ; ")
    )
}

/// Phrase de la référence qui contient un mot, abrégée (début … fin).
fn sentence_around(reference: &[&str], position: usize) -> String {
    if reference.is_empty() {
        return String::new();
    }
    let ends_sentence = |w: &str| w.ends_with(['.', '!', '?', ':']);
    let start = (0..position).rev().find(|&i| ends_sentence(reference[i])).map_or(0, |i| i + 1);
    let end = (position..reference.len()).find(|&i| ends_sentence(reference[i])).map_or(reference.len(), |i| i + 1);
    let sentence = &reference[start..end];
    if sentence.len() <= SENTENCE_HEAD_WORDS + SENTENCE_TAIL_WORDS {
        return sentence.join(" ");
    }
    format!(
        "{} … {}",
        sentence[..SENTENCE_HEAD_WORDS].join(" "),
        sentence[sentence.len() - SENTENCE_TAIL_WORDS..].join(" ")
    )
}

/// Cite des mots, tronqués à `MAX_HUNK_WORDS`.
fn quote(words: &[String]) -> String {
    if words.len() <= MAX_HUNK_WORDS {
        return words.join(" ");
    }
    format!("{} … ({} mots)", words[..MAX_HUNK_WORDS].join(" "), words.len())
}

/// Repère du code Typst affiché comme du texte dans un rendu.
///
/// # Arguments
///
/// * `rendered_text` - Texte rendu avec les données d'exemple.
/// * `reference_text` - Texte rendu de la référence (un marqueur qu'elle
///   affiche elle-même n'est pas du code).
///
/// # Returns
///
/// Un problème qui nomme la cause, ou `None`.
pub fn code_rendered_as_text(rendered_text: &str, reference_text: &str) -> Option<String> {
    let suspects: Vec<&str> = words(rendered_text)
        .into_iter()
        .filter(|w| CODE_MARKERS.iter().any(|m| w.contains(m)) || *w == "{" || *w == "}")
        .filter(|w| !reference_text.contains(*w))
        .collect();
    if suspects.is_empty() {
        return None;
    }
    let mut shown: Vec<&str> = suspects.clone();
    shown.dedup();
    shown.truncate(MAX_HUNK_WORDS);
    Some(format!(
        "Le rendu affiche du code Typst comme du texte : « {} ». Ce code est écrit en mode markup (corps du document \
         ou bloc [ … ]) sans # : en markup, chaque instruction commence par # (#data.summary, #if …, #for …). Dans \
         un bloc {{ … }} au contraire, jamais de #.",
        shown.join(" ")
    ))
}

/// Erreurs de compilation des rendus, dédoublonnées : une même erreur
/// n'est citée qu'une fois, avec la liste des rendus qui la produisent.
///
/// # Arguments
///
/// * `renders` - Rendus nommés (« exemple », « vide »…), texte ou erreur.
///
/// # Returns
///
/// Un problème par erreur distincte.
pub fn compile_problems(renders: &[NamedRender<'_>]) -> Vec<String> {
    let mut distinct: Vec<(String, Vec<&str>)> = Vec::new();
    for render in renders {
        let Err(error) = render.result else {
            continue;
        };
        match distinct.iter_mut().find(|(known, _)| known == error) {
            Some((_, names)) => names.push(render.name),
            None => distinct.push((error.clone(), vec![render.name])),
        }
    }
    distinct
        .into_iter()
        .map(|(error, names)| {
            let hint = if CODE_MODE_ERRORS.iter().any(|m| error.contains(m)) {
                " Indice : ce # est écrit alors qu'on est déjà en mode code (dans { … }, #( … ) ou les arguments \
                 d'une fonction) ; retire-le."
            } else {
                ""
            };
            format!("Le template ne compile pas (données : {}) :\n{error}{hint}", names.join(", "))
        })
        .collect()
}
