//! Tests du verrouillage des sessions : accepter une offre termine la
//! session ; une session en pause ou terminée refuse toute modification
//! (session, candidatures, contacts liés, références) sauf son statut ; la
//! repasser « active » rouvre les modifications.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::seed;
use crate::shared::test_state;
use findmyjob_lib::app_state::AppState;
use findmyjob_lib::db::{candidatures, contacts, sessions, timeline_events};
use findmyjob_lib::documents;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{
    ContactInput, NewTimelineEvent, UpdateSessionInput, UpdateSessionListFormatInput, UpdateSessionLocationInput,
};
use findmyjob_lib::services;
use findmyjob_lib::services::candidatures::ApplicationMessage;

fn event(event_type: &str, content: Option<&str>) -> NewTimelineEvent {
    NewTimelineEvent {
        event_type: event_type.into(),
        content: content.map(str::to_string),
        interview_data: None,
        interview_id: None,
        interview_issue: None,
    }
}

fn contact(nom: &str) -> ContactInput {
    ContactInput { nom: nom.into(), role: None, email: None, telephone: None, notes: None }
}

fn location(ville: &str) -> UpdateSessionLocationInput {
    UpdateSessionLocationInput { ville: Some(ville.into()), zip_code: None }
}

fn list_format(format_liste: &str) -> UpdateSessionListFormatInput {
    UpdateSessionListFormatInput { format_liste: format_liste.into() }
}

/// Change le statut d'une session sans toucher à son nom.
async fn set_statut(pool: &sqlx::SqlitePool, sid: i64, statut: &str) {
    let nom = sessions::fetch_view(pool, sid).await.expect("session").nom;
    sessions::update(pool, sid, &UpdateSessionInput { nom, statut: statut.into() }).await.expect("statut modifiable");
}

/// Amène une candidature au statut « offre » (raccourci SQL : le parcours
/// complet de la timeline est testé dans `db_timeline_events.rs`).
async fn with_offer(pool: &sqlx::SqlitePool, sid: i64) -> i64 {
    let cid = seed::candidature(pool, sid, seed::NOW).await;
    sqlx::query("UPDATE candidatures SET status = 'offre' WHERE id = $1").bind(cid).execute(pool).await.unwrap();
    cid
}

fn assert_locked<T>(result: Result<T, AppError>, action: &str) {
    match result {
        Err(AppError::SessionLocked(_)) => {}
        Err(other) => panic!("{action} doit être refusé par le verrou, obtenu {other:?}"),
        Ok(_) => panic!("{action} doit être refusé, l'opération a réussi"),
    }
}

#[tokio::test]
async fn accepter_une_offre_termine_la_session() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = with_offer(&pool, sid).await;
    timeline_events::add_event(&pool, cid, event("accepter_offre", None), seed::NOW).await.unwrap();

    let session = sessions::fetch_view(&pool, sid).await.unwrap();
    assert_eq!(session.statut, "termine");
    assert_eq!(session.candidature_count, 1, "une candidature acceptée compte");
    assert_eq!(candidatures::fetch_status(&pool, cid).await.unwrap(), "accepte");
}

#[tokio::test]
async fn decliner_une_offre_ne_termine_pas_la_session() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    let cid = with_offer(&pool, sid).await;
    timeline_events::add_event(&pool, cid, event("decliner_offre", None), seed::NOW).await.unwrap();
    assert_eq!(sessions::fetch_view(&pool, sid).await.unwrap().statut, "active");
}

/// Candidatures préparées pendant que la session est active : une « à
/// postuler », une postulée avec un contact lié, et un contact du carnet libre.
struct Fixture {
    sid: i64,
    a_postuler: i64,
    postulee: i64,
    linked_contact: i64,
    free_contact: i64,
}

async fn fixture(state: &AppState) -> Fixture {
    let pool = &state.pool;
    let sid = seed::session(pool).await;
    seed::reference(pool, sid, "cv").await;
    let a_postuler = seed::candidature(pool, sid, seed::NOW).await;
    let postulee = seed::candidature(pool, sid, seed::NOW).await;
    candidatures::mark_postule(pool, postulee, None, seed::NOW).await.unwrap();
    let linked_contact = contacts::create(pool, &contact("Alice"), Some(postulee), seed::NOW).await.unwrap().id;
    let free_contact = contacts::create(pool, &contact("Bob"), None, seed::NOW).await.unwrap().id;
    Fixture { sid, a_postuler, postulee, linked_contact, free_contact }
}

/// Vérifie que chaque modification de la session est refusée.
async fn assert_every_write_refused(state: &AppState, f: &Fixture) {
    let pool = &state.pool;
    let nom = sessions::fetch_view(pool, f.sid).await.unwrap().nom;
    let statut = sessions::fetch_view(pool, f.sid).await.unwrap().statut;

    // Session : nom, localisation et format des listes
    let renamed = UpdateSessionInput { nom: format!("{nom} bis"), statut: statut.clone() };
    assert_locked(sessions::update(pool, f.sid, &renamed).await, "renommer la session");
    assert_locked(sessions::update_location(pool, f.sid, &location("Paris")).await, "changer la ville");
    assert_locked(sessions::update_list_format(pool, f.sid, &list_format("tiret")).await, "changer le format des listes");

    // Candidatures : création, champs, suppression, postulation, timeline
    let input = test_state::candidature_input(f.sid, "Offre", None);
    assert_locked(services::candidatures::create(state, &input).await, "créer une candidature (service)");
    assert_locked(candidatures::insert(pool, f.sid, &seed::fields(None), "Offre", &[], seed::NOW).await, "créer une candidature");
    assert_locked(candidatures::update(pool, f.a_postuler, &seed::fields(None)).await, "modifier une candidature");
    assert_locked(services::candidatures::delete(state, f.a_postuler).await, "supprimer une candidature");
    assert_locked(services::candidatures::postuler_sans_message(pool, f.a_postuler).await, "postuler sans message");
    assert_locked(
        services::candidatures::postuler_avec_message(state, f.a_postuler, ApplicationMessage::Mail).await,
        "postuler par mail",
    );
    let note = event("note", Some("Relancer lundi"));
    assert_locked(timeline_events::add_event(pool, f.postulee, note, seed::NOW).await, "ajouter un événement");

    // Contacts de la fiche : ajout, liaison, retrait
    assert_locked(contacts::create(pool, &contact("Carla"), Some(f.postulee), seed::NOW).await, "créer un contact lié");
    assert_locked(contacts::link(pool, f.postulee, f.free_contact, seed::NOW).await, "lier un contact");
    assert_locked(contacts::unlink(pool, f.postulee, f.linked_contact).await, "retirer un contact");

    // Références : import et restauration
    let source = read_reference("cv_reference.typ");
    assert_locked(documents::save_reference(state, f.sid, "cv", "cv.typ".into(), source).await, "importer une référence");
    assert_locked(documents::restore_version(pool, &state.data_dir, f.sid, "cv", 1).await, "restaurer une version");
}

#[tokio::test]
async fn session_en_pause_ou_terminee_en_lecture_seule() {
    for statut in ["pause", "termine"] {
        let app = test_state::app(&format!("session_lock_{statut}")).await;
        let (state, pool) = (&app.state, &app.state.pool);
        let f = fixture(state).await;
        let before = (test_state::count(pool, "candidatures").await, test_state::count(pool, "candidature_contacts").await);

        set_statut(pool, f.sid, statut).await;
        assert_every_write_refused(state, &f).await;

        assert_eq!(app.llm.total_calls(), 0, "{statut} : aucun appel LLM pour une action refusée");
        let after = (test_state::count(pool, "candidatures").await, test_state::count(pool, "candidature_contacts").await);
        assert_eq!(before, after, "{statut} : rien n'a été écrit");
        assert_eq!(candidatures::fetch_status(pool, f.a_postuler).await.unwrap(), "a_postuler");
    }
}

#[tokio::test]
async fn repasser_active_rouvre_les_modifications() {
    let app = test_state::app("session_lock_reouverture").await;
    let (state, pool) = (&app.state, &app.state.pool);
    let f = fixture(state).await;
    set_statut(pool, f.sid, "termine").await;
    // Le statut reste modifiable dans une session verrouillée.
    set_statut(pool, f.sid, "pause").await;
    set_statut(pool, f.sid, "active").await;

    let renamed = UpdateSessionInput { nom: "Alternance".into(), statut: "active".into() };
    assert_eq!(sessions::update(pool, f.sid, &renamed).await.unwrap().nom, "Alternance");
    assert_eq!(sessions::update_location(pool, f.sid, &location("Paris")).await.unwrap().ville.as_deref(), Some("Paris"));
    assert_eq!(sessions::update_list_format(pool, f.sid, &list_format("tiret")).await.unwrap().format_liste, "tiret");
    candidatures::update(pool, f.a_postuler, &seed::fields(None)).await.unwrap();
    timeline_events::add_event(pool, f.postulee, event("note", Some("Relancer lundi")), seed::NOW).await.unwrap();
    contacts::link(pool, f.postulee, f.free_contact, seed::NOW).await.unwrap();
    contacts::unlink(pool, f.postulee, f.linked_contact).await.unwrap();
    // La version factice n'a pas de fichiers : seul le verrou importe ici.
    match documents::restore_version(pool, &state.data_dir, f.sid, "cv", 1).await {
        Ok(_) | Err(AppError::Validation(_)) => {}
        Err(other) => panic!("restauration refusée par autre chose que les fichiers : {other:?}"),
    }
    services::candidatures::postuler_sans_message(pool, f.a_postuler).await.unwrap();
}

#[tokio::test]
async fn statut_change_sans_renommer_dans_une_session_verrouillee() {
    let pool = seed::pool().await;
    let sid = seed::session(&pool).await;
    set_statut(&pool, sid, "pause").await;
    let same = UpdateSessionInput { nom: " Stage M2 ".into(), statut: "termine".into() };
    assert_eq!(sessions::update(&pool, sid, &same).await.unwrap().statut, "termine", "nom identique après trim");
}
