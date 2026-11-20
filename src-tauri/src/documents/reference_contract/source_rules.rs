//! Règles du fichier d'une référence, vérifiées avant toute compilation :
//! extension, taille, encodage, et constructions interdites, repérées par
//! analyse syntaxique (`typst-syntax`) plutôt que par recherche de
//! sous-chaîne (une chaîne ou un commentaire qui les cite reste permis).
//!
//! Interdits : `#import` / `#include` (paquets `@preview` compris),
//! `read(`, `eval(`, `plugin(`, `datetime.today`, et toute lecture d'un
//! fichier local (`image("photo.png")`, `json("data.json")`…). Une icône
//! SVG collée dans le fichier (`image(bytes("<svg…>"))`) reste permise.

// Importation des crates externes
use typst_syntax::ast::{Arg, Expr, FieldAccess, FuncCall};
use typst_syntax::{LinkedNode, SyntaxKind};

// Importation des modules internes
use crate::models::{ReferenceIssue, ReferenceIssueCode};

/// Taille maximale d'une référence, en octets : un CV complet avec ses
/// icônes SVG en fait quelques dizaines de kilo-octets.
pub const MAX_REFERENCE_BYTES: usize = 512 * 1024;
/// Fonctions toujours interdites : elles évaluent du code ou lisent un fichier.
const ALWAYS_FORBIDDEN: [&str; 3] = ["read", "eval", "plugin"];
/// Fonctions interdites quand elles reçoivent un chemin (chaîne littérale) :
/// le fichier n'existe pas pour l'application, seul le `.typ` est importé.
const FORBIDDEN_WITH_PATH: [&str; 7] = ["image", "json", "csv", "yaml", "toml", "xml", "cbor"];

/// Problèmes du fichier lui-même : extension, taille et encodage.
///
/// # Arguments
///
/// * `filename` - Nom du fichier importé.
/// * `content` - Contenu lu par l'interface (les octets non UTF-8 y sont
///   remplacés par U+FFFD).
///
/// # Returns
///
/// Problèmes bloquants (vide si le fichier est recevable).
pub fn file_issues(filename: &str, content: &str) -> Vec<ReferenceIssue> {
    let mut issues = Vec::new();
    if !filename.to_lowercase().ends_with(".typ") {
        issues.push(ReferenceIssue::new(ReferenceIssueCode::NotTyp).with_detail(filename));
    }
    if content.len() > MAX_REFERENCE_BYTES {
        issues.push(
            ReferenceIssue::new(ReferenceIssueCode::TooLarge)
                .with_detail(format!("{} > {MAX_REFERENCE_BYTES}", content.len())),
        );
    }
    if let Some(offset) = content.find('\u{FFFD}') {
        issues.push(ReferenceIssue::new(ReferenceIssueCode::NotUtf8).at_line(Some(line_of(content, offset))));
    }
    issues
}

/// Constructions interdites de la source, chacune avec sa ligne.
///
/// # Arguments
///
/// * `content` - Source Typst.
///
/// # Returns
///
/// Un problème `forbidden_construct` par construction trouvée.
pub fn forbidden_constructs(content: &str) -> Vec<ReferenceIssue> {
    let root = typst_syntax::parse(content);
    let mut issues = Vec::new();
    // Parcours itératif : la profondeur de l'arbre ne dépend que de la source.
    let mut stack = vec![LinkedNode::new(&root)];
    while let Some(node) = stack.pop() {
        if let Some(construct) = forbidden(&node) {
            issues.push(
                ReferenceIssue::new(ReferenceIssueCode::ForbiddenConstruct)
                    .at_line(Some(line_of(content, node.offset())))
                    .with_detail(construct),
            );
        }
        stack.extend(node.children());
    }
    issues.sort_by_key(|issue| issue.line);
    issues
}

/// Construction interdite portée par un nœud, s'il y en a une.
fn forbidden(node: &LinkedNode<'_>) -> Option<String> {
    match node.kind() {
        SyntaxKind::ModuleImport => Some("#import".to_string()),
        SyntaxKind::ModuleInclude => Some("#include".to_string()),
        SyntaxKind::FuncCall => node.get().cast::<FuncCall>().and_then(forbidden_call),
        SyntaxKind::FieldAccess => node.get().cast::<FieldAccess>().and_then(forbidden_field),
        _ => None,
    }
}

/// Appel interdit : fonction qui évalue ou lit un fichier.
fn forbidden_call(call: FuncCall<'_>) -> Option<String> {
    let Expr::Ident(callee) = call.callee() else {
        return None;
    };
    let name = callee.as_str();
    if ALWAYS_FORBIDDEN.contains(&name) {
        return Some(format!("{name}("));
    }
    if !FORBIDDEN_WITH_PATH.contains(&name) {
        return None;
    }
    let first = call.args().items().find_map(|arg| match arg {
        Arg::Pos(expr) => Some(expr),
        _ => None,
    })?;
    match first {
        Expr::Str(path) => Some(format!("{name}(\"{}\")", path.get())),
        _ => None,
    }
}

/// Accès interdit : `datetime.today`, qui rendrait le document dépendant
/// du jour de compilation (la date vient de `data.date`).
fn forbidden_field(access: FieldAccess<'_>) -> Option<String> {
    let Expr::Ident(target) = access.target() else {
        return None;
    };
    (target.as_str() == "datetime" && access.field().as_str() == "today").then(|| "datetime.today".to_string())
}

/// Ligne (à partir de 1) d'une position dans la source.
fn line_of(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())].matches('\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn details(content: &str) -> Vec<(Option<usize>, String)> {
        forbidden_constructs(content).into_iter().map(|i| (i.line, i.detail.unwrap_or_default())).collect()
    }

    #[test]
    fn constructions_interdites_situees() {
        let source = "#import \"@preview/x:0.1.0\": *\n#let a = read(\"f.txt\")\n#image(\"photo.png\")\n\
                      #let d = datetime.today()\n#include \"b.typ\"\n#eval(\"1\")";
        assert_eq!(
            details(source),
            vec![
                (Some(1), "#import".to_string()),
                (Some(2), "read(".to_string()),
                (Some(3), "image(\"photo.png\")".to_string()),
                (Some(4), "datetime.today".to_string()),
                (Some(5), "#include".to_string()),
                (Some(6), "eval(".to_string()),
            ]
        );
    }

    #[test]
    fn svg_colle_et_json_des_entrees_permis() {
        let source = "#let data = json(bytes(sys.inputs.data))\n#image(bytes(\"<svg></svg>\"))\n\
                      // read(\"x\") et #import dans un commentaire\n#let t = \"datetime.today, eval(\"";
        assert!(details(source).is_empty(), "{:?}", details(source));
    }

    #[test]
    fn fichier_non_recevable() {
        let codes: Vec<ReferenceIssueCode> =
            file_issues("cv.pdf", "a\u{FFFD}b").into_iter().map(|i| i.code).collect();
        assert_eq!(codes, vec![ReferenceIssueCode::NotTyp, ReferenceIssueCode::NotUtf8]);
        let big = "x".repeat(MAX_REFERENCE_BYTES + 1);
        assert_eq!(file_issues("cv.typ", &big)[0].code, ReferenceIssueCode::TooLarge);
        assert!(file_issues("CV.TYP", "#set page()").is_empty());
    }
}
