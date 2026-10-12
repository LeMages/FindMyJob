//! Tests de `db::skills` et `db::langues` : doublons refusés avec un message
//! clair, tri insensible à la casse.

#[path = "shared/seed.rs"]
mod seed;

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::models::{CreateLangInput, CreateSkillInput};
use findmyjob_lib::db::{langues, skills};

fn skill(domaine: &str, nom: &str) -> CreateSkillInput {
    CreateSkillInput { domaine: domaine.into(), nom: nom.into() }
}

fn langue(nom: &str) -> CreateLangInput {
    CreateLangInput { nom: nom.into(), level: Some("B2".into()) }
}

#[tokio::test]
async fn competences_triees_sans_tenir_compte_de_la_casse() {
    let pool = seed::pool().await;
    for (domaine, nom) in [("Outils", "pgvector"), ("Outils", "Polars"), ("Data Engineering", "Kafka"), ("Data & ML", "Pandas")] {
        skills::insert(&pool, skill(domaine, nom), seed::NOW).await.unwrap();
    }
    let liste = skills::list_all(&pool).await.unwrap();
    let ordre: Vec<String> = liste.iter().map(|s| format!("{}/{}", s.domaine, s.nom)).collect();
    assert_eq!(ordre, ["Data & ML/Pandas", "Data Engineering/Kafka", "Outils/pgvector", "Outils/Polars"]);
}

#[tokio::test]
async fn competence_en_double_refusee_avec_message_clair() {
    let pool = seed::pool().await;
    let python = skills::insert(&pool, skill("Langages", "Python"), seed::NOW).await.unwrap();
    let rust = skills::insert(&pool, skill("Langages", "Rust"), seed::NOW).await.unwrap();

    match skills::insert(&pool, skill(" Langages ", "Python "), seed::NOW).await {
        Err(AppError::Conflict(message)) => {
            assert_eq!(message, "La compétence « Python » existe déjà dans le domaine « Langages ».")
        }
        other => panic!("conflit attendu, obtenu {:?}", other.map(|s| s.nom)),
    }
    assert!(matches!(
        skills::update(&pool, rust.id, skill("Langages", "Python")).await,
        Err(AppError::Conflict(_))
    ));
    // Réenregistrer une compétence sans changement n'est pas un doublon.
    skills::update(&pool, python.id, skill("Langages", "Python")).await.unwrap();
    // Même nom dans un autre domaine : autorisé.
    skills::insert(&pool, skill("Data & ML", "Python"), seed::NOW).await.unwrap();
}

#[tokio::test]
async fn langues_doublon_tri_et_suppression() {
    let pool = seed::pool().await;
    let francais = langues::insert(&pool, langue("français"), seed::NOW).await.unwrap();
    let anglais = langues::insert(&pool, langue("Anglais"), seed::NOW).await.unwrap();

    match langues::insert(&pool, langue("Anglais"), seed::NOW).await {
        Err(AppError::Conflict(message)) => assert_eq!(message, "La langue « Anglais » est déjà dans votre profil."),
        _ => panic!("conflit attendu"),
    }
    assert!(matches!(langues::update(&pool, anglais.id, langue("français")).await, Err(AppError::Conflict(_))));

    let noms: Vec<Option<String>> = langues::list_all(&pool).await.unwrap().into_iter().map(|l| l.nom).collect();
    assert_eq!(noms, [Some("Anglais".into()), Some("français".into())]);

    langues::delete(&pool, francais.id).await.unwrap();
    assert!(matches!(langues::update(&pool, francais.id, langue("Italien")).await, Err(AppError::NotFound(_))));
}
