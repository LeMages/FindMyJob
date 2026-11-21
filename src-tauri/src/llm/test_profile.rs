//! Profils fictifs des tests unitaires de `llm` qui portent sur le contexte
//! détaillé (DTO de prompt, bloc de la lettre, messages de candidature).

// Importation des crates externes
use chrono::NaiveDate;

// Importation des modules internes
use crate::models::{ExperienceView, FormationView, LangView, ProfileDto, ProjetView, SkillView, UserView};

/// Date fixe.
fn date(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d)
}

/// Expérience fictive, champs renseignés, avec le contexte donné.
pub fn experience(id: i64, contexte: Option<&str>) -> ExperienceView {
    ExperienceView {
        id,
        poste: format!("Poste {id}"),
        entreprise: Some("Lumen Data".to_string()),
        lieu: None,
        description: Some("Pipelines SQL pour le reporting.".to_string()),
        date_debut: date(2024, 9, 1),
        date_fin: None,
        type_contrat: Some("alternance".to_string()),
        contexte: contexte.map(str::to_string),
    }
}

/// Projet fictif, champs renseignés, avec le contexte donné.
pub fn projet(id: i64, contexte: Option<&str>) -> ProjetView {
    ProjetView {
        id,
        titre: format!("Projet {id}"),
        description: Some("Suivi des adhésions en Python.".to_string()),
        stack: Some("Python".to_string()),
        url: None,
        pour_qui: Some("Association".to_string()),
        date_debut: date(2023, 1, 1),
        date_fin: date(2023, 6, 30),
        contexte: contexte.map(str::to_string),
    }
}

/// Profil fictif complet avec les expériences et les projets donnés.
pub fn profile_with(experiences: Vec<ExperienceView>, projets: Vec<ProjetView>) -> ProfileDto {
    ProfileDto {
        user: UserView {
            id: 1,
            nom: Some("Martin".to_string()),
            prenom: Some("Alex".to_string()),
            email: Some("alex@example.com".to_string()),
            telephone: None,
            permis: Some(true),
            description: Some("Étudiant en data.".to_string()),
            linkedin: None,
            github: None,
            portfolio: None,
            langue_profil: "fr".to_string(),
            created_at: chrono::Utc::now(),
        },
        experiences,
        formations: vec![FormationView {
            id: 1,
            titre: "Master informatique".to_string(),
            ecole: Some("Université Lyon 1".to_string()),
            description: None,
            date_debut: date(2023, 9, 1),
            date_fin: date(2025, 6, 30),
        }],
        projets,
        skills: vec![SkillView { id: 1, domaine: "Données".to_string(), nom: "SQL".to_string() }],
        langues: vec![LangView { id: 1, nom: Some("Anglais".to_string()), level: Some("B2".to_string()) }],
    }
}
