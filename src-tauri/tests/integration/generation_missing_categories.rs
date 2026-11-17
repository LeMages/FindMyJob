//! Tests : catégories du profil retirées du CV. Une unité affichée par
//! l'exemple de la référence mais vide dans le profil est retirée avec un
//! avertissement visible dans la fiche, sans reprise du texte d'exemple ;
//! le permis non renseigné se distingue du permis à « non » ; un profil
//! sans compétence n'impose plus de groupe au modèle.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::common;
use crate::shared::fake_llm::Step;
use crate::shared::seed;
use crate::shared::test_state::{self, artefact, candidature_input, wait_generation, TestApp};
use findmyjob_lib::db;
use findmyjob_lib::docgen::profile_categories::{missing_categories, ProfileCategory};
use findmyjob_lib::llm::cv_writing::cv_content_schema_for;
use findmyjob_lib::models::{CvData, CvLangue, GenerationStatus, Identity, SkillsGroup};
use findmyjob_lib::services::candidatures;

/// Exemple de référence qui affiche toutes les catégories.
fn full_example() -> CvData {
    CvData {
        experiences: vec![Default::default()],
        projects: vec![Default::default()],
        formations: vec![Default::default()],
        skills_groups: vec![SkillsGroup { category: "Cat".into(), skills: vec!["x".into()] }],
        langues: vec![CvLangue { nom: "Exemplais".into(), level: "courant".into() }],
        identity: Identity { permis: true, ..Default::default() },
        ..Default::default()
    }
}

#[test]
fn profil_complet_aucune_categorie_manquante() {
    let profile = common::profile();
    let missing = missing_categories(&full_example(), &profile);
    assert!(!missing.contains(&ProfileCategory::Experiences), "{missing:?}");
    assert!(!missing.contains(&ProfileCategory::Permis), "{missing:?}");
}

#[test]
fn categorie_connue_vide_dans_le_profil_signalee() {
    let mut profile = common::profile();
    profile.skills.clear();
    profile.langues.clear();
    let missing = missing_categories(&full_example(), &profile);
    assert!(missing.contains(&ProfileCategory::Competences) && missing.contains(&ProfileCategory::Langues), "{missing:?}");
    assert!(!missing.contains(&ProfileCategory::Experiences), "{missing:?}");
    assert!(ProfileCategory::Competences.warning().contains("aucune compétence"));
}

#[test]
fn categorie_absente_de_l_exemple_jamais_signalee() {
    let mut profile = common::profile();
    profile.skills.clear();
    let mut example = full_example();
    example.skills_groups.clear();
    assert!(!missing_categories(&example, &profile).contains(&ProfileCategory::Competences));
}

#[test]
fn permis_non_renseigne_signale_permis_non_ne_l_est_pas() {
    let mut profile = common::profile();
    profile.user.permis = None;
    assert!(missing_categories(&full_example(), &profile).contains(&ProfileCategory::Permis));
    profile.user.permis = Some(false);
    let missing = missing_categories(&full_example(), &profile);
    assert!(!missing.contains(&ProfileCategory::Permis), "un « non » est une vraie réponse : {missing:?}");
}

#[test]
fn schema_sans_competences_omet_skills_groups() {
    let schema = cv_content_schema_for(false);
    assert!(schema["properties"].get("skills_groups").is_none(), "{schema}");
    assert!(cv_content_schema_for(true)["properties"].get("skills_groups").is_some());
}

/// Génère un CV avec un profil auquel on a retiré compétences et langues.
async fn generate_without_skills_and_languages(name: &str) -> (TestApp, i64, i64) {
    let app = test_state::app(name).await;
    test_state::seed_profile(&app.state.pool).await;
    sqlx::query("DELETE FROM skills").execute(&app.state.pool).await.expect("suppression des compétences");
    sqlx::query("DELETE FROM langues").execute(&app.state.pool).await.expect("suppression des langues");
    let sid = seed::session(&app.state.pool).await;
    test_state::seed_references(&app.state, sid).await;
    let view = candidatures::create(&app.state, &candidature_input(sid, "Offre de stage data.", None))
        .await
        .expect("création");
    let progress = wait_generation(&app.state, view.id).await;
    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);
    (app, sid, view.id)
}

#[tokio::test]
async fn unite_vide_retiree_avec_avertissement_dans_la_fiche() {
    let (app, sid, cid) = generate_without_skills_and_languages("generation-categories-vides").await;

    let detail = db::candidatures::fetch_detail(&app.state.pool, cid).await.expect("fiche");
    let warnings = &detail.generation_warnings;
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings.iter().any(|w| w.contains("aucune compétence")), "{warnings:?}");
    assert!(warnings.iter().any(|w| w.contains("aucune langue")), "{warnings:?}");

    // Aucun texte d'exemple de la référence n'est repris.
    let raw = std::fs::read_to_string(artefact(&app.state, sid, "cv", cid, "data.json")).expect("data.json");
    let data: Value = serde_json::from_str(&raw).expect("data.json valide");
    assert_eq!(data["skills_groups"], serde_json::json!([]));
    assert_eq!(data["langues"], serde_json::json!([]));
    assert!(!raw.contains("Exemplais"), "texte d'exemple repris : {raw}");
}

#[tokio::test]
async fn profil_sans_competences_la_redaction_ne_demande_aucun_groupe() {
    let (app, _, _) = generate_without_skills_and_languages("generation-sans-competences").await;
    let messages = app.llm.messages_for(Step::CvContent);
    assert!(!messages.is_empty());
    assert!(messages.iter().all(|m| m.contains("ne produis pas ce champ")), "{messages:?}");
    assert!(messages.iter().all(|m| !m.contains("exactement 0 groupes")), "{messages:?}");
}
