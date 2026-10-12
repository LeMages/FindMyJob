//! Test des migrations 0002 et 0003 (lieu en un seul champ libre) : au
//! final, `adresse` reçoit l'ancienne ville et les colonnes de ville et de
//! code postal sont supprimées. Les fichiers SQL sont appliqués à la main sur
//! une connexion en mémoire, pour insérer une ligne avant les migrations
//! testées.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

/// Schéma initial, avant le passage à un seul champ de lieu.
const MIGRATION_INIT: &str = include_str!("../../../../db/migrations/sqlite/0001_init.sql");
/// Migrations testées, dans l'ordre d'application.
const MIGRATIONS_ADRESSE: [&str; 2] = [
    include_str!("../../../../db/migrations/sqlite/0002_adresse_libre.sql"),
    include_str!("../../../../db/migrations/sqlite/0003_adresse_seul_lieu.sql"),
];

/// Ouvre une base en mémoire au schéma initial (migration 0001).
async fn initial_db() -> SqliteConnection {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    sqlx::raw_sql(MIGRATION_INIT).execute(&mut conn).await.expect("migration 0001");
    conn
}

/// Applique les migrations testées (0002 puis 0003).
async fn apply_address_migrations(conn: &mut SqliteConnection) {
    for migration in MIGRATIONS_ADRESSE {
        sqlx::raw_sql(migration).execute(&mut *conn).await.expect("migration de l'adresse");
    }
}

/// Applique 0001, insère l'utilisateur avec l'ancienne adresse (rue, code
/// postal, ville) donnée, applique 0002 et 0003, puis relit `adresse`.
async fn migrate_address(rue: Option<&str>, code_postal: Option<&str>, ville: Option<&str>) -> Option<String> {
    let mut conn = initial_db().await;
    sqlx::query(
        "INSERT INTO users (id, nom, prenom, email, adresse, code_postal, ville, created_at) \
         VALUES (1, 'Martin', 'Alex', 'alex@example.com', $1, $2, $3, '2026-10-10T08:00:00.000Z')",
    )
    .bind(rue)
    .bind(code_postal)
    .bind(ville)
    .execute(&mut conn)
    .await
    .expect("utilisateur");
    apply_address_migrations(&mut conn).await;
    sqlx::query_scalar("SELECT adresse FROM users WHERE id = 1").fetch_one(&mut conn).await.expect("relecture")
}

#[tokio::test]
async fn reprend_la_ville_seule() {
    assert_eq!(migrate_address(Some("12 rue X"), Some("69001"), Some("Lyon")).await.as_deref(), Some("Lyon"));
    assert_eq!(migrate_address(None, None, Some("  Lyon ")).await.as_deref(), Some("Lyon"));
}

#[tokio::test]
async fn une_ville_vide_ou_nulle_donne_null() {
    assert_eq!(migrate_address(Some("12 rue X"), Some("69001"), None).await, None);
    assert_eq!(migrate_address(None, None, Some("   ")).await, None);
}

#[tokio::test]
async fn supprime_les_colonnes_ville_et_code_postal() {
    let mut conn = initial_db().await;
    apply_address_migrations(&mut conn).await;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('users') WHERE name IN ('ville', 'code_postal')")
            .fetch_one(&mut conn)
            .await
            .expect("pragma_table_info");
    assert_eq!(count, 0);
}
