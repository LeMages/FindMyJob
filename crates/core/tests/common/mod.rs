//! Fixtures anonymisées partagées par les tests du pipeline de génération
//! de documents : un profil fictif et des contenus de référence.

// Le module est inclus par plusieurs fichiers de test, qui n'utilisent
// chacun qu'une partie des fixtures.
#![allow(dead_code)]

// Importation des crates externes
use chrono::{NaiveDate, TimeZone, Utc};

// Importation des modules internes
use findmyjob_core::models::{
    CvData, CvExperience, CvProject, ExperienceView, LmData, MatchedItem, MatchingResult, ProfileDto, ProjetView,
    SkillView, SkillsGroup, UserView,
};

/// Date au format court, pour les fixtures.
pub fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date de fixture valide")
}

/// Utilisateur fictif.
pub fn user() -> UserView {
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().expect("date valide");
    UserView {
        id: 1,
        nom: Some("Martin".to_string()),
        prenom: Some("Camille".to_string()),
        email: Some("camille.martin@example.org".to_string()),
        telephone: Some("01 23 45 67 89".to_string()),
        ville: Some("Lyon".to_string()),
        adresse: None,
        code_postal: None,
        permis: Some(true),
        description: Some("Étudiante ingénieure en informatique.".to_string()),
        linkedin: Some("https://www.linkedin.com/in/camille-martin/".to_string()),
        github: None,
        portfolio: None,
        interets: Some("• Escalade\n\nPhotographie\n- Échecs".to_string()),
        created_at: now,
    }
}

/// Expérience fictive.
pub fn experience(id: i64, poste: &str, start: NaiveDate, end: Option<NaiveDate>) -> ExperienceView {
    ExperienceView {
        id,
        poste: poste.to_string(),
        entreprise: Some(format!("Entreprise {id}")),
        lieu: None,
        description: Some("A construit des tableaux de bord.".to_string()),
        date_debut: Some(start),
        date_fin: end,
        type_contrat: Some("stage".to_string()),
    }
}

/// Projet fictif.
pub fn projet(id: i64, titre: &str) -> ProjetView {
    ProjetView {
        id,
        titre: titre.to_string(),
        description: Some("Application de démonstration.".to_string()),
        stack: Some("Rust".to_string()),
        url: None,
        pour_qui: Some("École".to_string()),
        date_debut: Some(date(2024, 9, 1)),
        date_fin: Some(date(2025, 6, 30)),
    }
}

/// Profil fictif : 2 expériences, 4 projets (le plus récent en premier).
pub fn profile() -> ProfileDto {
    ProfileDto {
        user: user(),
        experiences: vec![
            experience(10, "Stage data", date(2025, 11, 1), Some(date(2026, 4, 30))),
            experience(11, "Stage web", date(2024, 6, 1), Some(date(2024, 8, 31))),
        ],
        formations: Vec::new(),
        projets: vec![projet(20, "Projet A"), projet(21, "Projet B"), projet(22, "Projet C"), projet(23, "Projet D")],
        skills: vec![
            SkillView { id: 1, domaine: "Langages".to_string(), nom: "Rust".to_string() },
            SkillView { id: 2, domaine: "Langages".to_string(), nom: "Python".to_string() },
            SkillView { id: 3, domaine: "Data".to_string(), nom: "SQL".to_string() },
        ],
        langues: Vec::new(),
    }
}

/// Entrée de matching.
pub fn matched(id: i64) -> MatchedItem {
    MatchedItem { id, relevance: "high".to_string(), highlights: vec!["fait".to_string()], keywords_covered: Vec::new() }
}

/// Matching fictif : désigne les projets 22 puis 20.
pub fn matching() -> MatchingResult {
    MatchingResult {
        relevant_experiences: vec![matched(10), matched(11)],
        relevant_projects: vec![matched(22), matched(20)],
        relevant_skills: Vec::new(),
        summary_strategy: "Angle data.".to_string(),
        gaps: Vec::new(),
    }
}

/// Contenu de référence d'un CV : résumé de 10 mots, 1 expérience à 3
/// bullets, 2 projets à 2 et 1 bullets, 2 groupes de compétences.
pub fn cv_example() -> CvData {
    let bullet = |n: usize| vec!["mot"; n].join(" ");
    CvData {
        summary: bullet(10),
        experiences: vec![CvExperience {
            poste: "Poste".to_string(),
            period: "Nov. 2025 – Avr. 2026".to_string(),
            bullets: vec![bullet(10), bullet(20), bullet(30)],
            ..Default::default()
        }],
        projects: vec![
            CvProject { titre: "P1".to_string(), bullets: vec![bullet(10), bullet(10)], ..Default::default() },
            CvProject { titre: "P2".to_string(), bullets: vec![bullet(10)], ..Default::default() },
        ],
        skills_groups: vec![
            SkillsGroup { category: "A".to_string(), skills: vec!["x".to_string()] },
            SkillsGroup { category: "B".to_string(), skills: vec!["y".to_string()] },
        ],
        ..Default::default()
    }
}

/// Contenu de référence d'une lettre : objet de 6 mots, 3 paragraphes de
/// 20, 50 et 10 mots.
pub fn lm_example() -> LmData {
    let words = |n: usize| vec!["mot"; n].join(" ");
    LmData { subject: words(6), paragraphs: vec![words(20), words(50), words(10)], ..Default::default() }
}
