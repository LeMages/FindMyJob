//! Tests : rédaction assainie. Budgets asymétriques (un texte un peu court
//! ne relance pas), meilleur essai conservé, profil seule source des listes
//! du CV, avertissement de pages visible dans la fiche de la candidature.

// Importation des modules standard
use std::cell::Cell;
use std::collections::BTreeMap;

// Importation des modules internes
use crate::common;
use crate::shared::fake_llm::{Reply, Step};
use crate::shared::seed;
use crate::shared::test_state::{self, candidature_input, wait_generation};
use findmyjob_lib::db;
use findmyjob_lib::docgen::build_cv_data::{build_cv_data, CvLayout};
use findmyjob_lib::docgen::entry_texts::source_texts;
use findmyjob_lib::docgen::lint::Violation;
use findmyjob_lib::docgen::selection::select;
use findmyjob_lib::llm::writing_loop::{write_with_retries, Review};
use findmyjob_lib::models::{CvContent, CvData, CvLabels, CvLangue, GenerationStatus, PeriodStyle, SkillsGroup};
use findmyjob_lib::services::candidatures;

/// Violation d'un texte trop court (signalée sans relance).
fn short() -> Violation {
    Violation { field: "summary".into(), message: "trop court (toléré)".into(), advisory: true }
}

/// Violation d'un texte trop long (relance).
fn long() -> Violation {
    Violation::new("summary", "trop long")
}

#[tokio::test]
async fn texte_trop_court_accepte_sans_relance() {
    let calls = Cell::new(0);
    let outcome = write_with_retries(
        "test",
        "Rappel : français.",
        |_| {
            calls.set(calls.get() + 1);
            async { Ok("court".to_string()) }
        },
        |_| Review { blocking: Vec::new(), violations: vec![short()] },
        |_| {},
    )
    .await
    .expect("accepté");
    assert_eq!(calls.get(), 1, "un écart toléré ne relance pas");
    assert_eq!(outcome.residual_violations, vec![short()], "l'écart reste tracé");
}

#[tokio::test]
async fn meilleur_essai_conserve_et_relance_avec_rappel_de_langue() {
    // Essai 1 : une violation stricte ; essais 2 et 3 : deux. Le premier
    // est retenu, et chaque relance part de lui avec le rappel de langue.
    let calls = Cell::new(0);
    let feedbacks = std::cell::RefCell::new(Vec::new());
    let outcome = write_with_retries(
        "test",
        "Rappel : anglais.",
        |feedback| {
            calls.set(calls.get() + 1);
            feedbacks.borrow_mut().extend(feedback);
            let attempt = calls.get();
            async move { Ok(format!("essai {attempt}")) }
        },
        |content: &String| {
            let violations = if content == "essai 1" { vec![long()] } else { vec![long(), long()] };
            Review { blocking: Vec::new(), violations }
        },
        |_| {},
    )
    .await
    .expect("non bloquant");
    assert_eq!(calls.get(), 3);
    assert_eq!(outcome.content, "essai 1");
    for feedback in feedbacks.borrow().iter() {
        assert_eq!(feedback.problems[0], "Rappel : anglais.");
        assert!(feedback.previous_output.contains("essai 1"), "{}", feedback.previous_output);
    }
}

#[test]
fn listes_du_cv_sans_repli_sur_la_reference() {
    // Profil sans langue ni centre d'intérêt, rédaction sans compétence :
    // les sections restent vides au lieu de reprendre celles de la référence.
    let mut profile = common::profile();
    profile.user.interets = None;
    let document = CvData {
        langues: vec![CvLangue { nom: "Allemand".into(), level: "C1".into() }],
        interets: vec!["Voile".into()],
        skills_groups: vec![SkillsGroup { category: "Référence".into(), skills: vec!["Fortran".into()] }],
        ..Default::default()
    };
    let selection = select(&profile, &[]);
    let texts = source_texts(&profile, &selection);
    let labels = CvLabels { months_short: vec![String::new(); 12], ..Default::default() };
    let no_texts = BTreeMap::new();
    let layout = CvLayout {
        document: &document,
        labels: &labels,
        period_style: &PeriodStyle::default(),
        separator: " – ",
        lang: "fr",
        texts: &no_texts,
    };
    let data = build_cv_data(&profile, &selection, &texts, &CvContent::default(), &layout);
    assert!(data.langues.is_empty() && data.interets.is_empty() && data.skills_groups.is_empty(), "{data:?}");
}

#[tokio::test]
async fn avertissement_de_pages_visible_dans_la_fiche() {
    let app = test_state::app("generation-pages").await;
    test_state::seed_profile(&app.state.pool).await;
    let sid = seed::session(&app.state.pool).await;
    test_state::seed_references(&app.state, sid).await;
    // Un résumé démesuré : trop long à chaque essai et à chaque relance.
    app.llm.respond_with(Step::CvContent, Reply::Fixture("cv_content_trop_long.json"));

    let view = candidatures::create(&app.state, &candidature_input(sid, "Offre de stage data.", None))
        .await
        .expect("création");
    let progress = wait_generation(&app.state, view.id).await;
    assert!(matches!(progress.status, GenerationStatus::Completed), "génération : {:?}", progress.error);

    let detail = db::candidatures::fetch_detail(&app.state.pool, view.id).await.expect("fiche");
    assert_eq!(detail.generation_warnings.len(), 1, "{:?}", detail.generation_warnings);
    assert!(detail.generation_warnings[0].starts_with("Le CV fait"), "{:?}", detail.generation_warnings);
    // Trois rédactions (initiale + deux relances pour les pages), trois essais chacune.
    assert_eq!(app.llm.calls_for(Step::CvContent), 9);
}
