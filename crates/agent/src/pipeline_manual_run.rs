//! Exécution manuelle du pipeline de génération sur une candidature
//! existante, sans rien modifier : ni la base, ni ses documents. Les
//! fichiers produits (PDF, `data.json`, trace) sont écrits dans un dossier
//! choisi par l'appelant.
//!
//! Test ignoré par défaut (appels LLM réels, base requise). Lancement :
//!
//! ```text
//! REGEN_CANDIDATURE_ID=11 REGEN_OUT_DIR=<dossier> \
//!   cargo test -p findmyjob-agent regenerate_candidature -- --ignored --nocapture
//! ```

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use crate::{db, documents, pipeline};

/// Lit une variable d'environnement obligatoire du test.
fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("variable d'environnement {name} requise"))
}

#[tokio::test]
#[ignore = "appels LLM réels et base de données requis : lancement manuel uniquement"]
async fn regenerate_candidature() {
    // ------------------------------------------------------------------
    // 1. Environnement de l'agent : les chemins de documents en base sont
    //    relatifs à la racine du dépôt (DATA_DIR=./data)
    // ------------------------------------------------------------------
    std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).expect("racine du dépôt");
    dotenvy::dotenv().expect("fichier .env à la racine du dépôt");
    let candidature_id: i64 = required_env("REGEN_CANDIDATURE_ID").parse().expect("identifiant numérique");
    let out_dir = PathBuf::from(required_env("REGEN_OUT_DIR"));
    std::fs::create_dir_all(&out_dir).expect("dossier de sortie créable");

    let pool = db::init_db(&db::DbConfig::from_env()).await.expect("connexion à la base");
    let data_dir = PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()));
    let fonts_dir = std::env::var("FONTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| documents::resolve_fonts_dir(&data_dir));

    // ------------------------------------------------------------------
    // 2. Génération, avec la candidature telle qu'en base
    // ------------------------------------------------------------------
    let candidature = db::candidatures::fetch_detail(&pool, candidature_id).await.expect("candidature existante");
    let session_id = candidature.session_id.expect("candidature rattachée à une session");
    let description = candidature.description_offre.unwrap_or_default();
    let request = pipeline::GenerationRequest { session_id, candidature_id, description_offre: &description };
    let generated = pipeline::generate_documents(&pool, &fonts_dir, request, |step, label| {
        println!("[{step}/{}] {label}", pipeline::GENERATION_STEPS_TOTAL)
    })
    .await
    .expect("génération réussie");

    // ------------------------------------------------------------------
    // 3. Écriture des résultats hors du dossier de données
    // ------------------------------------------------------------------
    let write = |name: &str, bytes: &[u8]| {
        std::fs::write(out_dir.join(name), bytes).unwrap_or_else(|e| panic!("écriture de {name} : {e}"))
    };
    write("cv.pdf", &generated.cv.pdf);
    write("cv.data.json", generated.cv.data_json.as_bytes());
    write("lm.pdf", &generated.lm.pdf);
    write("lm.data.json", generated.lm.data_json.as_bytes());
    for (name, content) in generated.trace.files().expect("trace sérialisable") {
        write(name, content.as_bytes());
    }
    println!("Résultats écrits dans {}", out_dir.display());
}
