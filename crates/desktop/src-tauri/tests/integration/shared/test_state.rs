//! État d'application des tests de services : base en mémoire, dossier de
//! données temporaire, faux trousseau et faux exécutant Claude. Rien ne lit
//! le `.env`, `dev-data`, le trousseau du système ni le réseau.

// Importation des modules standard
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// Importation des crates externes
use chrono::NaiveDate;
use sqlx::SqlitePool;

// Importation des modules internes
use crate::shared::fake_llm::{read_reference, FakeLlm};
use crate::shared::fake_secret_store::FakeSecretStore;
use crate::shared::seed;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_core::models::{
    CreateCandidatureInput, CreateExperienceInput, CreateFormationInput, CreateLangInput, CreateProjetInput,
    CreateSkillInput, GenerationProgress, GenerationStatus, UpdateUserInfoInput,
};
use findmyjob_lib::app_state::AppState;
use findmyjob_lib::db;
use findmyjob_lib::documents;
use findmyjob_lib::progress::ProgressStore;
use findmyjob_lib::services::api_key::ApiKeyAccess;
use findmyjob_lib::startup::run_mode::RunMode;
use findmyjob_lib::storage_paths;

/// Clé factice rangée dans le faux trousseau : jamais envoyée nulle part,
/// le faux exécutant ne fait aucun appel réseau.
const FAKE_API_KEY: &str = "sk-ant-test-0000";
/// Nombre maximal d'attentes de la fin d'une génération.
const MAX_GENERATION_POLLS: u32 = 600;
/// Intervalle entre deux lectures de la progression.
const GENERATION_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Application de test.
///
/// # Attributes
///
/// * `state` - État partagé, comme celui construit au démarrage.
/// * `llm` - Faux exécutant Claude, pour régler ses réponses et compter ses appels.
pub struct TestApp {
    pub state: AppState,
    pub llm: Arc<FakeLlm>,
}

/// Construit une application de test isolée.
///
/// # Arguments
///
/// * `name` - Nom unique du test (dossier de données temporaire).
pub async fn app(name: &str) -> TestApp {
    let data_dir = unique_temp_dir(name);
    let llm = Arc::new(FakeLlm::default());
    let state = AppState {
        pool: seed::pool().await,
        fonts_dir: storage_paths::fonts_dir(&data_dir),
        capture_profile_dir: data_dir.join("capture_webview"),
        data_dir,
        progress: ProgressStore::new(),
        api_key: ApiKeyAccess::new(RunMode::Prod, Arc::new(FakeSecretStore::with(FAKE_API_KEY)), || None),
        llm: llm.clone(),
    };
    TestApp { state, llm }
}

/// Date fixe.
pub fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

/// Crée un profil complet : identité, une expérience, un projet, une
/// formation, deux compétences et une langue. Identifiants attendus par les
/// fixtures LLM : expérience 1, projet 1.
pub async fn seed_profile(pool: &SqlitePool) {
    seed::user(pool).await;
    let info = UpdateUserInfoInput {
        prenom: None,
        nom: None,
        email: None,
        telephone: Some("06 12 34 56 78".into()),
        ville: None,
        zip_code: None,
        permis: Some(true),
        linkedin: None,
        github: None,
        portfolio: None,
        interets: Some("Escalade".into()),
    };
    db::users::update_user_info(pool, info).await.expect("infos utilisateur");
    let experience = CreateExperienceInput {
        poste: "Alternant data engineer".into(),
        entreprise: Some("Lumen Data".into()),
        lieu: Some("Lyon".into()),
        description: Some("Pipelines SQL pour le reporting.".into()),
        date_debut: Some(date(2024, 9, 1)),
        date_fin: Some(date(2025, 8, 31)),
        type_contrat: Some("alternance".into()),
    };
    db::experiences::insert(pool, experience, seed::NOW).await.expect("expérience");
    let projet = CreateProjetInput {
        titre: "Tableau de bord associatif".into(),
        description: Some("Suivi des adhésions en Python.".into()),
        stack: Some("Python".into()),
        url: None,
        pour_qui: Some("Association Escalade Lyon".into()),
        date_debut: Some(date(2023, 1, 1)),
        date_fin: Some(date(2023, 6, 30)),
    };
    db::projets::insert(pool, projet, seed::NOW).await.expect("projet");
    let formation = CreateFormationInput {
        titre: "Master informatique".into(),
        ecole: Some("Université Lyon 1".into()),
        description: Some("Spécialité données".into()),
        date_debut: Some(date(2023, 9, 1)),
        date_fin: Some(date(2025, 6, 30)),
    };
    db::formations::insert(pool, formation, seed::NOW).await.expect("formation");
    for nom in ["SQL", "Python"] {
        let skill = CreateSkillInput { nom: nom.into(), domaine: "Données".into() };
        db::skills::insert(pool, skill, seed::NOW).await.expect("compétence");
    }
    let langue = CreateLangInput { nom: "Anglais".into(), level: Some("B2".into()) };
    db::langues::insert(pool, langue, seed::NOW).await.expect("langue");
}

/// Source Typst d'une référence de test : le template de test, avec ses
/// données d'exemple écrites dans le document au lieu d'être lues depuis
/// `data.json`. Rendu, elle affiche donc exactement le texte attendu de la
/// templatisation.
pub fn reference_source(template: &str, example_data: &str) -> String {
    let template = read_reference(template);
    let example: serde_json::Value = serde_json::from_str(&read_reference(example_data)).expect("JSON d'exemple");
    let data = example.to_string().replace('\\', "\\\\").replace('"', "\\\"");
    template.replacen("#let data = json(\"data.json\")", &format!("#let data = json(bytes(\"{data}\"))"), 1)
}

/// Dépose et templatise les références CV et lettre d'une session (le faux
/// exécutant fournit la templatisation).
pub async fn seed_references(state: &AppState, session_id: i64) {
    let cv = reference_source("cv_template.typ", "cv_example_data.json");
    documents::save_reference(state, session_id, "cv", "cv.typ".into(), cv).await.expect("référence CV");
    let lm = reference_source("lm_template.typ", "lm_example_data.json");
    documents::save_reference(state, session_id, "lettre_motivation", "lettre.typ".into(), lm)
        .await
        .expect("référence lettre");
}

/// Données de création d'une candidature dans une session.
pub fn candidature_input(session_id: i64, description: &str, url: Option<&str>) -> CreateCandidatureInput {
    CreateCandidatureInput {
        poste: "Data engineer stagiaire".into(),
        entreprise: Some("Lumen Santé".into()),
        localisation: Some("Paris".into()),
        type_contrat: Some("stage".into()),
        source: Some("Site carrière".into()),
        remuneration: None,
        url_offre: url.map(str::to_string),
        description_offre: Some(description.into()),
        notes: None,
        cv_path: None,
        lettre_motivation_path: None,
        session_id: Some(session_id),
        contacts: Vec::new(),
    }
}

/// Attend la fin de la génération d'une candidature (bornée).
///
/// # Returns
///
/// La progression finale (`Completed` ou `Failed`).
pub async fn wait_generation(state: &AppState, candidature_id: i64) -> GenerationProgress {
    for _ in 0..MAX_GENERATION_POLLS {
        if let Some(progress) = state.progress.get(candidature_id) {
            if !matches!(progress.status, GenerationStatus::InProgress) {
                return progress;
            }
        }
        tokio::time::sleep(GENERATION_POLL_INTERVAL).await;
    }
    panic!("génération {candidature_id} non terminée après {MAX_GENERATION_POLLS} attentes");
}

/// Chemin absolu d'un artefact généré.
pub fn artefact(state: &AppState, session_id: i64, doc_type: &str, candidature_id: i64, extension: &str) -> PathBuf {
    storage_paths::to_absolute(&state.data_dir, &storage_paths::artefact_file(session_id, doc_type, candidature_id, extension))
}

/// Nombre de lignes d'une table.
pub async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}")).fetch_one(pool).await.expect("comptage")
}
