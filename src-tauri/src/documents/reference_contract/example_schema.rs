//! Schéma des valeurs d'exemple d'une référence (`example` de la
//! métadonnée), identique aux données que l'application injecte dans
//! `sys.inputs.data` : `models::doc_data` sans libellés ni textes fixes.
//!
//! Source unique des champs du contrat : la validation de l'exemple, les
//! données dérivées des rendus de contrôle (vides, sentinelles) et le test
//! qui vérifie que chaque champ est documenté dans le contrat en dépendent.

// Importation des crates externes
use serde_json::{Map, Value};

// Importation des modules internes
use crate::docgen::sentinel_data::{MARKER_END, MARKER_START};
use crate::models::{ReferenceIssue, ReferenceIssueCode};

/// Profondeur maximale parcourue : le schéma a moins de cinq niveaux.
const MAX_DEPTH: usize = 8;
/// Champ jamais marqué ni vidé : il pilote la langue du rendu.
const LANG_FIELD: &str = "lang";

/// Forme attendue d'une valeur.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Chaîne (vide plutôt qu'absente).
    Str,
    /// Booléen.
    Bool,
    /// Liste d'éléments de même forme.
    List(Box<Shape>),
    /// Objet aux clés exactes, toutes requises, dans l'ordre d'affichage.
    Object(Vec<(&'static str, Shape)>),
}

/// Objet aux clés données.
fn object(fields: Vec<(&'static str, Shape)>) -> Shape {
    Shape::Object(fields)
}

/// Liste de chaînes.
fn strings() -> Shape {
    Shape::List(Box::new(Shape::Str))
}

/// Liste d'objets.
fn list_of(fields: Vec<(&'static str, Shape)>) -> Shape {
    Shape::List(Box::new(object(fields)))
}

/// Forme de `identity`.
fn identity() -> Shape {
    object(vec![
        ("prenom", Shape::Str),
        ("nom", Shape::Str),
        ("title", Shape::Str),
        ("email", Shape::Str),
        ("telephone", Shape::Str),
        ("ville", Shape::Str),
        ("zip_code", Shape::Str),
        ("linkedin", Shape::Str),
        ("github", Shape::Str),
        ("portfolio", Shape::Str),
        ("permis", Shape::Bool),
    ])
}

/// Forme des données d'un CV.
pub fn cv_shape() -> Shape {
    object(vec![
        ("lang", Shape::Str),
        ("identity", identity()),
        ("summary", Shape::Str),
        ("skills_groups", list_of(vec![("category", Shape::Str), ("skills", strings())])),
        (
            "experiences",
            list_of(vec![
                ("poste", Shape::Str),
                ("entreprise", Shape::Str),
                ("lieu", Shape::Str),
                ("period", Shape::Str),
                ("intro", Shape::Str),
                ("bullets", strings()),
            ]),
        ),
        (
            "projects",
            list_of(vec![
                ("titre", Shape::Str),
                ("pour_qui", Shape::Str),
                ("period", Shape::Str),
                ("stack", Shape::Str),
                ("url", Shape::Str),
                ("intro", Shape::Str),
                ("bullets", strings()),
            ]),
        ),
        (
            "formations",
            list_of(vec![
                ("titre", Shape::Str),
                ("ecole", Shape::Str),
                ("period", Shape::Str),
                ("intro", Shape::Str),
                ("details", strings()),
            ]),
        ),
        ("langues", list_of(vec![("nom", Shape::Str), ("level", Shape::Str)])),
        ("list_marker", Shape::Str),
    ])
}

/// Forme des données d'une lettre de motivation.
pub fn lm_shape() -> Shape {
    object(vec![
        ("lang", Shape::Str),
        ("identity", identity()),
        (
            "recipient",
            object(vec![("company", Shape::Str), ("location", Shape::Str), ("extra_lines", strings())]),
        ),
        ("date", Shape::Str),
        ("subject", Shape::Str),
        ("paragraphs", strings()),
    ])
}

/// Forme des données d'un type de document.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
pub fn shape_for(doc_type: &str) -> Shape {
    if doc_type == "cv" {
        cv_shape()
    } else {
        lm_shape()
    }
}

/// Chemins de tous les champs d'une forme (`identity.prenom`,
/// `experiences[].bullets`…), dans l'ordre du schéma.
///
/// # Arguments
///
/// * `shape` - Forme parcourue.
///
/// # Returns
///
/// Chemins des champs feuilles et des listes.
pub fn field_paths(shape: &Shape) -> Vec<String> {
    let mut paths = Vec::new();
    collect_paths(shape, "", &mut paths, 0);
    paths
}

/// Ajoute à `paths` les chemins de `shape` sous `prefix`.
fn collect_paths(shape: &Shape, prefix: &str, paths: &mut Vec<String>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    match shape {
        Shape::Object(fields) => {
            for (name, field) in fields {
                let path = if prefix.is_empty() { (*name).to_string() } else { format!("{prefix}.{name}") };
                collect_paths(field, &path, paths, depth + 1);
            }
        }
        Shape::List(item) if matches!(**item, Shape::Object(_)) => {
            paths.push(prefix.to_string());
            collect_paths(item, &format!("{prefix}[]"), paths, depth + 1);
        }
        _ => paths.push(prefix.to_string()),
    }
}

/// Vérifie qu'une valeur suit exactement une forme : chaque champ manquant,
/// en trop ou mal typé donne un problème `example_invalid` avec son chemin.
///
/// # Arguments
///
/// * `value` - Valeur lue dans la métadonnée.
/// * `shape` - Forme attendue.
/// * `path` - Chemin de la valeur (ex. `example`).
///
/// # Returns
///
/// Tous les problèmes (vide si conforme).
pub fn shape_issues(value: &Value, shape: &Shape, path: &str) -> Vec<ReferenceIssue> {
    let mut issues = Vec::new();
    check(value, shape, path, &mut issues, 0);
    issues
}

/// Problème `example_invalid` sur un chemin.
fn invalid(path: &str, detail: impl Into<String>) -> ReferenceIssue {
    ReferenceIssue::new(ReferenceIssueCode::ExampleInvalid).at_path(path).with_detail(detail)
}

/// Nom JSON du type d'une valeur, pour le détail d'un problème.
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "none",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "str",
        Value::Array(_) => "array",
        Value::Object(_) => "dictionary",
    }
}

/// Parcours de `shape_issues`.
fn check(value: &Value, shape: &Shape, path: &str, issues: &mut Vec<ReferenceIssue>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    match (shape, value) {
        (Shape::Str, Value::String(_)) | (Shape::Bool, Value::Bool(_)) => {}
        (Shape::Str, other) => issues.push(invalid(path, format!("expected str, found {}", type_name(other)))),
        (Shape::Bool, other) => issues.push(invalid(path, format!("expected bool, found {}", type_name(other)))),
        (Shape::List(item), Value::Array(items)) => {
            for (i, element) in items.iter().enumerate() {
                check(element, item, &format!("{path}[{i}]"), issues, depth + 1);
            }
        }
        (Shape::List(_), other) => issues.push(invalid(path, format!("expected array, found {}", type_name(other)))),
        (Shape::Object(fields), Value::Object(map)) => check_object(map, fields, path, issues, depth),
        (Shape::Object(_), other) => {
            issues.push(invalid(path, format!("expected dictionary, found {}", type_name(other))))
        }
    }
}

/// Champs manquants, en trop ou mal typés d'un objet.
fn check_object(
    map: &Map<String, Value>,
    fields: &[(&'static str, Shape)],
    path: &str,
    issues: &mut Vec<ReferenceIssue>,
    depth: usize,
) {
    for (name, field) in fields {
        let field_path = format!("{path}.{name}");
        match map.get(*name) {
            Some(value) => check(value, field, &field_path, issues, depth + 1),
            None => issues.push(invalid(&field_path, "missing field")),
        }
    }
    for key in map.keys().filter(|key| !fields.iter().any(|(name, _)| name == key)) {
        issues.push(invalid(&format!("{path}.{key}"), "unknown field"));
    }
}

/// Données vides d'une forme : chaînes vides, listes vides, booléens faux ;
/// la langue est conservée.
///
/// # Arguments
///
/// * `shape` - Forme des données.
/// * `lang` - Langue du rendu.
///
/// # Returns
///
/// Données vides au format JSON.
pub fn blank(shape: &Shape, lang: &str) -> Value {
    build(shape, &mut String::new, 0, false, lang)
}

/// Données sentinelles d'une forme : un élément par liste (listes imbriquées
/// comprises), booléens vrais, et un marqueur unique (`__F1__`, `__F2__`…)
/// dans chaque chaîne sauf la langue. Tout texte venu des données est alors
/// remplacé par un marqueur : ce qui reste du rendu est écrit en dur.
///
/// # Arguments
///
/// * `shape` - Forme des données.
/// * `lang` - Langue du rendu.
///
/// # Returns
///
/// Données sentinelles au format JSON.
pub fn sentinel(shape: &Shape, lang: &str) -> Value {
    let mut counter = 0;
    build(
        shape,
        &mut || {
            counter += 1;
            format!("{MARKER_START}{counter}{MARKER_END}")
        },
        0,
        true,
        lang,
    )
}

/// Construit des données d'une forme : chaînes produites par `text`, une
/// entrée par liste si `filled`, booléens à `filled`.
fn build(shape: &Shape, text: &mut dyn FnMut() -> String, depth: usize, filled: bool, lang: &str) -> Value {
    if depth > MAX_DEPTH {
        return Value::Null;
    }
    match shape {
        Shape::Str => Value::String(text()),
        Shape::Bool => Value::Bool(filled),
        Shape::List(item) if filled => Value::Array(vec![build(item, text, depth + 1, filled, lang)]),
        Shape::List(_) => Value::Array(Vec::new()),
        Shape::Object(fields) => {
            let mut map = Map::new();
            for (name, field) in fields {
                let value = if *name == LANG_FIELD {
                    Value::String(lang.to_string())
                } else {
                    build(field, text, depth + 1, filled, lang)
                };
                map.insert((*name).to_string(), value);
            }
            Value::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn lm_example() -> Value {
        json!({
            "lang": "fr",
            "identity": {
                "prenom": "A", "nom": "B", "title": "", "email": "", "telephone": "", "ville": "",
                "zip_code": "", "linkedin": "", "github": "", "portfolio": "", "permis": false
            },
            "recipient": { "company": "C", "location": "D", "extra_lines": [] },
            "date": "", "subject": "", "paragraphs": ["P"]
        })
    }

    #[test]
    fn exemple_conforme_accepte() {
        assert!(shape_issues(&lm_example(), &lm_shape(), "example").is_empty());
    }

    #[test]
    fn chaque_ecart_rapporte_avec_son_chemin() {
        let mut example = lm_example();
        example["identity"].as_object_mut().expect("objet").remove("email");
        example["recipient"]["extra_lines"] = json!("ligne");
        example["paragraphs"] = json!(["P", 3]);
        example["labels"] = json!({});
        let paths: Vec<String> =
            shape_issues(&example, &lm_shape(), "example").into_iter().filter_map(|i| i.path).collect();
        assert_eq!(
            paths,
            vec![
                "example.identity.email",
                "example.recipient.extra_lines",
                "example.paragraphs[1]",
                "example.labels",
            ]
        );
    }

    #[test]
    fn sentinelle_un_element_par_liste_et_langue_gardee() {
        let data = sentinel(&cv_shape(), "en");
        assert_eq!(data["lang"], "en");
        assert_eq!(data["experiences"].as_array().map(Vec::len), Some(1));
        assert_eq!(data["experiences"][0]["bullets"].as_array().map(Vec::len), Some(1));
        assert_eq!(data["identity"]["permis"], true);
        assert!(data["summary"].as_str().is_some_and(|s| s.starts_with(MARKER_START)));
        assert!(shape_issues(&data, &cv_shape(), "example").is_empty());
    }

    #[test]
    fn donnees_vides_conformes() {
        let data = blank(&cv_shape(), "fr");
        assert_eq!(data["lang"], "fr");
        assert_eq!(data["experiences"], json!([]));
        assert!(shape_issues(&data, &cv_shape(), "example").is_empty());
    }

    #[test]
    fn chemins_des_champs() {
        let paths = field_paths(&cv_shape());
        assert!(paths.contains(&"identity.permis".to_string()));
        assert!(paths.contains(&"experiences".to_string()));
        assert!(paths.contains(&"experiences[].bullets".to_string()));
        assert!(paths.contains(&"list_marker".to_string()));
    }
}
