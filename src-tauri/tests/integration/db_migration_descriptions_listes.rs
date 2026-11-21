//! Test de la migration 0013 : les descriptions des expériences, projets et
//! formations passent au format « - élément ». Chaque ligne non vide devient
//! un élément, identique à celui que l'ancien découpage (`lines()`, recopié
//! ici comme référence) produisait pour le CV, sauf « -30 % » qui garde
//! désormais son signe. La migration rejouée ne change rien. Les fichiers
//! SQL sont appliqués à la main sur une connexion en mémoire, pour insérer
//! des descriptions à l'ancien format avant la migration testée.

// Importation des crates externes
use sqlx::{Connection, SqliteConnection};

// Importation des modules internes
use findmyjob_lib::docgen::description_list::parse_description;

/// Migrations antérieures, dans l'ordre d'application.
const MIGRATIONS_AVANT: [&str; 12] = [
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
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0011_session_terminee_si_acceptee.sql")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0012_format_liste_session.sql")),
];
/// Migration testée.
const MIGRATION_DESCRIPTIONS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations/sqlite/0013_descriptions_listes_tiret.sql"));

const NOW: &str = "2026-11-20T08:00:00.000Z";

/// Seule ligne dont l'élément migré diffère volontairement de l'ancien
/// découpage : le signe d'un nombre négatif est gardé.
const NOMBRE_NEGATIF: &str = "-30 % de coûts";

/// Descriptions à l'ancien format et texte attendu après la migration.
const CASES: [(Option<&str>, Option<&str>); 12] = [
    // Format réel de la base DEV (expériences et projets).
    (Some("• Développement d'une API\n• Tests"), Some("- Développement d'une API\n- Tests")),
    // Format réel de la base DEV (formation, demi-cadratins au milieu).
    (
        Some("Semestre d'échange – Université de Lisbonne, Portugal (09.2024 – 01.2025)"),
        Some("- Semestre d'échange – Université de Lisbonne, Portugal (09.2024 – 01.2025)"),
    ),
    (Some("•Collée\n   • Indentée  "), Some("- Collée\n- Indentée")),
    (Some("• Python • SQL"), Some("- Python • SQL")),
    (Some("Ligne sans puce\nAutre ligne"), Some("- Ligne sans puce\n- Autre ligne")),
    (Some("• A\r\n\r\n• B\r\n"), Some("- A\n- B")),
    (
        Some("-30 % de coûts\n* Étoile\n— Cadratin\n– Demi-cadratin"),
        Some("- -30 % de coûts\n- Étoile\n- Cadratin\n- Demi-cadratin"),
    ),
    (Some("- Déjà au format\n-Collé"), Some("- Déjà au format\n- Collé")),
    (Some("• - Double symbole"), Some("- - Double symbole")),
    (Some("\t\u{a0}• Espaces\u{a0}"), Some("- Espaces")),
    (Some("•\n   \n"), None),
    (None, None),
];

/// Ancien découpage du CV (`docgen::identity::lines`, avant la migration),
/// gardé ici comme référence de « rien perdu ».
fn old_lines(value: Option<&str>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    value
        .lines()
        .map(|line| line.trim().trim_start_matches(['•', '-', '*', '–', '—']).trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// Applique 0001 à 0012, insère chaque cas dans les trois tables (même id
/// que l'indice du cas) et retourne la connexion.
async fn seeded() -> SqliteConnection {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.expect("base en mémoire");
    for migration in MIGRATIONS_AVANT {
        sqlx::raw_sql(migration).execute(&mut conn).await.expect("migration antérieure");
    }
    for (id, (before, _)) in CASES.iter().enumerate() {
        for sql in [
            "INSERT INTO experiences (id, poste, description, created_at) VALUES ($1, 'Poste', $2, $3)",
            "INSERT INTO projets (id, titre, description, created_at) VALUES ($1, 'Projet', $2, $3)",
            "INSERT INTO formations (id, titre, description, created_at) VALUES ($1, 'Formation', $2, $3)",
        ] {
            sqlx::query(sql).bind(id as i64).bind(before).bind(NOW).execute(&mut conn).await.expect("description");
        }
    }
    conn
}

/// Applique 0013 dans une transaction, comme le migrateur.
async fn apply(conn: &mut SqliteConnection) {
    let mut tx = conn.begin().await.expect("transaction");
    sqlx::raw_sql(MIGRATION_DESCRIPTIONS).execute(&mut *tx).await.expect("migration 0013");
    tx.commit().await.expect("commit");
}

/// Descriptions d'une table, par id croissant.
async fn descriptions(conn: &mut SqliteConnection, table: &str) -> Vec<Option<String>> {
    let rows: Vec<(Option<String>,)> = sqlx::query_as(&format!("SELECT description FROM {table} ORDER BY id"))
        .fetch_all(&mut *conn)
        .await
        .expect("descriptions");
    rows.into_iter().map(|(description,)| description).collect()
}

/// Éléments du nouveau découpage : chaque ligne migrée est un élément d'une
/// ligne, sans intro.
fn new_items(migrated: Option<&str>) -> Vec<String> {
    let parsed = parse_description(migrated);
    assert!(parsed.intro.is_empty(), "intro inattendue après migration : {migrated:?}");
    parsed
        .items
        .into_iter()
        .map(|item| {
            assert_eq!(item.len(), 1, "suite de ligne inattendue après migration : {migrated:?}");
            item.concat()
        })
        .collect()
}

#[tokio::test]
async fn descriptions_converties_en_elements_a_tiret() {
    let mut conn = seeded().await;
    apply(&mut conn).await;

    let expected: Vec<Option<String>> = CASES.iter().map(|(_, after)| after.map(str::to_string)).collect();
    for table in ["experiences", "projets", "formations"] {
        assert_eq!(descriptions(&mut conn, table).await, expected, "table {table}");
    }
}

#[tokio::test]
async fn elements_identiques_a_l_ancien_decoupage() {
    let mut conn = seeded().await;
    apply(&mut conn).await;

    for table in ["experiences", "projets", "formations"] {
        let migrated = descriptions(&mut conn, table).await;
        for ((before, _), after) in CASES.iter().zip(&migrated) {
            let mut expected = old_lines(*before);
            // Seul écart voulu : le signe du nombre négatif est conservé.
            let sans_signe = NOMBRE_NEGATIF.trim_start_matches('-');
            for line in expected.iter_mut().filter(|line| *line == sans_signe) {
                *line = NOMBRE_NEGATIF.to_string();
            }
            assert_eq!(new_items(after.as_deref()), expected, "table {table}, description {before:?}");
        }
    }
}

#[tokio::test]
async fn migration_rejouee_sans_effet() {
    let mut conn = seeded().await;
    apply(&mut conn).await;
    let first: Vec<_> = [
        descriptions(&mut conn, "experiences").await,
        descriptions(&mut conn, "projets").await,
        descriptions(&mut conn, "formations").await,
    ]
    .concat();

    apply(&mut conn).await;
    let second: Vec<_> = [
        descriptions(&mut conn, "experiences").await,
        descriptions(&mut conn, "projets").await,
        descriptions(&mut conn, "formations").await,
    ]
    .concat();
    assert_eq!(first, second);
}

#[tokio::test]
async fn format_liste_puce_par_defaut() {
    let mut conn = seeded().await;
    sqlx::query("INSERT INTO sessions (id, nom, statut, created_at) VALUES (1, 'Session', 'active', $1)")
        .bind(NOW)
        .execute(&mut conn)
        .await
        .expect("session");
    let (format,): (String,) =
        sqlx::query_as("SELECT format_liste FROM sessions WHERE id = 1").fetch_one(&mut conn).await.expect("format");
    assert_eq!(format, "puce");

    let invalid = sqlx::query("UPDATE sessions SET format_liste = 'numero' WHERE id = 1").execute(&mut conn).await;
    assert!(invalid.is_err(), "CHECK de format_liste absent");
}
