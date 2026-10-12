//! Tests des services liés aux fichiers : résolution des chemins reçus du
//! frontend, nettoyage des générations interrompues, suppression d'une
//! session avec son dossier, et validation des champs de candidature.

// Importation des modules standard
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Importation des modules internes
use crate::shared::temp_dir::unique_temp_dir;
use crate::shared::{fake_secret_store, seed};
use fake_secret_store::FakeSecretStore;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::AnthropicCaller;
use findmyjob_core::models::CreateCandidatureInput;
use findmyjob_lib::app_state::AppState;
use findmyjob_lib::documents;
use findmyjob_lib::progress::ProgressStore;
use findmyjob_lib::services::api_key::ApiKeyAccess;
use findmyjob_lib::services::{candidature_validation, generation, sessions};
use findmyjob_lib::startup::run_mode::RunMode;
use findmyjob_lib::storage_paths;

/// Écrit un fichier (et ses dossiers parents) sous le dossier de données.
fn touch(data_dir: &Path, relative: &str) -> PathBuf {
    let path = storage_paths::to_absolute(data_dir, relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"%PDF").unwrap();
    path
}

#[test]
fn les_chemins_recus_restent_sous_le_dossier_de_donnees() {
    let data_dir = unique_temp_dir("paths");
    let pdf = storage_paths::artefact_file(1, "cv", 7, "pdf");
    assert_eq!(pdf, "documents/sessions/1/cv/artefact/7.pdf");
    touch(&data_dir, &pdf);

    assert!(storage_paths::resolve_untrusted(&data_dir, &pdf).unwrap().is_file());
    assert!(!storage_paths::resolve_untrusted(&data_dir, "documents/absent.pdf").unwrap().exists());
    for refused in ["", "../secret.pdf", "documents/../../secret.pdf", "/etc/passwd", "C:\\Windows\\win.ini"] {
        assert!(
            matches!(storage_paths::resolve_untrusted(&data_dir, refused), Err(AppError::Validation(_))),
            "« {refused} » doit être refusé"
        );
    }

    assert_eq!(documents::read_document_bytes(&data_dir, &pdf).unwrap(), b"%PDF");
    touch(&data_dir, "documents/notes.txt");
    assert!(matches!(documents::read_document_bytes(&data_dir, "documents/notes.txt"), Err(AppError::NotFound(_))));
    assert!(matches!(documents::read_document_bytes(&data_dir, "../x.pdf"), Err(AppError::NotFound(_))));
    assert!(documents::document_info(&data_dir, &pdf).unwrap().exists);

    fs::remove_dir_all(&data_dir).unwrap();
}

#[tokio::test]
async fn le_demarrage_nettoie_les_generations_interrompues() {
    let data_dir = unique_temp_dir("interrupted");
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let interrompue = seed::candidature(&pool, sid, seed::NOW).await;
    let partiel = touch(&data_dir, &storage_paths::artefact_file(sid, "cv", interrompue, "data.json"));
    let trace = touch(&data_dir, &format!("{}/offer.json", storage_paths::debug_dir(sid, interrompue)));

    assert_eq!(generation::clean_interrupted(&pool, &data_dir).await.unwrap(), 1);
    assert!(!partiel.exists() && !trace.exists(), "fichiers partiels supprimés");
    assert_eq!(generation::clean_interrupted(&pool, &data_dir).await.unwrap(), 0);

    fs::remove_dir_all(&data_dir).unwrap();
}

#[tokio::test]
async fn supprimer_une_session_supprime_son_dossier() {
    let data_dir = unique_temp_dir("session");
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let fichier = touch(&data_dir, &format!("{}/reference.typ", storage_paths::version_dir(sid, "cv", 1)));
    let state = AppState {
        pool,
        fonts_dir: storage_paths::fonts_dir(&data_dir),
        data_dir: data_dir.clone(),
        capture_profile_dir: data_dir.join("capture_webview"),
        progress: ProgressStore::new(),
        api_key: ApiKeyAccess::new(RunMode::Prod, Arc::new(FakeSecretStore::default()), || None),
        // Jamais appelé : supprimer une session ne passe pas par Claude.
        llm: Arc::new(AnthropicCaller),
    };

    sessions::delete(&state, sid).await.unwrap();
    assert!(!fichier.exists());
    assert!(!storage_paths::to_absolute(&data_dir, &storage_paths::session_dir(sid)).exists());
    assert!(matches!(sessions::delete(&state, sid).await, Err(AppError::NotFound(_))));

    fs::remove_dir_all(&data_dir).unwrap();
}

#[test]
fn validation_des_champs_de_candidature() {
    let mut input = CreateCandidatureInput {
        poste: " Data engineer ".into(),
        entreprise: Some("Acme".into()),
        localisation: Some("Paris".into()),
        type_contrat: Some("Stage".into()),
        source: Some("LinkedIn".into()),
        remuneration: Some("  ".into()),
        url_offre: None,
        description_offre: Some("Offre".into()),
        notes: None,
        cv_path: None,
        lettre_motivation_path: None,
        session_id: Some(1),
        contacts: Vec::new(),
    };
    let new = candidature_validation::validate_create(&input).unwrap();
    assert_eq!((new.fields.poste.as_str(), new.fields.type_contrat.as_str()), ("Data engineer", "stage"));
    assert_eq!(new.fields.remuneration, None);

    input.localisation = None;
    input.description_offre = None;
    match candidature_validation::validate_create(&input) {
        Err(AppError::Validation(message)) => assert_eq!(message, "Le lieu est requis."),
        _ => panic!("validation attendue"),
    }
}
