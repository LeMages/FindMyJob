//! Sessions au niveau des fonctions appelées par les commandes : création,
//! lecture, modification et refus des données invalides avec un message en
//! français. Complète `db_sessions_documents` (statistiques, cascade).

// Importation des modules internes
use crate::shared::seed;
use findmyjob_core::error::AppError;
use findmyjob_core::models::UpdateSessionInput;
use findmyjob_lib::db::sessions;

/// Message d'une erreur de validation.
fn validation_message<T>(result: Result<T, AppError>) -> String {
    match result {
        Err(AppError::Validation(message)) => message,
        Err(other) => panic!("validation attendue, obtenu {other:?}"),
        Ok(_) => panic!("validation attendue, l'opération a réussi"),
    }
}

#[tokio::test]
async fn creation_lecture_et_modification() {
    let pool = seed::pool().await;

    let stage = sessions::insert(&pool, "  Stage M2  ", seed::NOW).await.expect("création");
    let alternance = sessions::insert(&pool, "Alternance 2027", seed::NOW).await.expect("création");
    assert_eq!(stage.nom, "Stage M2");
    assert_eq!(stage.statut, "active");
    assert_eq!(stage.candidature_count, 0);

    let listed = sessions::list_all(&pool).await.expect("lecture");
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|s| s.id == stage.id && s.nom == "Stage M2"));
    assert!(listed.iter().any(|s| s.id == alternance.id && s.nom == "Alternance 2027"));

    let input = UpdateSessionInput { nom: "Stage de fin d'études".into(), statut: "pause".into() };
    let updated = sessions::update(&pool, stage.id, &input).await.expect("modification");
    assert_eq!((updated.nom.as_str(), updated.statut.as_str()), ("Stage de fin d'études", "pause"));

    let relu = sessions::list_all(&pool).await.unwrap().into_iter().find(|s| s.id == stage.id).unwrap();
    assert_eq!(relu.statut, "pause");
    let autre = sessions::list_all(&pool).await.unwrap().into_iter().find(|s| s.id == alternance.id).unwrap();
    assert_eq!(autre.nom, "Alternance 2027");
}

#[tokio::test]
async fn donnees_invalides_refusees_en_francais() {
    let pool = seed::pool().await;

    assert_eq!(validation_message(sessions::insert(&pool, "   ", seed::NOW).await), "Le nom de la session est requis.");
    assert!(sessions::list_all(&pool).await.unwrap().is_empty());

    let id = seed::session(&pool).await;
    let nom_vide = UpdateSessionInput { nom: "".into(), statut: "active".into() };
    assert_eq!(validation_message(sessions::update(&pool, id, &nom_vide).await), "Le nom de la session est requis.");
    let statut_inconnu = UpdateSessionInput { nom: "Stage".into(), statut: "archivee".into() };
    assert_eq!(validation_message(sessions::update(&pool, id, &statut_inconnu).await), "Statut de session invalide.");

    // Rien n'a changé.
    let session = sessions::list_all(&pool).await.unwrap().remove(0);
    assert_eq!((session.nom.as_str(), session.statut.as_str()), ("Stage M2", "active"));

    let input = UpdateSessionInput { nom: "Stage".into(), statut: "active".into() };
    match sessions::update(&pool, 999, &input).await {
        Err(AppError::NotFound(message)) => assert_eq!(message, "Session introuvable."),
        other => panic!("introuvable attendu, obtenu {:?}", other.map(|s| s.id)),
    }
}
