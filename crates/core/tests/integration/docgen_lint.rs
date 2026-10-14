//! Tests du linter de la prose générée (`findmyjob_core::docgen::lint`).

// Importation des modules internes
use crate::common;
use findmyjob_core::docgen::budgets::{cv_budget, lm_budget};
use findmyjob_core::docgen::lint::{lint_cv_budget, lint_lm_budget, lint_prose, strip_dashes};
use findmyjob_core::models::{CvContent, ItemBullets, LmContent, ProseField, SkillsGroup};

/// Champ de prose de test.
fn field(path: &str, text: &str) -> ProseField {
    ProseField { path: path.to_string(), text: text.to_string() }
}

/// Texte de `n` mots.
fn words(n: usize) -> String {
    vec!["mot"; n].join(" ")
}

#[test]
fn caracteres_interdits_releves_par_champ() {
    let fields = [
        field("a", "Conçu un pipeline — en production"),
        field("b", "Période 2024 – 2025"),
        field("c", "Un **gras** et un `code`"),
        field("d", "Texte propre, sans marque."),
    ];
    let violations = lint_prose(&fields, "");
    let paths: Vec<&str> = violations.iter().map(|v| v.field.as_str()).collect();
    assert_eq!(paths, vec!["a", "b", "c", "c"]);
}

#[test]
fn formule_interdite_sauf_si_presente_dans_l_exemple() {
    let fields = [field("p", "Une stack directement transférable au poste.")];
    assert_eq!(lint_prose(&fields, "").len(), 1);
    assert!(lint_prose(&fields, "Mon expérience est Directement Transférable.").is_empty());
}

#[test]
fn formule_anglaise_detectee() {
    let fields = [field("p", "This experience aligns with your needs.")];
    assert_eq!(lint_prose(&fields, "").len(), 1);
}

#[test]
fn budget_cv_respecte_sans_violation() {
    let budget = cv_budget(&common::cv_example(), 1);
    let content = CvContent {
        title: "Titre".to_string(),
        summary: words(10),
        experiences: vec![ItemBullets { id: 1, bullets: vec![words(12), words(12), words(12)] }],
        projects: vec![
            ItemBullets { id: 2, bullets: vec![words(12), words(12)] },
            ItemBullets { id: 3, bullets: vec![words(12)] },
        ],
        skills_groups: vec![
            SkillsGroup { category: "A".to_string(), skills: vec!["Rust".to_string()] },
            SkillsGroup { category: "B".to_string(), skills: vec!["SQL".to_string()] },
        ],
    };
    assert!(lint_cv_budget(&content, &budget).is_empty(), "{:?}", lint_cv_budget(&content, &budget));
}

#[test]
fn budget_cv_depasse_releve_chaque_ecart() {
    let budget = cv_budget(&common::cv_example(), 1);
    let content = CvContent {
        title: "Titre".to_string(),
        summary: words(30),
        experiences: vec![ItemBullets { id: 1, bullets: vec![words(50)] }],
        projects: vec![ItemBullets { id: 2, bullets: vec![words(5), words(5)] }],
        skills_groups: Vec::new(),
    };
    let fields: Vec<String> = lint_cv_budget(&content, &budget).into_iter().map(|v| v.field).collect();
    assert!(fields.contains(&"summary".to_string()));
    assert!(fields.contains(&"projects".to_string()));
    assert!(fields.contains(&"skills_groups".to_string()));
    assert!(fields.contains(&"experiences[0].bullets".to_string()));
    assert!(fields.contains(&"experiences[0].bullets[0]".to_string()));
}

#[test]
fn budget_lettre_nombre_de_paragraphes_et_mots() {
    let budget = lm_budget(&common::lm_example(), 1);
    let ok = LmContent { subject: words(6), paragraphs: vec![words(20), words(50), words(10)] };
    assert!(lint_lm_budget(&ok, &budget).is_empty());

    let too_long = LmContent { subject: words(6), paragraphs: vec![words(20), words(80)] };
    let fields: Vec<String> = lint_lm_budget(&too_long, &budget).into_iter().map(|v| v.field).collect();
    assert_eq!(fields, vec!["paragraphs".to_string(), "paragraphs[1]".to_string()]);
}

#[test]
fn remplacement_mecanique_des_tirets() {
    assert_eq!(strip_dashes("Un pipeline — en production — robuste"), "Un pipeline, en production, robuste");
    assert_eq!(strip_dashes("Data–ML et **gras** `x`"), "Data-ML et gras x");
    assert!(lint_prose(&[field("x", &strip_dashes("a — b – c"))], "").is_empty());
}
