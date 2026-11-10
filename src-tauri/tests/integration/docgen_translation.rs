//! Tests des textes d'entrées du CV et de la validation de leur traduction
//! (`findmyjob_lib::docgen::entry_texts`, `findmyjob_lib::docgen::translation_check`).

// Importation des modules internes
use crate::common;
use findmyjob_lib::docgen::entry_texts::{source_texts, CvEntryTexts, EntryLines, LanguageTexts, ProjectTexts};
use findmyjob_lib::docgen::selection::select;
use findmyjob_lib::docgen::translation_check::{max_translated_chars, translation_problems};

/// Textes d'origine : une expérience à deux lignes, un projet, une formation.
fn original() -> CvEntryTexts {
    CvEntryTexts {
        experiences: vec![EntryLines {
            id: 10,
            lines: vec!["Conception d'un pipeline de données pour le reporting hebdomadaire".to_string(), "Mise en production".to_string()],
        }],
        projects: vec![ProjectTexts {
            id: 20,
            pour_qui: "Mairie".to_string(),
            lines: vec!["Tableau de bord des adhésions".to_string()],
        }],
        formations: vec![EntryLines { id: 30, lines: vec!["Spécialité données".to_string()] }],
        langues: vec![LanguageTexts { id: 40, nom: "Anglais".to_string(), level: "Courant (TOEIC 905)".to_string() }],
        interets: vec!["Escalade".to_string()],
    }
}

/// Traduction valide de `original`.
fn translated() -> CvEntryTexts {
    CvEntryTexts {
        experiences: vec![EntryLines {
            id: 10,
            lines: vec!["Designed a data pipeline for weekly reporting".to_string(), "Deployed to production".to_string()],
        }],
        projects: vec![ProjectTexts {
            id: 20,
            pour_qui: "Town hall".to_string(),
            lines: vec!["Membership dashboard".to_string()],
        }],
        formations: vec![EntryLines { id: 30, lines: vec!["Data major".to_string()] }],
        langues: vec![LanguageTexts { id: 40, nom: "English".to_string(), level: "Fluent (TOEIC 905)".to_string() }],
        interets: vec!["Climbing".to_string()],
    }
}

#[test]
fn textes_source_lignes_exactes_du_profil() {
    let profile = common::profile();
    let texts = source_texts(&profile, &select(&profile, &[20]));
    assert_eq!(texts.experiences.len(), 2, "toutes les expériences");
    assert_eq!(texts.projects.len(), 1, "seulement les projets retenus");
    assert_eq!(texts.projects[0].lines, vec!["Application de démonstration.".to_string()]);
    assert_eq!(texts.projects[0].pour_qui, "École");
}

#[test]
fn longueur_maximale_avec_marge_pour_les_textes_courts() {
    assert_eq!(max_translated_chars(200), 220, "110 % sur un texte long");
    assert_eq!(max_translated_chars(6), 16, "marge de 10 caractères sur un texte court");
    assert_eq!(max_translated_chars(100), 110);
}

#[test]
fn traduction_valide_sans_probleme() {
    assert!(translation_problems(&original(), &translated()).is_empty(), "{:?}", translation_problems(&original(), &translated()));
}

#[test]
fn nombre_de_lignes_different_signale() {
    let mut bad = translated();
    bad.experiences[0].lines.pop();
    let problems = translation_problems(&original(), &bad);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("contient 1 lignes, attendu exactement 2"), "{problems:?}");
}

#[test]
fn ligne_trop_longue_signalee_plus_courte_acceptee() {
    let mut long = translated();
    let original_chars = original().experiences[0].lines[0].chars().count();
    long.experiences[0].lines[0] = "x".repeat(max_translated_chars(original_chars) + 1);
    let problems = translation_problems(&original(), &long);
    assert!(problems.iter().any(|p| p.contains("experiences id 10.lines[0]") && p.contains("caractères")), "{problems:?}");

    let mut short = translated();
    short.formations[0].lines[0] = "Data".to_string();
    assert!(translation_problems(&original(), &short).is_empty());
}

#[test]
fn ligne_vide_ou_multiligne_signalee() {
    let mut bad = translated();
    bad.projects[0].lines[0] = " ".to_string();
    bad.formations[0].lines[0] = "Data\nmajor".to_string();
    let problems = translation_problems(&original(), &bad);
    assert!(problems.iter().any(|p| p.contains("projects id 20.lines[0] : la traduction est vide")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("sans saut de ligne")), "{problems:?}");
}

#[test]
fn pour_qui_soumis_a_la_meme_regle() {
    let mut long = translated();
    long.projects[0].pour_qui = "The town hall of a small city in the Alps".to_string();
    let problems = translation_problems(&original(), &long);
    assert!(problems.iter().any(|p| p.contains("pour_qui")), "{problems:?}");

    let mut added = translated();
    let mut source = original();
    source.projects[0].pour_qui = String::new();
    added.projects[0].pour_qui = "School".to_string();
    let problems = translation_problems(&source, &added);
    assert!(problems.iter().any(|p| p.contains("l'original est vide")), "{problems:?}");
}

#[test]
fn identifiants_inconnus_manquants_ou_en_double() {
    let mut bad = translated();
    bad.projects[0].id = 99;
    bad.formations.push(bad.formations[0].clone());
    let problems = translation_problems(&original(), &bad);
    assert!(problems.iter().any(|p| p.contains("projects : l'id 99 n'est pas une entrée")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("projects : l'id 20 manque")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("formations : l'id 30 apparaît plusieurs fois")), "{problems:?}");
}

#[test]
fn langues_et_interets_traduits_valides() {
    let mut bad = translated();
    bad.langues[0].level = String::new();
    bad.interets.push("Chess".to_string());
    let problems = translation_problems(&original(), &bad);
    assert!(problems.iter().any(|p| p.contains("langues id 40.level")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("interets contient 2")), "{problems:?}");
}

#[test]
fn textes_source_avec_langues_et_interets_du_profil() {
    let mut profile = common::profile();
    profile.user.interets = Some("Escalade\nPhotographie".to_string());
    let texts = source_texts(&profile, &select(&profile, &[]));
    assert_eq!(texts.interets, ["Escalade", "Photographie"]);
}
