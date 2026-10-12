//! Amorçage partagé des tests de la couche `db` : base en mémoire migrée et
//! insertion des lignes parentes (utilisateur, session, références,
//! candidature).

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use findmyjob_core::models::{CreateUserInput, TemplateArtifacts};
use findmyjob_lib::db::candidatures::{self, CandidatureFields};
use findmyjob_lib::db::document_versions::{self, NewVersion};
use findmyjob_lib::db::transaction::begin_write;
use findmyjob_lib::db::{documents, init, sessions, users};

/// Horodatage fixe utilisé par les tests.
pub const NOW: &str = "2026-10-10T08:00:00.000Z";

/// Ouvre une base en mémoire migrée.
pub async fn pool() -> SqlitePool {
    init::open_memory_pool().await.expect("base en mémoire")
}

/// Données d'onboarding minimales.
pub fn user_input() -> CreateUserInput {
    CreateUserInput {
        nom: "Martin".into(),
        prenom: "Alex".into(),
        email: "alex@example.com".into(),
        telephone: None,
        adresse: Some("Lyon".into()),
        permis: None,
        description: None,
        linkedin: None,
        github: None,
        portfolio: None,
    }
}

/// Crée l'utilisateur unique.
pub async fn user(pool: &SqlitePool) {
    users::insert_user(pool, user_input(), NOW).await.expect("utilisateur");
}

/// Crée une session et retourne son identifiant.
pub async fn session(pool: &SqlitePool) -> i64 {
    sessions::insert(pool, "Stage M2", NOW).await.expect("session").id
}

/// Fichiers de templatisation factices.
pub fn template() -> TemplateArtifacts {
    TemplateArtifacts {
        template_path: "documents/x/template.typ".into(),
        example_data_path: "documents/x/example_data.json".into(),
        page_count: 1,
    }
}

/// Insère une version de référence et la rend courante ; retourne son id.
pub async fn reference(pool: &SqlitePool, session_id: i64, doc_type: &str) -> i64 {
    let mut tx = begin_write(pool).await.expect("transaction");
    let id = document_versions::next_id(&mut tx).await.expect("id");
    let template = template();
    let new = NewVersion {
        id,
        session_id,
        doc_type,
        original_filename: "reference.typ",
        typst_path: "documents/x/reference.typ",
        pdf_path: "documents/x/reference.pdf",
        template: &template,
    };
    document_versions::insert_version(&mut tx, &new, NOW).await.expect("version");
    documents::set_current(&mut tx, session_id, doc_type, id, NOW).await.expect("pointeur");
    tx.commit().await.expect("commit");
    id
}

/// Champs de candidature valides.
pub fn fields(url: Option<&str>) -> CandidatureFields {
    CandidatureFields {
        poste: "Data engineer".into(),
        entreprise: "Acme".into(),
        localisation: "Paris".into(),
        type_contrat: "stage".into(),
        source: "Site carrière".into(),
        remuneration: None,
        url_offre: url.map(str::to_string),
        notes: None,
    }
}

/// Crée une candidature dans une session ; retourne son identifiant.
pub async fn candidature(pool: &SqlitePool, session_id: i64, now: &str) -> i64 {
    candidatures::insert(pool, session_id, &fields(None), "Offre", &[], now)
        .await
        .expect("candidature")
        .id
}
