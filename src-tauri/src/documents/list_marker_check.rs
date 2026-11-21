//! Contrôle du symbole des listes d'un template de CV : le template doit
//! placer `data.list_marker` devant chaque élément de description
//! (`bullets`, `details`), et nulle part ailleurs. Le template est rendu avec
//! les données d'exemple dont le symbole est remplacé par un caractère
//! sentinelle absent de la référence : ce caractère doit apparaître
//! exactement une fois par élément, juste avant lui. Un symbole écrit en dur
//! (« • ») ne suivrait pas le format de la session ; un séparateur de
//! l'en-tête branché sur `list_marker` le suivrait à tort.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::docgen::rendered_words::{normalize_quotes, word_key, words};
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

/// Écart entre le symbole de liste rendu et les éléments de description.
///
/// # Attributes
///
/// * `shown` - Nombre d'occurrences du symbole sentinelle dans le rendu.
/// * `expected` - Nombre d'éléments de description (`bullets`, `details`).
/// * `unmarked` - Premiers mots des éléments que le symbole ne précède pas
///   (seulement quand les nombres coïncident).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListMarkerMismatch {
    pub shown: usize,
    pub expected: usize,
    pub unmarked: Vec<String>,
}

/// Contrôle du rendu avec le symbole sentinelle : exactement une
/// occurrence par élément de description, juste devant lui.
///
/// # Arguments
///
/// * `rendered_text` - Texte du rendu avec la sentinelle.
/// * `example` - Données d'exemple (pour compter les éléments).
/// * `sentinel` - Symbole sentinelle utilisé.
///
/// # Returns
///
/// `None` si chaque élément porte le symbole des données, l'écart sinon.
pub fn list_marker_mismatch(rendered_text: &str, example: &Value, sentinel: &str) -> Option<ListMarkerMismatch> {
    let first_words = item_first_words(example);
    let shown = rendered_text.matches(sentinel).count();
    if shown != first_words.len() {
        return Some(ListMarkerMismatch { shown, expected: first_words.len(), unmarked: Vec::new() });
    }
    let rendered = normalize_quotes(rendered_text);
    let rendered_words = words(&rendered);
    let unmarked: Vec<String> =
        first_words.iter().filter(|word| !marker_before(&rendered_words, sentinel, word)).cloned().collect();
    (!unmarked.is_empty()).then_some(ListMarkerMismatch { shown, expected: first_words.len(), unmarked })
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
        assert_eq!(list_marker_mismatch(text, &example(), "¤"), None);
        // Collé à l'élément : toujours devant lui.
        let glued = "¤Pipeline de données ¤ Tests ¤ Application web ¤ Spécialité data";
        assert_eq!(list_marker_mismatch(glued, &example(), "¤"), None);
    }

    #[test]
    fn symbole_en_dur_refuse() {
        let text = "• Pipeline de données • Tests • Application web • Spécialité data";
        let mismatch = list_marker_mismatch(text, &example(), "¤").expect("écart attendu");
        assert_eq!((mismatch.shown, mismatch.expected), (0, 4));
    }

    #[test]
    fn separateur_branche_sur_le_symbole_refuse() {
        let text = "Nom ¤ email ¤ Pipeline de données ¤ Tests ¤ Application web ¤ Spécialité data";
        let mismatch = list_marker_mismatch(text, &example(), "¤").expect("écart attendu");
        assert_eq!((mismatch.shown, mismatch.expected), (5, 4));
    }

    #[test]
    fn symbole_mal_place_refuse() {
        let text = "¤ ¤ ¤ ¤ Pipeline de données Tests Application web Spécialité data";
        let mismatch = list_marker_mismatch(text, &example(), "¤").expect("écart attendu");
        // Seul le premier élément suit un symbole : les trois autres sont signalés.
        assert_eq!(mismatch.unmarked, ["Tests", "Application", "Spécialité"]);
    }
}
