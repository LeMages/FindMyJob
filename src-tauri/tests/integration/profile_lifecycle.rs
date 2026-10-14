//! Cycle de vie du profil tel que la génération le lit : chaque élément est
//! ajouté, modifié et supprimé par les fonctions appelées par les commandes,
//! et l'état est relu après chaque opération par `db::profile::build_profile`.
//! Complète `db_users`, `db_profile_entries` et `db_skills_langues` (tri,
//! doublons, validations) sans les répéter.

// Importation des crates externes
use chrono::NaiveDate;
use sqlx::SqlitePool;

// Importation des modules internes
use crate::shared::seed;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{
    CreateExperienceInput, CreateFormationInput, CreateLangInput, CreateProjetInput, CreateSkillInput, ProfileDto,
    UpdateUserInfoInput,
};
use findmyjob_lib::db::{experiences, formations, langues, profile, projets, skills, users};

/// Base avec l'utilisateur unique.
async fn pool_with_user() -> SqlitePool {
    let pool = seed::pool().await;
    seed::user(&pool).await;
    pool
}

/// Relit le profil complet.
async fn read(pool: &SqlitePool) -> ProfileDto {
    profile::build_profile(pool).await.expect("profil")
}

/// Date fixe.
fn date(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d)
}

/// Mise à jour des infos sans aucun champ renseigné.
fn no_change() -> UpdateUserInfoInput {
    UpdateUserInfoInput {
        prenom: None,
        nom: None,
        email: None,
        telephone: None,
        ville: None,
        zip_code: None,
        permis: None,
        linkedin: None,
        github: None,
        portfolio: None,
        interets: None,
    }
}

#[tokio::test]
async fn identite_et_lieu() {
    let pool = pool_with_user().await;
    assert_eq!(read(&pool).await.user.ville.as_deref(), Some("Lyon"));

    // Modification : identité, coordonnées, ville et code postal.
    let input = UpdateUserInfoInput {
        prenom: Some("Alexandra".into()),
        email: Some("alexandra@example.com".into()),
        telephone: Some("06 00 00 00 01".into()),
        ville: Some("Lyon".into()),
        zip_code: Some("69003".into()),
        interets: Some("Escalade\nPhotographie".into()),
        ..no_change()
    };
    users::update_user_info(&pool, input).await.expect("mise à jour");
    users::update_description(&pool, Some("Étudiante en data.".into())).await.expect("description");
    let user = read(&pool).await.user;
    assert_eq!(user.prenom.as_deref(), Some("Alexandra"));
    assert_eq!(user.nom.as_deref(), Some("Martin"));
    assert_eq!(user.email.as_deref(), Some("alexandra@example.com"));
    assert_eq!(user.telephone.as_deref(), Some("06 00 00 00 01"));
    assert_eq!(user.ville.as_deref(), Some("Lyon"));
    assert_eq!(user.zip_code.as_deref(), Some("69003"));
    assert_eq!(user.interets.as_deref(), Some("Escalade\nPhotographie"));
    assert_eq!(user.description.as_deref(), Some("Étudiante en data."));

    // Suppression des champs facultatifs.
    let input = UpdateUserInfoInput {
        telephone: Some(String::new()),
        ville: Some(String::new()),
        zip_code: Some(String::new()),
        ..no_change()
    };
    users::update_user_info(&pool, input).await.expect("effacement");
    users::update_description(&pool, None).await.expect("effacement de la description");
    let user = read(&pool).await.user;
    assert_eq!(user.ville, None);
    assert_eq!(user.zip_code, None);
    assert_eq!(user.telephone, None);
    assert_eq!(user.description, None);

    // Un e-mail invalide est refusé, en français, sans rien modifier.
    let input = UpdateUserInfoInput { email: Some("sans-arobase".into()), ..no_change() };
    match users::update_user_info(&pool, input).await {
        Err(AppError::Validation(message)) => assert_eq!(message, "L'e-mail doit contenir un @."),
        other => panic!("validation attendue, obtenu {:?}", other.map(|u| u.email)),
    }
    assert_eq!(read(&pool).await.user.email.as_deref(), Some("alexandra@example.com"));
}

#[tokio::test]
async fn experiences_relues_par_le_profil() {
    let pool = pool_with_user().await;
    let input = |poste: &str| CreateExperienceInput {
        poste: poste.into(),
        entreprise: Some("Lumen Data".into()),
        lieu: Some("Lyon".into()),
        description: None,
        date_debut: date(2024, 9, 1),
        date_fin: None,
        type_contrat: Some("alternance".into()),
    };

    let id = experiences::insert(&pool, input("Alternant data"), seed::NOW).await.unwrap().id;
    assert_eq!(read(&pool).await.experiences[0].poste, "Alternant data");

    experiences::update(&pool, id, input("Alternant data engineer")).await.unwrap();
    let profil = read(&pool).await;
    assert_eq!(profil.experiences.len(), 1);
    assert_eq!(profil.experiences[0].poste, "Alternant data engineer");
    assert_eq!(profil.experiences[0].date_fin, None);

    experiences::delete(&pool, id).await.unwrap();
    assert!(read(&pool).await.experiences.is_empty());
}

#[tokio::test]
async fn formations_ajout_modification_suppression() {
    let pool = pool_with_user().await;
    let input = |titre: &str, ecole: Option<&str>| CreateFormationInput {
        titre: titre.into(),
        ecole: ecole.map(str::to_string),
        description: Some("Spécialité données".into()),
        date_debut: date(2023, 9, 1),
        date_fin: date(2025, 6, 30),
    };

    let id = formations::insert(&pool, input("Master", Some("Université Lyon 1")), seed::NOW).await.unwrap().id;
    assert_eq!(read(&pool).await.formations[0].titre, "Master");

    formations::update(&pool, id, input("Master informatique", None)).await.unwrap();
    let profil = read(&pool).await;
    assert_eq!(profil.formations.len(), 1);
    assert_eq!(profil.formations[0].titre, "Master informatique");
    assert_eq!(profil.formations[0].ecole, None);
    assert_eq!(profil.formations[0].description.as_deref(), Some("Spécialité données"));

    // Un titre vide est refusé, la formation reste inchangée.
    assert!(matches!(formations::update(&pool, id, input("  ", None)).await, Err(AppError::Validation(m)) if m == "Le titre est requis."));
    assert_eq!(read(&pool).await.formations[0].titre, "Master informatique");
    assert!(matches!(formations::update(&pool, 999, input("Licence", None)).await, Err(AppError::NotFound(_))));

    formations::delete(&pool, id).await.unwrap();
    assert!(read(&pool).await.formations.is_empty());
}

#[tokio::test]
async fn projets_ajout_modification_suppression() {
    let pool = pool_with_user().await;
    let input = |titre: &str, stack: &str| CreateProjetInput {
        titre: titre.into(),
        description: Some("Suivi des adhésions.".into()),
        stack: Some(stack.into()),
        url: None,
        pour_qui: Some("Association".into()),
        date_debut: date(2023, 1, 1),
        date_fin: date(2023, 6, 30),
    };

    let id = projets::insert(&pool, input("Tableau de bord", "Python"), seed::NOW).await.unwrap().id;
    assert_eq!(read(&pool).await.projets[0].stack.as_deref(), Some("Python"));

    projets::update(&pool, id, input("Tableau de bord associatif", "Python, SQL")).await.unwrap();
    let profil = read(&pool).await;
    assert_eq!(profil.projets.len(), 1);
    assert_eq!(profil.projets[0].titre, "Tableau de bord associatif");
    assert_eq!(profil.projets[0].stack.as_deref(), Some("Python, SQL"));
    assert!(matches!(projets::update(&pool, 999, input("X", "Y")).await, Err(AppError::NotFound(_))));

    projets::delete(&pool, id).await.unwrap();
    assert!(read(&pool).await.projets.is_empty());
}

#[tokio::test]
async fn competences_ajout_modification_suppression() {
    let pool = pool_with_user().await;
    let skill = |nom: &str, domaine: &str| CreateSkillInput { nom: nom.into(), domaine: domaine.into() };

    let sql = skills::insert(&pool, skill("SQL", "Données"), seed::NOW).await.unwrap().id;
    let python = skills::insert(&pool, skill("Pyton", "Données"), seed::NOW).await.unwrap().id;
    assert_eq!(read(&pool).await.skills.len(), 2);

    skills::update(&pool, python, skill("Python", "Langages")).await.unwrap();
    let profil = read(&pool).await;
    let corrige = profil.skills.iter().find(|s| s.id == python).expect("compétence modifiée");
    assert_eq!((corrige.nom.as_str(), corrige.domaine.as_str()), ("Python", "Langages"));

    skills::delete(&pool, sql).await.unwrap();
    let profil = read(&pool).await;
    assert_eq!(profil.skills.iter().map(|s| s.nom.as_str()).collect::<Vec<_>>(), ["Python"]);
    assert!(matches!(skills::delete(&pool, sql).await, Err(AppError::NotFound(m)) if m == "Compétence introuvable."));
}

#[tokio::test]
async fn langues_ajout_modification_suppression() {
    let pool = pool_with_user().await;
    let langue = |nom: &str, level: Option<&str>| CreateLangInput { nom: nom.into(), level: level.map(str::to_string) };

    let id = langues::insert(&pool, langue("Anglais", Some("B1")), seed::NOW).await.unwrap().id;
    assert_eq!(read(&pool).await.langues[0].level.as_deref(), Some("B1"));

    langues::update(&pool, id, langue("Anglais", Some("C1"))).await.unwrap();
    let profil = read(&pool).await;
    assert_eq!(profil.langues.len(), 1);
    assert_eq!(profil.langues[0].level.as_deref(), Some("C1"));
    assert!(matches!(langues::update(&pool, id, langue(" ", None)).await, Err(AppError::Validation(m)) if m == "Le nom de la langue est requis."));

    langues::delete(&pool, id).await.unwrap();
    assert!(read(&pool).await.langues.is_empty());
}
