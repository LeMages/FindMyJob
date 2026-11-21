//! Contrôles d'une référence sur ses rendus de contrôle (voir
//! `render_data`), chacun traduit en problèmes typés : valeurs d'exemple
//! écrites en dur, titres orphelins, symbole de liste, tirets, bloc
//! destinataire et séparateurs orphelins.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::dash_normalization::{dash_marks, DashMark};
use crate::docgen::example_facts::word_keys;
use crate::docgen::sentinel_data::{hardcoded_values, strip_markers};
use crate::documents::list_marker_check::list_marker_mismatch;
use crate::documents::recipient_check::recipient_misalignment;
use crate::documents::typst::CompiledDocument;
use crate::models::{ReferenceIssue, ReferenceIssueCode};

/// Nombre de mots maximal d'un texte court : une valeur d'un seul mot
/// (ville, prénom) n'est signalée que dans un texte court (en-tête,
/// coordonnées), pas au milieu d'une phrase fixe qui la cite légitimement
/// (« l'École du Numérique de Lyon »).
const SHORT_SEGMENT_WORDS: usize = 3;

/// Valeurs d'exemple d'identité ou du profil encore visibles dans le rendu
/// sentinelle, où toute donnée est remplacée par un marqueur.
///
/// # Arguments
///
/// * `sentinel` - Rendu avec les données sentinelles.
/// * `values` - Valeurs d'exemple qui viennent du profil ou de la session
///   (`docgen::example_facts::*_information_values`).
///
/// # Returns
///
/// Un problème `hardcoded_value` par valeur retrouvée.
pub fn hardcoded_issues(sentinel: &CompiledDocument, values: &[String]) -> Vec<ReferenceIssue> {
    // Segments de texte en dur, ligne par ligne : un marqueur coupe un segment.
    let segments: Vec<Vec<String>> = sentinel
        .lines
        .iter()
        .flat_map(|line| {
            strip_markers(&line.words.join(" ")).split(" | ").map(word_keys).collect::<Vec<_>>()
        })
        .filter(|keys| !keys.is_empty())
        .collect();
    values
        .iter()
        .filter(|value| match word_keys(value).as_slice() {
            [] => false,
            [single] => segments.iter().any(|seg| seg.len() <= SHORT_SEGMENT_WORDS && seg.contains(single)),
            _ => !hardcoded_values(&sentinel.text, std::slice::from_ref(value)).is_empty(),
        })
        .map(|value| ReferenceIssue::new(ReferenceIssueCode::HardcodedValue).with_detail(value))
        .collect()
}

/// Titres de section restés affichés sans contenu, dans le rendu aux
/// listes vides.
///
/// # Arguments
///
/// * `empty_lists` - Rendu avec les listes du CV vides.
///
/// # Returns
///
/// Un problème `orphan_heading` par titre orphelin.
pub fn orphan_heading_issues(empty_lists: &CompiledDocument) -> Vec<ReferenceIssue> {
    empty_lists
        .headings
        .iter()
        .filter(|heading| !heading.has_content)
        .map(|heading| ReferenceIssue::new(ReferenceIssueCode::OrphanHeading).with_detail(&heading.title))
        .collect()
}

/// Symbole de liste du rendu avec un symbole sentinelle.
///
/// # Arguments
///
/// * `rendered` - Rendu avec `list_marker` = `sentinel`.
/// * `example` - Valeurs d'exemple (éléments de description).
/// * `sentinel` - Symbole sentinelle.
///
/// # Returns
///
/// Un problème `list_marker_wrong` si le symbole ne précède pas exactement
/// chaque élément.
pub fn list_marker_issue(rendered: &CompiledDocument, example: &Value, sentinel: &str) -> Option<ReferenceIssue> {
    let mismatch = list_marker_mismatch(&rendered.text, example, sentinel)?;
    let detail = if mismatch.unmarked.is_empty() {
        format!("{} shown for {} items", mismatch.shown, mismatch.expected)
    } else {
        format!("not before: {}", mismatch.unmarked.join(", "))
    };
    Some(ReferenceIssue::new(ReferenceIssueCode::ListMarkerWrong).at_path("list_marker").with_detail(detail))
}

/// Tirets cadratins ou demi-cadratins du rendu de l'exemple dans une langue.
///
/// # Arguments
///
/// * `lang` - Langue du rendu.
/// * `text` - Texte rendu.
///
/// # Returns
///
/// Un avertissement `dash_marks` si le rendu en contient.
pub fn dash_issue(lang: &str, text: &str) -> Option<ReferenceIssue> {
    let marks = dash_marks(text);
    if marks.is_empty() {
        return None;
    }
    let shown: Vec<&str> = marks.iter().map(|mark| raw_mark(*mark)).collect();
    Some(ReferenceIssue::new(ReferenceIssueCode::DashMarks).with_detail(format!("{lang}: {}", shown.join(" "))))
}

/// Marque telle qu'écrite dans le texte.
fn raw_mark(mark: DashMark) -> &'static str {
    match mark {
        DashMark::EmDash => "\u{2014}",
        DashMark::EnDash => "\u{2013}",
        DashMark::TripleHyphen => "---",
        DashMark::DoubleHyphen => "--",
    }
}

/// Bloc destinataire réduit à l'entreprise et la ville, plus calé à droite.
///
/// # Arguments
///
/// * `with_example` - Rendu de l'exemple.
/// * `example` - Valeurs d'exemple de la lettre.
/// * `sparse` - Rendu au destinataire réduit.
///
/// # Returns
///
/// Un avertissement `recipient_not_right_aligned` si le bloc a décroché.
pub fn recipient_issue(with_example: &CompiledDocument, example: &Value, sparse: &CompiledDocument) -> Option<ReferenceIssue> {
    let (block_right, margin) = recipient_misalignment(with_example, example, sparse)?;
    Some(
        ReferenceIssue::new(ReferenceIssueCode::RecipientNotRightAligned)
            .at_path("recipient")
            .with_detail(format!("{block_right:.0} pt / {margin:.0} pt")),
    )
}
