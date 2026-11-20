//! Contrôle du symbole des listes d'un template de CV : le template doit
//! placer `data.list_marker` devant chaque élément de description
//! (`bullets`, `details`), et nulle part ailleurs. Le template est rendu avec
//! les données d'exemple dont le symbole est remplacé par un caractère
//! sentinelle absent de la référence : ce caractère doit apparaître
//! exactement une fois par élément, juste avant lui. Un symbole écrit en dur
//! (« • ») ne suivrait pas le format de la session ; un séparateur de
//! l'en-tête branché sur `list_marker` le suivrait à tort.
//!
//! La comparaison du rendu d'exemple avec la référence tolère, elle, un
//! symbole ajouté devant un élément : le symbole de la session s'applique à
//! toutes les sections de description, même celles que la référence
//! affichait sans symbole ou avec un autre.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::text_diff::{normalize_quotes, word_key, words, DiffHunk};
use crate::error::AppError;

/// Caractères sentinelles candidats, dans l'ordre : le premier absent du
/// texte de la référence est utilisé. Tous sont couverts par les polices
/// embarquées de Typst et le sous-ensemble Arimo des tests.
const SENTINEL_CANDIDATES: [&str; 3] = ["¤", "¶", "§"];

/// Sections de description du CV et le nom de leur liste d'éléments.
const DESCRIPTION_LISTS: [(&str, &str); 3] = [("experiences", "bullets"), ("projects", "bullets"), ("formations", "details")];

/// Choisit un symbole sentinelle absent de la référence.
///
/// # Arguments
///
/// * `reference_text` - Texte rendu de la référence.
///
/// # Returns
///
/// Le premier candidat absent, ou `None` si la référence les contient tous
/// (le contrôle est alors sauté).
pub fn pick_sentinel(reference_text: &str) -> Option<&'static str> {
    SENTINEL_CANDIDATES.into_iter().find(|candidate| !reference_text.contains(candidate))
}

/// Données d'exemple dont le symbole des listes est remplacé par la sentinelle.
///
/// # Arguments
///
/// * `example` - Données d'exemple sérialisées (rendu de validation).
/// * `sentinel` - Symbole sentinelle.
///
/// # Returns
///
/// Les données modifiées, sérialisées.
pub fn sentinel_marker_data(example: &[u8], sentinel: &str) -> Result<Vec<u8>, AppError> {
    let mut data: Value = serde_json::from_slice(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple illisibles pour le contrôle des listes : {e}")))?;
    data["list_marker"] = Value::String(sentinel.to_string());
    serde_json::to_vec(&data)
        .map_err(|e| AppError::Validation(format!("Données de rendu non sérialisables : {e}")))
}

/// Premiers mots de chaque élément de description des données d'exemple.
fn item_first_words(example: &Value) -> Vec<String> {
    DESCRIPTION_LISTS
        .iter()
        .flat_map(|(section, list)| {
            example[*section].as_array().into_iter().flatten().flat_map(move |entry| {
                entry[*list].as_array().into_iter().flatten().filter_map(|item| item.as_str())
            })
        })
        .filter_map(|item| item.split_whitespace().next().map(|word| normalize_quotes(word_key(word))))
        .collect()
}

/// Vrai si le symbole précède directement le mot attendu quelque part (séparé
/// par un espace, ou collé à lui).
fn marker_before(rendered: &[&str], sentinel: &str, first_word: &str) -> bool {
    let glued = |word: &str| word.strip_prefix(sentinel).is_some_and(|rest| normalize_quotes(word_key(rest)) == first_word);
    rendered.windows(2).any(|pair| pair[0] == sentinel && normalize_quotes(word_key(pair[1])) == first_word)
        || rendered.iter().any(|word| glued(word))
}

/// Problèmes du rendu avec le symbole sentinelle.
///
/// # Arguments
///
/// * `rendered_text` - Texte du rendu avec la sentinelle.
/// * `example` - Données d'exemple (pour compter les éléments).
/// * `sentinel` - Symbole sentinelle utilisé.
///
/// # Returns
///
/// Messages en français, vides si chaque élément porte le symbole des données.
pub fn list_marker_problems(rendered_text: &str, example: &Value, sentinel: &str) -> Vec<String> {
    let first_words = item_first_words(example);
    let shown = rendered_text.matches(sentinel).count();
    let mut problems = Vec::new();
    if shown != first_words.len() {
        problems.push(format!(
            "Rendu avec data.list_marker = « {sentinel} », le template affiche ce symbole {shown} fois pour {} \
             élément(s) de description (bullets, details). Chaque élément de description est précédé de \
             data.list_marker, une seule fois, jamais d'un symbole écrit en dur ; les séparateurs de l'en-tête et \
             des coordonnées restent écrits en dur et ne lisent jamais data.list_marker.",
            first_words.len()
        ));
        return problems;
    }
    let rendered = normalize_quotes(rendered_text);
    let rendered_words = words(&rendered);
    let unmarked: Vec<&String> =
        first_words.iter().filter(|word| !marker_before(&rendered_words, sentinel, word)).collect();
    if !unmarked.is_empty() {
        let quoted: Vec<String> = unmarked.iter().map(|w| format!("« {w} »")).collect();
        problems.push(format!(
            "Rendu avec data.list_marker = « {sentinel} », le symbole ne précède pas les éléments qui commencent \
             par {} : data.list_marker s'affiche juste avant chaque élément de bullets ou details.",
            quoted.join(", ")
        ));
    }
    problems
}

/// Vrai si un écart avec la référence ne porte que sur des symboles de
/// liste : symbole des données ajouté (section sans symbole dans la
/// référence), éventuellement à la place d'un autre symbole d'un caractère.
///
/// # Arguments
///
/// * `hunk` - Écart mot à mot entre la référence et le rendu d'exemple.
/// * `marker` - Symbole des données d'exemple (`list_marker`).
///
/// # Returns
///
/// Vrai si l'écart est toléré.
pub fn is_marker_hunk(hunk: &DiffHunk, marker: &str) -> bool {
    let is_symbol = |word: &String| {
        let mut chars = word.chars();
        matches!((chars.next(), chars.next()), (Some(c), None) if !c.is_alphanumeric())
    };
    !marker.is_empty()
        && !hunk.extra.is_empty()
        && hunk.extra.iter().all(|word| word == marker)
        && hunk.missing.iter().all(is_symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Value {
        serde_json::json!({
            "experiences": [{ "bullets": ["Pipeline de données", "Tests"] }],
            "projects": [{ "bullets": ["Application web"] }],
            "formations": [{ "details": ["Spécialité data"] }]
        })
    }

    #[test]
    fn sentinelle_absente_de_la_reference() {
        assert_eq!(pick_sentinel("Texte • avec puce"), Some("¤"));
        assert_eq!(pick_sentinel("¤ déjà là"), Some("¶"));
        assert_eq!(pick_sentinel("¤ ¶ §"), None);
    }

    #[test]
    fn un_symbole_par_element_accepte() {
        let text = "Nom • email ¤ Pipeline de données ¤ Tests ¤ Application web ¤ Spécialité data";
        assert!(list_marker_problems(text, &example(), "¤").is_empty());
        // Collé à l'élément : toujours devant lui.
        let glued = "¤Pipeline de données ¤ Tests ¤ Application web ¤ Spécialité data";
        assert!(list_marker_problems(glued, &example(), "¤").is_empty());
    }

    #[test]
    fn symbole_en_dur_refuse() {
        let text = "• Pipeline de données • Tests • Application web • Spécialité data";
        let problems = list_marker_problems(text, &example(), "¤");
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("0 fois pour 4"), "{problems:?}");
    }

    #[test]
    fn separateur_branche_sur_le_symbole_refuse() {
        let text = "Nom ¤ email ¤ Pipeline de données ¤ Tests ¤ Application web ¤ Spécialité data";
        assert!(list_marker_problems(text, &example(), "¤")[0].contains("5 fois pour 4"));
    }

    #[test]
    fn symbole_mal_place_refuse() {
        let text = "¤ ¤ ¤ ¤ Pipeline de données Tests Application web Spécialité data";
        let problems = list_marker_problems(text, &example(), "¤");
        // Seul le premier élément suit un symbole : les trois autres sont signalés.
        assert!(problems[0].contains("« Tests », « Application », « Spécialité »"), "{problems:?}");
        assert!(!problems[0].contains("« Pipeline »"), "{problems:?}");
    }

    #[test]
    fn ecart_de_symbole_tolere() {
        let hunk = |missing: &[&str], extra: &[&str]| DiffHunk {
            expected_start: 0,
            missing: missing.iter().map(|w| w.to_string()).collect(),
            extra: extra.iter().map(|w| w.to_string()).collect(),
        };
        assert!(is_marker_hunk(&hunk(&[], &["•"]), "•"), "symbole ajouté");
        assert!(is_marker_hunk(&hunk(&["▸"], &["•"]), "•"), "autre symbole remplacé");
        assert!(!is_marker_hunk(&hunk(&["•"], &[]), "•"), "symbole retiré : écart réel");
        assert!(!is_marker_hunk(&hunk(&["Texte"], &["•"]), "•"), "mot remplacé : écart réel");
        assert!(!is_marker_hunk(&hunk(&[], &["•"]), ""), "pas de symbole dans les données");
    }
}
