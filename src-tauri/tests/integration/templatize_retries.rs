//! Tests : boucle de templatisation (deux étapes) avec un LLM factice qui
//! rejoue des sorties défaillantes, dont des sorties réelles reprises des
//! traces de debug (`fixtures/llm/templatize/real_failures/`).

// Importation des modules standard
use std::sync::Arc;

// Importation des modules internes
use crate::shared::fake_llm::{read_reference, FakeLlm, Reply, Step};
use crate::shared::font_fixtures::install_reference_font;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::templatize::{templatize, TemplatizeInput, ValidatedTemplate};
use findmyjob_lib::documents::typst::compile_document;
use findmyjob_lib::error::AppError;
use findmyjob_lib::llm::client::TEMPLATIZE_MODEL;
use findmyjob_lib::llm::{ApiKey, ClaudeTarget};
use findmyjob_lib::models::ReferenceTexts;

/// Données d'exemple correctes de la lettre réelle.
const LM_EXAMPLE: &str = "references/real/lm_balisee_example_data.json";
/// Template correct de la lettre réelle.
const LM_TEMPLATE: &str = "references/real/lm_balisee_template.typ";

/// Templatise une référence réelle avec les réponses données.
async fn run(reference: &str, doc_type: &str, llm: Arc<FakeLlm>) -> Result<ValidatedTemplate, AppError> {
    let root = unique_temp_dir("templatize-retries");
    let fonts = root.join("polices");
    install_reference_font(&fonts);
    let source = read_reference(&format!("real/{reference}"));
    let rendered = compile_document(source.clone(), root.clone(), fonts.clone()).expect("la référence compile");
    // Le modèle des paramètres (Haiku) doit être remplacé par le modèle imposé.
    let target = ClaudeTarget {
        api_key: ApiKey::parse("sk-ant-test-0000").expect("clé de test"),
        model: "claude-haiku-4-5-20251001".to_string(),
        caller: llm,
        recorder: None,
    };
    let input = TemplatizeInput { source: &source, rendered: &rendered, doc_type, root: &root, fonts_dir: &fonts };
    templatize(&input, &target, None).await
}

/// Faux LLM de la lettre : données d'exemple puis templates donnés.
fn letter_llm(examples: &'static [&'static str], templates: &'static [&'static str]) -> Arc<FakeLlm> {
    let llm = Arc::new(FakeLlm::default());
    llm.respond_with(Step::ExampleLm, Reply::Files(examples));
    llm.respond_with(Step::TemplateLm, Reply::Files(templates));
    llm
}

#[tokio::test]
async fn sortie_reelle_defaillante_puis_correcte_acceptee_au_second_essai() {
    let llm = letter_llm(&[LM_EXAMPLE], &["llm/templatize/real_failures/lm_methode_inexistante.typ", LM_TEMPLATE]);
    let validated = run("lm_balisee.typ", "lettre_motivation", Arc::clone(&llm)).await.expect("acceptée");
    assert_eq!((validated.example_attempts, validated.template_attempts), (1, 2));
    assert_eq!(llm.models_for(Step::TemplateLm), [TEMPLATIZE_MODEL, TEMPLATIZE_MODEL], "modèle imposé");

    // Le second appel reçoit le template refusé et ses problèmes.
    let messages = llm.messages_for(Step::TemplateLm);
    assert!(messages[1].contains("TEMPLATE À CORRIGER"), "{}", messages[1]);
    assert!(messages[1].contains("Le template ne compile pas"), "{}", messages[1]);
    let texts: ReferenceTexts = serde_json::from_str(&validated.reference_texts_json).expect("textes de la référence");
    assert_eq!(texts.texts["t1"], "P.J. : Curriculum vitae");
    assert!(!texts.guidance.zones.is_empty(), "consignes lues dans le balisage");
}

#[tokio::test]
async fn correction_ciblee_appliquee_au_template_precedent() {
    let llm = letter_llm(&[LM_EXAMPLE], &["llm/templatize/lm_diese_en_mode_code.typ", "llm/templatize/lm_correction_ciblee.txt"]);
    let validated = run("lm_balisee.typ", "lettre_motivation", llm).await.expect("acceptée après correction");
    assert_eq!(validated.template_attempts, 2);
    assert!(validated.template_typ.contains("  if data.recipient.company != \"\""));
}

#[tokio::test]
async fn donnees_inventees_corrigees_au_second_essai() {
    let llm = letter_llm(&["llm/templatize/lm_exemple_ville_inventee.json", LM_EXAMPLE], &[LM_TEMPLATE]);
    let validated = run("lm_balisee.typ", "lettre_motivation", Arc::clone(&llm)).await.expect("acceptée");
    assert_eq!(validated.example_attempts, 2);
    assert!(llm.messages_for(Step::ExampleLm)[1].contains("identity.ville"), "le retour nomme le champ inventé");
}

#[tokio::test]
async fn meilleur_essai_conserve_sans_regression() {
    // Essai 1 : seul un séparateur orphelin (non bloquant) ; essais 2 et 3 :
    // erreurs de compilation. Le premier reste le meilleur et est accepté.
    let llm = Arc::new(FakeLlm::default());
    llm.respond_with(Step::ExampleCv, Reply::Files(&["references/real/cv_balise_example_data.json"]));
    llm.respond_with(
        Step::TemplateCv,
        Reply::Files(&[
            "llm/templatize/cv_separateur_orphelin.typ",
            "llm/templatize/real_failures/cv_hash_en_mode_code.typ",
        ]),
    );
    let validated = run("cv_balise.typ", "cv", Arc::clone(&llm)).await.expect("meilleur essai accepté");
    assert!(validated.template_typ.contains("l.nom + \" : \""), "le premier essai est retenu");
    assert_eq!(llm.calls_for(Step::TemplateCv), 3);
    // Chaque correction part du meilleur template, pas du dernier refusé.
    let last = llm.messages_for(Step::TemplateCv).pop().expect("troisième appel");
    assert!(last.contains("l.nom + \" : \""), "le template à corriger est le meilleur");
}

#[tokio::test]
async fn echec_definitif_message_lisible() {
    let llm = letter_llm(&[LM_EXAMPLE], &["llm/templatize/real_failures/lm_valeur_ajoutee.typ"]);
    let error = run("lm_balisee.typ", "lettre_motivation", llm).await.err().expect("échec");
    let message = error.to_string();
    assert!(message.starts_with("La mise en page de la référence n'a pas pu être reproduite"), "{message}");
    assert!(message.contains("Réessayez"), "{message}");
}

#[tokio::test]
async fn erreur_de_balisage_avant_tout_appel() {
    let llm = Arc::new(FakeLlm::default());
    let root = unique_temp_dir("templatize-balisage");
    let source = "// --- Corps --- [FIXE]\nTexte de la lettre.\n";
    let rendered = compile_document(source.to_string(), root.clone(), root.join("polices")).expect("compile");
    let target = ClaudeTarget {
        api_key: ApiKey::parse("sk-ant-test-0000").expect("clé de test"),
        model: TEMPLATIZE_MODEL.to_string(),
        caller: Arc::clone(&llm) as _,
        recorder: None,
    };
    let fonts = root.join("polices");
    let input = TemplatizeInput { source, rendered: &rendered, doc_type: "lettre_motivation", root: &root, fonts_dir: &fonts };
    let error = templatize(&input, &target, None).await.err().expect("balisage refusé");
    assert!(error.to_string().contains("ligne 1"), "{error}");
    assert_eq!(llm.total_calls(), 0, "aucun appel payé pour une erreur de balisage");
}
