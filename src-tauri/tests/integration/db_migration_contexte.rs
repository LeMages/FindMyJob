//! Test de la migration 0010 (contexte détaillé) : la colonne `contexte`
//! s'ajoute à `experiences` et à `projets`, les lignes existantes restent
//! intactes avec un contexte NULL. Les fichiers SQL sont appliqués à la main
//! sur une connexion en mémoire, pour insérer les entrées avant la
//! migration testée.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

/// Migrations antérieures, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 9] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0001_init.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0002_adresse_libre.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0003_adresse_seul_lieu.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0004_contacts_carnet.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0005_ville_code_postal.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0006_avertissements_generation.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0007_langue_profil.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0008_localisation_session.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0009_retrait_interets.sql")),
];
/// Migration testée.
const MIGRATION_CONTEXTE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0010_contexte_detaille.sql"));

/// Noms des colonnes d'une table.
async fn columns(conn: &mut SqliteConnection, table: &str) -> Vec<String> {
    let rows: Vec<(String,)> = sqlx::query_as(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .fetch_all(conn)
        .await
        .expect("colonnes");
    rows.into_iter().map(|(name,)| name).collect()
}

#[tokio::test]
async fn colonne_contexte_ajoutee_lignes_existantes_intactes() {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    sqlx::raw_sql(
        "INSERT INTO experiences (id, poste, description, created_at) \
         VALUES (1, 'Alternant data engineer', 'Pipelines SQL.', '2026-11-19T08:00:00.000Z'); \
         INSERT INTO projets (id, titre, stack, created_at) \
         VALUES (1, 'Tableau de bord', 'Python', '2026-11-19T08:00:00.000Z');",
    )
    .execute(&mut conn)
    .await
    .expect("entrées du profil");
    let before = (columns(&mut conn, "experiences").await, columns(&mut conn, "projets").await);

    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_CONTEXTE).execute(&mut *tx).await.expect("migration 0010");
    tx.commit().await.expect("commit");

    for (table, previous) in [("experiences", before.0), ("projets", before.1)] {
        let mut expected = previous;
        expected.push("contexte".to_string());
        assert_eq!(columns(&mut conn, table).await, expected, "{table} : seule la colonne contexte s'ajoute");
    }
    let (poste, description, contexte): (String, String, Option<String>) =
        sqlx::query_as("SELECT poste, description, contexte FROM experiences WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .expect("expérience");
    assert_eq!((poste.as_str(), description.as_str(), contexte), ("Alternant data engineer", "Pipelines SQL.", None));
    let (titre, stack, contexte): (String, String, Option<String>) =
        sqlx::query_as("SELECT titre, stack, contexte FROM projets WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .expect("projet");
    assert_eq!((titre.as_str(), stack.as_str(), contexte), ("Tableau de bord", "Python", None));
}
