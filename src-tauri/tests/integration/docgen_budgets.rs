//! Tests des budgets de longueur (`findmyjob_lib::docgen::budgets`).

// Importation des modules internes
use crate::common;
use findmyjob_lib::docgen::budgets::{cv_budget, lm_budget, WordRange};
use findmyjob_lib::docgen::words::count_words;

#[test]
fn plage_de_mots_a_dix_pour_cent() {
    let range = WordRange::around(100);
    assert_eq!((range.min, range.target, range.max), (90, 100, 110));
    assert!(range.contains(90) && range.contains(110));
    assert!(!range.contains(89) && !range.contains(111));
}

#[test]
fn comptage_de_mots_ignore_la_ponctuation_isolee() {
    assert_eq!(count_words("Objet : candidature au poste"), 4);
    assert_eq!(count_words("  "), 0);
}

#[test]
fn budget_cv_reprend_les_nombres_d_elements_de_l_exemple() {
    let budget = cv_budget(&common::cv_example(), 1);
    assert_eq!(budget.page_count, 1);
    assert_eq!(budget.experience_bullets, vec![3]);
    assert_eq!(budget.project_bullets, vec![2, 1]);
    assert_eq!(budget.project_count, 2);
    assert_eq!(budget.skills_group_count, 2);
    assert_eq!(budget.summary_words, WordRange::around(10));
}

#[test]
fn budget_cv_bullets_entre_le_plus_court_et_le_plus_long() {
    // Bullets de l'exemple : 10, 20, 30 (expérience) puis 10, 10, 10 (projets).
    let budget = cv_budget(&common::cv_example(), 1);
    assert_eq!(budget.bullet_words.min, 9);
    assert_eq!(budget.bullet_words.max, 33);
    assert_eq!(budget.bullet_words.target, 15);
}

#[test]
fn budget_cv_rang_au_dela_de_l_exemple_reprend_le_dernier() {
    let budget = cv_budget(&common::cv_example(), 1);
    assert_eq!(budget.experience_bullets_at(0), 3);
    assert_eq!(budget.experience_bullets_at(5), 3);
    assert_eq!(budget.project_bullets_at(1), 1);
    assert_eq!(budget.project_bullets_at(4), 1);
}

#[test]
fn budget_cv_reduit_garde_la_structure() {
    let budget = cv_budget(&common::cv_example(), 1).scaled(0.5);
    assert_eq!(budget.project_count, 2);
    assert_eq!(budget.skills_group_count, 2);
    assert_eq!(budget.experience_bullets, vec![2]);
    assert_eq!(budget.project_bullets, vec![1, 1], "jamais moins d'un bullet");
    assert_eq!(budget.summary_words.target, 5);
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
