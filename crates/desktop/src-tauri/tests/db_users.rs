//! Tests de `db::users` sur base en mémoire : création unique, permis
//! optionnel, mises à jour et contraintes.

#[path = "shared/seed.rs"]
mod seed;

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_core::models::UpdateUserInfoInput;
use findmyjob_lib::db::users;

#[tokio::test]
async fn cree_puis_relit_l_utilisateur_unique() {
    let pool = seed::pool().await;
    assert!(!users::user_exists(&pool).await.unwrap());
    assert!(matches!(users::fetch_user(&pool).await, Err(AppError::NotFound(_))));

    let user = users::insert_user(&pool, seed::user_input(), seed::NOW).await.unwrap();
    assert_eq!(user.id, 1);
    assert_eq!(user.permis, None, "le permis non renseigné reste NULL");
    assert_eq!(user.created_at.to_rfc3339(), "2026-10-10T08:00:00+00:00");
    assert!(users::user_exists(&pool).await.unwrap());
}

#[tokio::test]
async fn refuse_un_second_utilisateur() {
    let pool = seed::pool().await;
    seed::user(&pool).await;
    let second = users::insert_user(&pool, seed::user_input(), seed::NOW).await;
    assert!(matches!(second, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn valide_les_champs_obligatoires() {
    let pool = seed::pool().await;
    let mut input = seed::user_input();
    input.email = "sans-arobase".into();
    assert!(matches!(
        users::insert_user(&pool, input, seed::NOW).await,
        Err(AppError::Validation(_))
    ));
    assert!(!users::user_exists(&pool).await.unwrap(), "rien n'est écrit après un refus");
}

#[tokio::test]
async fn met_a_jour_les_infos_avec_coalesce() {
    let pool = seed::pool().await;
    seed::user(&pool).await;

    let input = UpdateUserInfoInput {
        prenom: None,
        nom: Some("Durand".into()),
        email: None,
        telephone: Some("0600000000".into()),
        adresse: None,
        code_postal: Some("69001".into()),
        permis: Some(true),
        linkedin: None,
        github: None,
        portfolio: None,
        interets: Some("Escalade".into()),
    };
    let user = users::update_user_info(&pool, input).await.unwrap();
    assert_eq!(user.nom.as_deref(), Some("Durand"));
    assert_eq!(user.prenom.as_deref(), Some("Alex"), "un champ absent est conservé");
    assert_eq!(user.permis, Some(true));
    assert_eq!(user.code_postal.as_deref(), Some("69001"));

    let user = users::update_description(&pool, Some("  Profil data  ".into())).await.unwrap();
    assert_eq!(user.description.as_deref(), Some("Profil data"));
}

#[tokio::test]
async fn la_base_refuse_un_permis_hors_booleen() {
    let pool = seed::pool().await;
    seed::user(&pool).await;
    let result = sqlx::query("UPDATE users SET permis = 2 WHERE id = 1").execute(&pool).await;
    assert!(result.is_err(), "CHECK (permis IN (0, 1))");
}
