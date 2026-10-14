//! Tests des tables enfants du profil (`experiences`, `formations`,
//! `projets`) et de l'assemblage `db::profile::build_profile`.

// Importation des crates externes
use chrono::NaiveDate;

// Importation des modules internes
use crate::shared::seed;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{CreateExperienceInput, CreateFormationInput, CreateProjetInput};
use findmyjob_lib::db::{experiences, formations, profile, projets};

fn date(s: &str) -> Option<NaiveDate> {
    Some(NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap())
}

fn experience(poste: &str, debut: Option<NaiveDate>, fin: Option<NaiveDate>) -> CreateExperienceInput {
    CreateExperienceInput {
        poste: poste.into(),
        entreprise: Some("Acme".into()),
        lieu: None,
        description: None,
        date_debut: debut,
        date_fin: fin,
        type_contrat: Some("stage".into()),
    }
}

#[tokio::test]
async fn experiences_crud_et_tri_par_dates() {
    let pool = seed::pool().await;
    let ancienne = experiences::insert(&pool, experience("Ancienne", date("2024-01-01"), date("2024-06-30")), seed::NOW)
        .await
        .unwrap();
    experiences::insert(&pool, experience("En cours", date("2025-11-01"), None), seed::NOW).await.unwrap();
    experiences::insert(&pool, experience("Sans début", None, None), seed::NOW).await.unwrap();

    let liste = experiences::list_all(&pool).await.unwrap();
    let postes: Vec<&str> = liste.iter().map(|e| e.poste.as_str()).collect();
    // En cours d'abord ; à date de fin égale, date de début absente en tête
    // (NULLS FIRST, comme PostgreSQL en tri décroissant).
    assert_eq!(postes, ["Sans début", "En cours", "Ancienne"]);
    assert_eq!(liste[2].date_fin, date("2024-06-30"));

    let maj = experiences::update(&pool, ancienne.id, experience("Renommée", None, None)).await.unwrap();
    assert_eq!(maj.poste, "Renommée");
    assert!(matches!(
        experiences::update(&pool, 999, experience("X", None, None)).await,
        Err(AppError::NotFound(_))
    ));

    experiences::delete(&pool, ancienne.id).await.unwrap();
    assert!(matches!(experiences::delete(&pool, ancienne.id).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn experience_refuse_un_type_de_contrat_inconnu() {
    let pool = seed::pool().await;
    let mut input = experience("Poste", None, None);
    input.type_contrat = Some("benevolat".into());
    assert!(matches!(experiences::insert(&pool, input, seed::NOW).await, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn formations_et_projets_crud() {
    let pool = seed::pool().await;
    let formation = formations::insert(
        &pool,
        CreateFormationInput {
            titre: "Master".into(),
            ecole: Some("INSA".into()),
            description: None,
            date_debut: date("2022-01-01"),
            date_fin: date("2027-01-01"),
        },
        seed::NOW,
    )
    .await
    .unwrap();
    assert_eq!(formations::list_all(&pool).await.unwrap().len(), 1);
    formations::delete(&pool, formation.id).await.unwrap();
    assert!(formations::list_all(&pool).await.unwrap().is_empty());

    let projet = projets::insert(
        &pool,
        CreateProjetInput {
            titre: "FindMyJob".into(),
            description: None,
            stack: Some("Rust".into()),
            url: None,
            pour_qui: None,
            date_debut: None,
            date_fin: None,
        },
        seed::NOW,
    )
    .await
    .unwrap();
    assert_eq!(projet.stack.as_deref(), Some("Rust"));
    assert!(matches!(projets::delete(&pool, 999).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn assemble_le_profil_complet() {
    let pool = seed::pool().await;
    assert!(matches!(profile::build_profile(&pool).await, Err(AppError::NotFound(_))));

    seed::user(&pool).await;
    experiences::insert(&pool, experience("Poste", None, None), seed::NOW).await.unwrap();
    let profil = profile::build_profile(&pool).await.unwrap();
    assert_eq!(profil.user.id, 1);
    assert_eq!(profil.experiences.len(), 1);
    assert!(profil.skills.is_empty() && profil.langues.is_empty());
}
