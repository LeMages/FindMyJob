//! Tests des budgets de longueur (`findmyjob_lib::docgen::budgets`).

// Importation des modules internes
use crate::common;
use findmyjob_lib::docgen::budgets::{cv_budget, lm_budget, WordRange};
use findmyjob_lib::docgen::words::count_words;

#[test]
fn plage_de_mots_asymetrique() {
    // Trop long : limite stricte à +10 %. Trop court : souple, jusqu'à -30 %.
    let range = WordRange::around(100);
    assert_eq!((range.min, range.target, range.max), (70, 100, 110));
    assert!(range.contains(70) && range.contains(110));
    assert!(!range.contains(69) && !range.contains(111));
}

#[test]
fn comptage_de_mots_ignore_la_ponctuation_isolee() {
    assert_eq!(count_words("Objet : candidature au poste"), 4);
    assert_eq!(count_words("  "), 0);
}

#[test]
fn budget_cv_limite_a_la_prose() {
    let budget = cv_budget(&common::cv_example(), 1);
    assert_eq!(budget.page_count, 1);
    assert_eq!(budget.skills_group_count, 2);
    assert_eq!(budget.summary_words, WordRange::around(10));
    let json = serde_json::to_value(&budget).expect("budget sérialisable");
    for removed in ["experience_bullets", "project_bullets", "bullet_words", "project_count"] {
        assert!(json.get(removed).is_none(), "{removed} : les entrées du profil n'ont pas de budget");
    }
}

#[test]
fn budget_cv_reduit_seulement_le_resume() {
    let budget = cv_budget(&common::cv_example(), 1).scaled(0.5);
    assert_eq!(budget.skills_group_count, 2);
    assert_eq!(budget.summary_words.target, 5);
    assert_eq!(budget.page_count, 1);
}

#[test]
fn budget_lettre_par_paragraphe() {
    let budget = lm_budget(&common::lm_example(), 1);
    assert_eq!(budget.paragraph_count(), 3);
    assert_eq!(budget.paragraph_words[1], WordRange::around(50));
    assert_eq!(budget.subject_words, WordRange::around(6));
    let reduced = budget.scaled(0.85);
    assert_eq!(reduced.paragraph_count(), 3);
    assert!(reduced.paragraph_words[1].max < budget.paragraph_words[1].max);
}
