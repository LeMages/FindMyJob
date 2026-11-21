//! Test de la migration 0008 (localisation de l'expéditeur du profil vers les
//! sessions) : la valeur du profil est copiée dans TOUTES les sessions
//! existantes (valeurs présentes ou NULL, aucune session), puis les colonnes
//! `ville` et `zip_code` disparaissent de `users`. Les fichiers SQL sont
//! appliqués à la main sur une connexion en mémoire, pour insérer des lignes
//! avant la migration testée.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

/// Migrations antérieures, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 7] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0001_init.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0002_adresse_libre.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0003_adresse_seul_lieu.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0004_contacts_carnet.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0005_ville_code_postal.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0006_avertissements_generation.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0007_langue_profil.sql")),
];
/// Migration testée.
const MIGRATION_LOCALISATION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0008_localisation_session.sql"));

/// Couple `(ville, zip_code)` tel que stocké.
type Location = (Option<String>, Option<String>);

/// Applique 0001 à 0007, insère le profil (si demandé) puis `session_count`
/// sessions, applique 0008 dans une transaction (comme le migrateur) et
/// retourne la connexion.
async fn migrate(profil: Option<(Option<&str>, Option<&str>)>, session_count: usize) -> SqliteConnection {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    if let Some((ville, zip_code)) = profil {
        sqlx::query(
            "INSERT INTO users (id, nom, prenom, email, ville, zip_code, created_at) \
             VALUES (1, 'Martin', 'Alex', 'alex@example.com', $1, $2, '2026-10-10T08:00:00.000Z')",
        )
        .bind(ville)
        .bind(zip_code)
        .execute(&mut conn)
        .await
        .expect("utilisateur");
    }
    for i in 0..session_count {
        sqlx::query("INSERT INTO sessions (nom, created_at) VALUES ($1, '2026-10-10T08:00:00.000Z')")
            .bind(format!("Session {i}"))
            .execute(&mut conn)
            .await
            .expect("session");
    }
    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_LOCALISATION).execute(&mut *tx).await.expect("migration 0008");
    tx.commit().await.expect("commit");
    conn
}

/// Localisation de toutes les sessions, dans l'ordre de création.
async fn session_locations(conn: &mut SqliteConnection) -> Vec<Location> {
    sqlx::query_as("SELECT ville, zip_code FROM sessions ORDER BY id").fetch_all(conn).await.expect("relecture")
}

/// Noms des colonnes d'une table.
async fn columns(conn: &mut SqliteConnection, table: &str) -> Vec<String> {
    let rows: Vec<(String,)> = sqlx::query_as(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .fetch_all(conn)
        .await
        .expect("colonnes");
    rows.into_iter().map(|(name,)| name).collect()
}

#[tokio::test]
async fn le_lieu_du_profil_est_copie_dans_toutes_les_sessions() {
    let mut conn = migrate(Some((Some("Lyon"), Some("69003"))), 3).await;

    let expected: Location = (Some("Lyon".to_string()), Some("69003".to_string()));
    assert_eq!(session_locations(&mut conn).await, vec![expected.clone(), expected.clone(), expected]);
}

#[tokio::test]
async fn les_valeurs_null_du_profil_restent_null() {
    let mut conn = migrate(Some((Some("Lyon"), None)), 2).await;
    let lyon: Location = (Some("Lyon".to_string()), None);
    assert_eq!(session_locations(&mut conn).await, vec![lyon.clone(), lyon]);

    let mut conn = migrate(Some((None, None)), 1).await;
    assert_eq!(session_locations(&mut conn).await, vec![(None, None)]);
}

#[tokio::test]
async fn sans_session_ni_profil_la_migration_passe() {
    let mut conn = migrate(Some((Some("Lyon"), Some("69003"))), 0).await;
    assert!(session_locations(&mut conn).await.is_empty());

    // Sessions sans profil : rien à copier, pas d'échec.
    let mut conn = migrate(None, 2).await;
    assert_eq!(session_locations(&mut conn).await, vec![(None, None), (None, None)]);
}

#[tokio::test]
async fn les_colonnes_du_profil_sont_supprimees() {
    let mut conn = migrate(Some((Some("Lyon"), Some("69003"))), 1).await;

    let users = columns(&mut conn, "users").await;
    assert!(!users.iter().any(|c| c == "ville" || c == "zip_code"), "colonnes restantes : {users:?}");
    let sessions = columns(&mut conn, "sessions").await;
    assert!(sessions.iter().any(|c| c == "ville") && sessions.iter().any(|c| c == "zip_code"), "{sessions:?}");
}
