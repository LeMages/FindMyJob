//! Test de la migration 0005 (lieu en `ville` + `zip_code`) : reprise depuis
//! l'ancienne `adresse` libre (code postal FR ou CH, ville avant ou après
//! le code postal, ville seule, rue sans code postal, vide), puis
//! suppression de la colonne `adresse`. Les fichiers SQL sont appliqués à
//! la main sur une connexion en mémoire, pour insérer une ligne avant la
//! migration testée. L'application unique par le migrateur est couverte
//! par `db_init` (réouverture d'une base déjà migrée).

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

/// Migrations antérieures, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 4] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0001_init.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0002_adresse_libre.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0003_adresse_seul_lieu.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0004_contacts_carnet.sql")),
];
/// Migration testée.
const MIGRATION_VILLE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../db/migrations/sqlite/0005_ville_code_postal.sql"));

/// Applique 0001 à 0004, insère l'utilisateur avec l'adresse donnée,
/// applique 0005 dans une transaction (comme le migrateur), puis relit
/// `(ville, zip_code)`.
async fn migrate(adresse: Option<&str>) -> (Option<String>, Option<String>) {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    sqlx::query(
        "INSERT INTO users (id, nom, prenom, email, adresse, created_at) \
         VALUES (1, 'Martin', 'Alex', 'alex@example.com', $1, '2026-10-10T08:00:00.000Z')",
    )
    .bind(adresse)
    .execute(&mut conn)
    .await
    .expect("utilisateur");
    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_VILLE).execute(&mut *tx).await.expect("migration 0005");
    tx.commit().await.expect("commit");
    sqlx::query_as("SELECT ville, zip_code FROM users WHERE id = 1").fetch_one(&mut conn).await.expect("relecture")
}

/// Raccourci : couple attendu.
fn expected(ville: Option<&str>, zip_code: Option<&str>) -> (Option<String>, Option<String>) {
    (ville.map(str::to_string), zip_code.map(str::to_string))
}

#[tokio::test]
async fn code_postal_fr_suivi_de_la_ville() {
    assert_eq!(migrate(Some("1 rue de l'Exemple, 69000 Lyon")).await, expected(Some("Lyon"), Some("69000")));
    assert_eq!(migrate(Some("12 rue des Lilas\n69003 Lyon")).await, expected(Some("Lyon"), Some("69003")));
    assert_eq!(migrate(Some("74300 Thyez, France")).await, expected(Some("Thyez"), Some("74300")));
}

#[tokio::test]
async fn code_postal_ch_suivi_de_la_ville() {
    assert_eq!(migrate(Some("Rue du Lac 12, 1200 Genève")).await, expected(Some("Genève"), Some("1200")));
}

#[tokio::test]
async fn ville_avant_le_code_postal() {
    assert_eq!(migrate(Some("Thyez, 74300")).await, expected(Some("Thyez"), Some("74300")));
    assert_eq!(migrate(Some("12 rue X, 74300")).await, expected(None, Some("74300")));
}

#[tokio::test]
async fn ville_seule() {
    assert_eq!(migrate(Some("  Lyon ")).await, expected(Some("Lyon"), None));
    assert_eq!(migrate(Some("Saint-Julien-en-Genevois")).await, expected(Some("Saint-Julien-en-Genevois"), None));
}

#[tokio::test]
async fn rue_sans_code_postal_ou_vide() {
    assert_eq!(migrate(Some("12 rue des Lilas")).await, expected(None, None));
    assert_eq!(migrate(Some("Avenue de la Gare")).await, expected(None, None));
    assert_eq!(migrate(Some("   ")).await, expected(None, None));
    assert_eq!(migrate(None).await, expected(None, None));
}

#[tokio::test]
async fn supprime_la_colonne_adresse() {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT.iter().chain([&MIGRATION_VILLE]) {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration");
    }
    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('users') WHERE name IN ('adresse', 'ville', 'zip_code') ORDER BY name")
            .fetch_all(&mut conn)
            .await
            .expect("pragma_table_info");
    assert_eq!(columns, vec!["ville".to_string(), "zip_code".to_string()]);
}
