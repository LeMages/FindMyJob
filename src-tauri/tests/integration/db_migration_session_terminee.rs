//! Test de la migration 0011 : une session qui contient une candidature
//! acceptée passe à « termine », les autres gardent leur statut. Les
//! fichiers SQL sont appliqués à la main sur une connexion en mémoire, pour
//! insérer les données avant la migration testée.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

/// Migrations antérieures, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 10] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0001_init.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0002_adresse_libre.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0003_adresse_seul_lieu.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0004_contacts_carnet.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0005_ville_code_postal.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0006_avertissements_generation.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0007_langue_profil.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0008_localisation_session.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0009_retrait_interets.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0010_contexte_detaille.sql")),
];
/// Migration testée.
const MIGRATION_SESSION_TERMINEE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0011_session_terminee_si_acceptee.sql"));

const NOW: &str = "2026-11-19T08:00:00.000Z";

/// Insère une session et une candidature par statut donné.
async fn session_with(conn: &mut SqliteConnection, id: i64, statut: &str, statuses: &[&str]) {
    sqlx::query("INSERT INTO sessions (id, nom, statut, created_at) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(format!("Session {id}"))
        .bind(statut)
        .bind(NOW)
        .execute(&mut *conn)
        .await
        .expect("session");
    for status in statuses {
        sqlx::query(
            "INSERT INTO candidatures \
               (poste, type_contrat, source, description_offre, entreprise, localisation, status, session_id, created_at) \
             VALUES ('Data engineer', 'stage', 'Site carrière', 'Offre', 'Acme', 'Paris', $1, $2, $3)",
        )
        .bind(status)
        .bind(id)
        .bind(NOW)
        .execute(&mut *conn)
        .await
        .expect("candidature");
    }
}

#[tokio::test]
async fn sessions_avec_offre_acceptee_terminees() {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    session_with(&mut conn, 1, "active", &["postule", "accepte"]).await;
    session_with(&mut conn, 2, "pause", &["accepte"]).await;
    session_with(&mut conn, 3, "active", &["refus", "offre_refusee", "offre"]).await;
    session_with(&mut conn, 4, "pause", &[]).await;

    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_SESSION_TERMINEE).execute(&mut *tx).await.expect("migration 0011");
    tx.commit().await.expect("commit");

    let statuts: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, statut FROM sessions ORDER BY id").fetch_all(&mut conn).await.expect("sessions");
    let expected = [(1, "termine"), (2, "termine"), (3, "active"), (4, "pause")];
    let statuts: Vec<(i64, &str)> = statuts.iter().map(|(id, s)| (*id, s.as_str())).collect();
    assert_eq!(statuts, expected);
}
