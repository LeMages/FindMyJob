//! Test de la migration 0009 (retrait des centres d'intérêt du profil) :
//! la colonne `interets` disparaît de `users`, les autres colonnes et les
//! valeurs existantes restent intactes. Les fichiers SQL sont appliqués à
//! la main sur une connexion en mémoire, pour insérer le profil avant la
//! migration testée.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

/// Migrations antérieures, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 8] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0001_init.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0002_adresse_libre.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0003_adresse_seul_lieu.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0004_contacts_carnet.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0005_ville_code_postal.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0006_avertissements_generation.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0007_langue_profil.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0008_localisation_session.sql")),
];
/// Migration testée.
const MIGRATION_RETRAIT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0009_retrait_interets.sql"));

/// Noms des colonnes de `users`.
async fn user_columns(conn: &mut SqliteConnection) -> Vec<String> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM pragma_table_info('users')")
        .fetch_all(conn)
        .await
        .expect("colonnes");
    rows.into_iter().map(|(name,)| name).collect()
}

#[tokio::test]
async fn colonne_interets_supprimee_autres_colonnes_intactes() {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    sqlx::query(
        "INSERT INTO users (id, nom, prenom, email, interets, created_at) \
         VALUES (1, 'Martin', 'Alex', 'alex@example.com', 'Escalade', '2026-10-10T08:00:00.000Z')",
    )
    .execute(&mut conn)
    .await
    .expect("utilisateur");
    let before = user_columns(&mut conn).await;
    assert!(before.contains(&"interets".to_string()), "{before:?}");

    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_RETRAIT).execute(&mut *tx).await.expect("migration 0009");
    tx.commit().await.expect("commit");

    let after = user_columns(&mut conn).await;
    let expected: Vec<String> = before.into_iter().filter(|c| c != "interets").collect();
    assert_eq!(after, expected, "seule la colonne interets disparaît");
    let (nom, email): (String, String) =
        sqlx::query_as("SELECT nom, email FROM users WHERE id = 1").fetch_one(&mut conn).await.expect("profil");
    assert_eq!((nom.as_str(), email.as_str()), ("Martin", "alex@example.com"));
}
