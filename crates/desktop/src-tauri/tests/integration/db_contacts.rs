//! Tests du carnet de contacts (`db::contacts`) : CRUD et validation,
//! liaison N-N avec les candidatures, interlocuteurs des entretiens,
//! création d'une candidature avec ses contacts, et migration 0004.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection, SqlitePool};

// Importation des modules internes
use crate::shared::seed;
use findmyjob_core::error::AppError;
use findmyjob_core::models::{
    CandidatureContactInput, ContactInput, CreateCandidatureInput, NewInterviewData, NewTimelineEvent,
};
use findmyjob_lib::db::{candidatures, contacts, interviews, timeline_events};
use findmyjob_lib::services::candidature_validation;

// ======================================================================
// Helpers
// ======================================================================

/// Saisie de contact avec le nom donné, rôle « RH », reste vide.
fn input(nom: &str) -> ContactInput {
    ContactInput {
        nom: nom.into(),
        role: Some("RH".into()),
        email: Some("  ".into()),
        telephone: None,
        notes: None,
    }
}

/// Base migrée avec une session ; retourne le pool et la session.
async fn setup() -> (SqlitePool, i64) {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    (pool, sid)
}

/// Crée un contact lié à la candidature donnée ; retourne son id.
async fn linked_contact(pool: &SqlitePool, nom: &str, candidature_id: i64) -> i64 {
    contacts::create(pool, &input(nom), Some(candidature_id), seed::NOW).await.expect("contact").id
}

/// Événement `entretien_planifie` avec les interlocuteurs donnés.
fn planned_interview(contact_ids: Vec<i64>) -> NewTimelineEvent {
    NewTimelineEvent {
        event_type: "entretien_planifie".into(),
        content: None,
        interview_data: Some(NewInterviewData {
            type_entretien: "rh".into(),
            date_entretien: Some(chrono::Utc::now()),
            lieu: None,
            modalite: None,
            contact_ids,
        }),
        interview_id: None,
        interview_issue: None,
    }
}

/// Planifie un entretien sur une candidature déjà postulée ; retourne son id.
async fn plan(pool: &SqlitePool, candidature_id: i64, contact_ids: Vec<i64>) -> Result<i64, AppError> {
    timeline_events::add_event(pool, candidature_id, planned_interview(contact_ids), seed::NOW).await?;
    let id = sqlx::query_scalar("SELECT MAX(id) FROM interviews WHERE candidature_id = $1")
        .bind(candidature_id)
        .fetch_one(pool)
        .await
        .expect("entretien");
    Ok(id)
}

/// Nombre de lignes d'une table de liaison.
async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}")).fetch_one(pool).await.expect("comptage")
}

// ======================================================================
// CRUD et validation
// ======================================================================

#[tokio::test]
async fn crud_du_carnet() {
    let (pool, _) = setup().await;

    let created = contacts::create(&pool, &input("  Camille Martin "), None, seed::NOW).await.unwrap();
    assert_eq!(created.nom, "Camille Martin");
    assert_eq!(created.role.as_deref(), Some("RH"));
    assert_eq!(created.email, None);
    assert_eq!(created.candidature_count, 0);
    assert_eq!(contacts::list_all(&pool).await.unwrap().len(), 1);

    let mut change = input("Camille Martin");
    change.telephone = Some(" 06 00 00 00 00 ".into());
    change.role = Some("".into());
    let updated = contacts::update(&pool, created.id, &change).await.unwrap();
    assert_eq!(updated.telephone.as_deref(), Some("06 00 00 00 00"));
    assert_eq!(updated.role, None);

    contacts::delete(&pool, created.id).await.unwrap();
    assert!(contacts::list_all(&pool).await.unwrap().is_empty());
    assert!(matches!(contacts::delete(&pool, created.id).await, Err(AppError::NotFound(_))));
    assert!(matches!(contacts::update(&pool, created.id, &change).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn nom_obligatoire() {
    let (pool, _) = setup().await;
    assert!(matches!(contacts::create(&pool, &input("   "), None, seed::NOW).await, Err(AppError::Validation(_))));

    let id = contacts::create(&pool, &input("Camille"), None, seed::NOW).await.unwrap().id;
    assert!(matches!(contacts::update(&pool, id, &input("")).await, Err(AppError::Validation(_))));
    assert_eq!(contacts::fetch(&pool, id).await.unwrap().nom, "Camille");
}

// ======================================================================
// Liaison aux candidatures
// ======================================================================

#[tokio::test]
async fn meme_contact_sur_deux_candidatures() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    let c2 = seed::candidature(&pool, sid, seed::NOW).await;

    let id = linked_contact(&pool, "Camille", c1).await;
    contacts::link(&pool, c2, id, seed::NOW).await.unwrap();

    assert_eq!(contacts::list_for_candidature(&pool, c1).await.unwrap()[0].id, id);
    assert_eq!(contacts::list_for_candidature(&pool, c2).await.unwrap()[0].id, id);
    assert_eq!(contacts::fetch(&pool, id).await.unwrap().candidature_count, 2);
    assert!(matches!(contacts::link(&pool, c2, id, seed::NOW).await, Err(AppError::Conflict(_))));
    assert!(matches!(contacts::link(&pool, 999, id, seed::NOW).await, Err(AppError::NotFound(_))));
    assert!(matches!(contacts::link(&pool, c1, 999, seed::NOW).await, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn ordre_de_liaison() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    let premier = contacts::create(&pool, &input("Zoé"), None, seed::NOW).await.unwrap().id;
    let second = linked_contact(&pool, "Alex", c1).await;
    contacts::link(&pool, c1, premier, "2026-10-10T09:00:00.000Z").await.unwrap();

    let ids: Vec<i64> = contacts::list_for_candidature(&pool, c1).await.unwrap().iter().map(|c| c.id).collect();
    assert_eq!(ids, vec![second, premier]);
}

#[tokio::test]
async fn suppression_candidature_conserve_le_contact() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    let id = linked_contact(&pool, "Camille", c1).await;

    candidatures::delete(&pool, c1).await.unwrap();
    let contact = contacts::fetch(&pool, id).await.unwrap();
    assert_eq!(contact.candidature_count, 0);
    assert_eq!(count(&pool, "candidature_contacts").await, 0);
}

#[tokio::test]
async fn suppression_definitive_nettoie_les_liaisons() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    let c2 = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::mark_postule(&pool, c1, None, seed::NOW).await.unwrap();
    let id = linked_contact(&pool, "Camille", c1).await;
    contacts::link(&pool, c2, id, seed::NOW).await.unwrap();
    plan(&pool, c1, vec![id]).await.unwrap();

    contacts::delete(&pool, id).await.unwrap();
    assert!(contacts::list_for_candidature(&pool, c1).await.unwrap().is_empty());
    assert!(contacts::list_for_candidature(&pool, c2).await.unwrap().is_empty());
    assert_eq!(count(&pool, "candidature_contacts").await, 0);
    assert_eq!(count(&pool, "interview_contacts").await, 0);
    assert_eq!(count(&pool, "interviews").await, 1);
}

// ======================================================================
// Interlocuteurs des entretiens
// ======================================================================

#[tokio::test]
async fn plusieurs_interlocuteurs_par_entretien() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::mark_postule(&pool, c1, None, seed::NOW).await.unwrap();
    let a = linked_contact(&pool, "Alex", c1).await;
    let b = linked_contact(&pool, "Camille", c1).await;

    plan(&pool, c1, vec![a, b, a]).await.unwrap();
    assert_eq!(count(&pool, "interview_contacts").await, 2);

    let planning = interviews::list_for_session(&pool, sid).await.unwrap();
    let noms: Vec<&str> = planning[0].contacts.iter().map(|c| c.nom.as_str()).collect();
    assert_eq!(noms, vec!["Alex", "Camille"]);
}

#[tokio::test]
async fn refus_d_un_contact_non_lie() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    let c2 = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::mark_postule(&pool, c1, None, seed::NOW).await.unwrap();
    let autre = linked_contact(&pool, "Camille", c2).await;

    assert!(matches!(plan(&pool, c1, vec![autre]).await, Err(AppError::Validation(_))));
    assert!(matches!(plan(&pool, c1, vec![999]).await, Err(AppError::Validation(_))));
    assert_eq!(count(&pool, "interviews").await, 0);
    assert_eq!(candidatures::fetch_status(&pool, c1).await.unwrap(), "postule");
}

#[tokio::test]
async fn retrait_d_une_candidature_retire_des_entretiens() {
    let (pool, sid) = setup().await;
    let c1 = seed::candidature(&pool, sid, seed::NOW).await;
    let c2 = seed::candidature(&pool, sid, seed::NOW).await;
    candidatures::mark_postule(&pool, c1, None, seed::NOW).await.unwrap();
    candidatures::mark_postule(&pool, c2, None, seed::NOW).await.unwrap();
    let id = linked_contact(&pool, "Camille", c1).await;
    contacts::link(&pool, c2, id, seed::NOW).await.unwrap();
    plan(&pool, c1, vec![id]).await.unwrap();
    plan(&pool, c2, vec![id]).await.unwrap();

    contacts::unlink(&pool, c1, id).await.unwrap();
    assert!(contacts::list_for_candidature(&pool, c1).await.unwrap().is_empty());
    // L'entretien de l'autre candidature garde son interlocuteur.
    assert_eq!(count(&pool, "interview_contacts").await, 1);
    assert_eq!(contacts::fetch(&pool, id).await.unwrap().candidature_count, 1);
    assert!(matches!(contacts::unlink(&pool, c1, id).await, Err(AppError::NotFound(_))));
}

// ======================================================================
// Création d'une candidature avec ses contacts
// ======================================================================

/// Saisie de création de candidature valide, avec les contacts donnés.
fn create_input(session_id: i64, contacts: Vec<CandidatureContactInput>) -> CreateCandidatureInput {
    CreateCandidatureInput {
        poste: "Data engineer".into(),
        entreprise: Some("Acme".into()),
        localisation: Some("Paris".into()),
        type_contrat: Some("stage".into()),
        source: Some("LinkedIn".into()),
        remuneration: None,
        url_offre: None,
        description_offre: Some("Offre".into()),
        notes: None,
        cv_path: None,
        lettre_motivation_path: None,
        session_id: Some(session_id),
        contacts,
    }
}

/// Valide puis insère une candidature comme `services::candidatures::create`.
async fn create_candidature(pool: &SqlitePool, input: &CreateCandidatureInput) -> Result<i64, AppError> {
    let new = candidature_validation::validate_create(input)?;
    let view =
        candidatures::insert(pool, new.session_id, &new.fields, &new.description_offre, &new.contacts, seed::NOW)
            .await?;
    Ok(view.id)
}

#[tokio::test]
async fn creation_avec_un_contact_existant_et_un_nouveau() {
    let (pool, sid) = setup().await;
    let existant = contacts::create(&pool, &input("Alex"), None, seed::NOW).await.unwrap().id;

    let cid = create_candidature(
        &pool,
        &create_input(
            sid,
            vec![
                CandidatureContactInput::Existing { contact_id: existant },
                CandidatureContactInput::New { contact: input(" Camille ") },
                CandidatureContactInput::Existing { contact_id: existant },
            ],
        ),
    )
    .await
    .unwrap();

    let lies = contacts::list_for_candidature(&pool, cid).await.unwrap();
    let noms: Vec<&str> = lies.iter().map(|c| c.nom.as_str()).collect();
    assert_eq!(noms, vec!["Alex", "Camille"]);
    assert_eq!(contacts::list_all(&pool).await.unwrap().len(), 2);
}

#[tokio::test]
async fn creation_annulee_si_un_contact_echoue() {
    let (pool, sid) = setup().await;

    // Nouveau contact sans nom : refusé avant toute écriture.
    let invalide = create_input(sid, vec![CandidatureContactInput::New { contact: input(" ") }]);
    assert!(matches!(create_candidature(&pool, &invalide).await, Err(AppError::Validation(_))));

    // Contact existant introuvable : la transaction entière est annulée.
    let introuvable = create_input(
        sid,
        vec![
            CandidatureContactInput::New { contact: input("Camille") },
            CandidatureContactInput::Existing { contact_id: 999 },
        ],
    );
    assert!(matches!(create_candidature(&pool, &introuvable).await, Err(AppError::NotFound(_))));

    assert_eq!(count(&pool, "candidatures").await, 0);
    assert_eq!(count(&pool, "contacts").await, 0);
    assert_eq!(count(&pool, "status_history").await, 0);
}

// ======================================================================
// Migration 0004
// ======================================================================

/// Migrations antérieures au carnet, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 3] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0001_init.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0002_adresse_libre.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0003_adresse_seul_lieu.sql")),
];
/// Migration testée.
const MIGRATION_CARNET: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0004_contacts_carnet.sql"));

/// Données de l'ancien schéma : deux candidatures, trois contacts (nommé,
/// sans nom avec email, vide), un entretien avec `contact_id` et une ligne
/// d'historique qui pointe vers cet entretien.
const DONNEES_AVANT: &str = "
INSERT INTO sessions (id, nom, created_at) VALUES (1, 'Stage', '2026-10-10T08:00:00.000Z');
INSERT INTO candidatures (id, session_id, poste, type_contrat, source, description_offre, entreprise, localisation, created_at)
VALUES (1, 1, 'Data', 'stage', 'LinkedIn', 'Offre', 'Acme', 'Paris', '2026-10-10T08:00:00.000Z'),
       (2, 1, 'Dev', 'stage', 'LinkedIn', 'Offre', 'Beta', 'Lyon', '2026-10-10T08:00:00.000Z');
INSERT INTO contacts (id, candidature_id, nom, role, email, created_at)
VALUES (1, 1, ' Camille Martin ', 'RH', NULL, '2026-10-10T08:01:00.000Z'),
       (2, 1, '  ', NULL, ' rh@acme.fr ', '2026-10-10T08:02:00.000Z'),
       (3, 2, NULL, ' ', '', '2026-10-10T08:03:00.000Z');
INSERT INTO interviews (id, candidature_id, contact_id, type_entretien, issue, created_at)
VALUES (1, 1, 1, 'rh', 'en_attente', '2026-10-10T09:00:00.000Z'),
       (2, 2, 3, 'rh', 'en_attente', '2026-10-10T09:00:00.000Z');
INSERT INTO status_history (id, candidature_id, event_type, interview_id, changed_at)
VALUES (1, 1, 'entretien_planifie', 1, '2026-10-10T09:00:00.000Z');
";

/// Applique 0001 à 0003, insère les données de l'ancien schéma, puis
/// applique 0004 dans une transaction, comme le migrateur (clés étrangères
/// actives).
async fn migrated_db() -> SqliteConnection {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    sqlx::raw_sql(DONNEES_AVANT).execute(&mut conn).await.expect("données de l'ancien schéma");
    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_CARNET).execute(&mut *tx).await.expect("migration 0004");
    tx.commit().await.expect("commit");
    conn
}

#[tokio::test]
async fn migration_reprend_les_contacts_et_les_entretiens() {
    let mut conn = migrated_db().await;

    let fk: bool = sqlx::query_scalar("PRAGMA foreign_keys").fetch_one(&mut conn).await.expect("pragma");
    assert!(fk, "le test doit migrer avec les clés étrangères actives, comme l'application");

    let contacts: Vec<(i64, String, Option<String>)> =
        sqlx::query_as("SELECT id, nom, email FROM contacts ORDER BY id").fetch_all(&mut conn).await.expect("contacts");
    assert_eq!(
        contacts,
        vec![
            (1, "Camille Martin".to_string(), None),
            (2, "rh@acme.fr".to_string(), Some("rh@acme.fr".to_string())),
        ]
    );

    let liens: Vec<(i64, i64)> =
        sqlx::query_as("SELECT candidature_id, contact_id FROM candidature_contacts ORDER BY contact_id")
            .fetch_all(&mut conn)
            .await
            .expect("liaisons");
    assert_eq!(liens, vec![(1, 1), (1, 2)]);

    let interlocuteurs: Vec<(i64, i64)> = sqlx::query_as("SELECT interview_id, contact_id FROM interview_contacts")
        .fetch_all(&mut conn)
        .await
        .expect("interlocuteurs");
    assert_eq!(interlocuteurs, vec![(1, 1)]);

    let entretiens: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM interviews").fetch_one(&mut conn).await.expect("entretiens");
    assert_eq!(entretiens, 2);
    let historique: Option<i64> = sqlx::query_scalar("SELECT interview_id FROM status_history WHERE id = 1")
        .fetch_one(&mut conn)
        .await
        .expect("historique");
    assert_eq!(historique, Some(1));
}

#[tokio::test]
async fn migration_retire_les_anciennes_colonnes() {
    let mut conn = migrated_db().await;
    let colonnes: i64 = sqlx::query_scalar(
        "SELECT (SELECT COUNT(*) FROM pragma_table_info('interviews') WHERE name = 'contact_id') \
              + (SELECT COUNT(*) FROM pragma_table_info('contacts') WHERE name = 'candidature_id')",
    )
    .fetch_one(&mut conn)
    .await
    .expect("pragma_table_info");
    assert_eq!(colonnes, 0);

    let nom_obligatoire: bool =
        sqlx::query_scalar("SELECT \"notnull\" FROM pragma_table_info('contacts') WHERE name = 'nom'")
            .fetch_one(&mut conn)
            .await
            .expect("pragma_table_info");
    assert!(nom_obligatoire);

    let violations: Vec<(String,)> =
        sqlx::query_as("SELECT \"table\" FROM pragma_foreign_key_check").fetch_all(&mut conn).await.expect("fk");
    assert!(violations.is_empty());

    // Supprimer une candidature supprime ses liaisons, pas les contacts.
    sqlx::query("DELETE FROM candidatures WHERE id = 1").execute(&mut conn).await.expect("suppression");
    let restants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts").fetch_one(&mut conn).await.expect("contacts");
    assert_eq!(restants, 2);
}
