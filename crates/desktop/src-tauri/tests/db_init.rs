//! Test de `db::init::open_pool` sur un vrai fichier : création, pragmas
//! appliqués à chaque connexion et suivi des migrations à la réouverture.

// Importation des modules standard
use std::fs;

// Importation des modules internes
use findmyjob_lib::db::init;

#[tokio::test]
async fn ouvre_un_fichier_en_wal_avec_cles_etrangeres() {
    let dir = std::env::temp_dir().join(format!("findmyjob-db-init-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("findmyjob.db");

    for _ in 0..2 {
        let pool = init::open_pool(&path).await.unwrap();
        let journal: String = sqlx::query_scalar("PRAGMA journal_mode").fetch_one(&pool).await.unwrap();
        let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys").fetch_one(&pool).await.unwrap();
        let timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout").fetch_one(&pool).await.unwrap();
        let migrations: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations").fetch_one(&pool).await.unwrap();
        assert_eq!((journal.as_str(), fk, timeout, migrations), ("wal", 1, 5000, 3));
        pool.close().await;
    }

    // Sous Windows, le fichier peut rester verrouillé quelques instants après
    // `close()` (fermeture asynchrone du worker sqlx) : le nettoyage du
    // dossier temporaire ne conditionne pas le résultat du test.
    if let Err(e) = fs::remove_dir_all(&dir) {
        eprintln!("Nettoyage de {} impossible : {e}", dir.display());
    }
}
