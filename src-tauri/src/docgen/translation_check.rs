//! Validation d'une traduction des textes du profil affichés par le CV
//! (`CvEntryTexts`) : mêmes entrées, même nombre de lignes, chaque texte
//! traduit non vide et pas plus long qu'environ 110 % de l'original. Une
//! traduction fidèle ne rallonge pas : un texte plus court est accepté.
//! Fonctions pures, sans LLM.

// Importation des modules internes
use crate::docgen::entry_texts::{CvEntryTexts, EntryLines, LanguageTexts};

/// Allongement toléré d'un texte traduit, en pourcentage de l'original.
pub const LENGTH_TOLERANCE_PERCENT: usize = 10;
/// Allongement toléré en caractères, quelle que soit la longueur : sur un
/// texte court (« Mairie » → « Town hall »), 10 % ne laissent aucune marge.
pub const SHORT_TEXT_MARGIN_CHARS: usize = 10;

/// Longueur maximale, en caractères, de la traduction d'un texte.
///
/// # Arguments
///
/// * `original_chars` - Nombre de caractères de l'original.
///
/// # Returns
///
/// `max(ceil(original × 1,10), original + 10)`.
pub fn max_translated_chars(original_chars: usize) -> usize {
    (original_chars * (100 + LENGTH_TOLERANCE_PERCENT))
        .div_ceil(100)
        .max(original_chars + SHORT_TEXT_MARGIN_CHARS)
}

/// Vérifie un texte traduit contre son original.
fn text_problem(path: &str, original: &str, translated: &str) -> Option<String> {
    let translated = translated.trim();
    if original.trim().is_empty() {
        return (!translated.is_empty()).then(|| format!("{path} : l'original est vide, la traduction doit l'être aussi."));
    }
    if translated.is_empty() {
        return Some(format!("{path} : la traduction est vide."));
    }
    if translated.contains('\n') {
        return Some(format!("{path} : la traduction tient sur une seule ligne, sans saut de ligne."));
    }
    let (chars, max) = (translated.chars().count(), max_translated_chars(original.trim().chars().count()));
    (chars > max).then(|| {
        format!("{path} : la traduction fait {chars} caractères, au plus {max} (original : « {} »).", original.trim())
    })
}

/// Vérifie les lignes d'une entrée traduite.
fn lines_problems(path: &str, original: &[String], translated: &[String]) -> Vec<String> {
    if original.len() != translated.len() {
        return vec![format!(
            "{path}.lines contient {} lignes, attendu exactement {} (une par ligne de l'original).",
            translated.len(),
            original.len()
        )];
    }
    original
        .iter()
        .zip(translated)
        .enumerate()
        .filter_map(|(i, (o, t))| text_problem(&format!("{path}.lines[{i}]"), o, t))
        .collect()
}

/// Vérifie que les identifiants traduits sont exactement ceux de l'original.
fn ids_problems(kind: &str, original: &[i64], translated: &[i64]) -> Vec<String> {
    let mut problems = Vec::new();
    for (rank, id) in translated.iter().enumerate() {
        if !original.contains(id) {
            problems.push(format!("{kind} : l'id {id} n'est pas une entrée à traduire (ids attendus : {original:?})."));
        } else if translated[..rank].contains(id) {
            problems.push(format!("{kind} : l'id {id} apparaît plusieurs fois."));
        }
    }
    problems.extend(original.iter().filter(|id| !translated.contains(id)).map(|id| format!("{kind} : l'id {id} manque.")));
    problems
}

/// Vérifie une section d'entrées à lignes (expériences ou formations).
fn section_problems(kind: &str, original: &[EntryLines], translated: &[EntryLines]) -> Vec<String> {
    let ids = |entries: &[EntryLines]| entries.iter().map(|e| e.id).collect::<Vec<_>>();
    let mut problems = ids_problems(kind, &ids(original), &ids(translated));
    for source in original {
        if let Some(entry) = translated.iter().find(|e| e.id == source.id) {
            problems.extend(lines_problems(&format!("{kind} id {}", source.id), &source.lines, &entry.lines));
        }
    }
    problems
}

/// Vérifie une traduction des textes d'entrées du CV.
///
/// # Arguments
///
/// * `original` - Textes dans la langue du profil.
/// * `translated` - Textes rendus par la traduction.
///
/// # Returns
///
/// Problèmes relevés, formulés pour être renvoyés au LLM ; vide si la
/// traduction est valide.
pub fn translation_problems(original: &CvEntryTexts, translated: &CvEntryTexts) -> Vec<String> {
    let mut problems = section_problems("experiences", &original.experiences, &translated.experiences);
    problems.extend(section_problems("formations", &original.formations, &translated.formations));

    let ids = |texts: &CvEntryTexts| texts.projects.iter().map(|p| p.id).collect::<Vec<_>>();
    problems.extend(ids_problems("projects", &ids(original), &ids(translated)));
    for source in &original.projects {
        let Some(project) = translated.project(source.id) else {
            continue;
        };
        let path = format!("projects id {}", source.id);
        problems.extend(text_problem(&format!("{path}.pour_qui"), &source.pour_qui, &project.pour_qui));
        problems.extend(lines_problems(&path, &source.lines, &project.lines));
    }
    problems.extend(language_problems(&original.langues, &translated.langues));
    if original.interets.len() == translated.interets.len() {
        for (i, (o, t)) in original.interets.iter().zip(&translated.interets).enumerate() {
            problems.extend(text_problem(&format!("interets[{i}]"), o, t));
        }
    } else {
        problems.push(format!(
            "interets contient {} éléments, attendu exactement {}.",
            translated.interets.len(),
            original.interets.len()
        ));
    }
    problems
}

/// Vérifie les langues parlées traduites.
fn language_problems(original: &[LanguageTexts], translated: &[LanguageTexts]) -> Vec<String> {
    let ids = |langues: &[LanguageTexts]| langues.iter().map(|l| l.id).collect::<Vec<_>>();
    let mut problems = ids_problems("langues", &ids(original), &ids(translated));
    for source in original {
        if let Some(langue) = translated.iter().find(|l| l.id == source.id) {
            let path = format!("langues id {}", source.id);
            problems.extend(text_problem(&format!("{path}.nom"), &source.nom, &langue.nom));
            problems.extend(text_problem(&format!("{path}.level"), &source.level, &langue.level));
        }
    }
    problems
}
