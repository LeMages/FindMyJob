//! Tests : validation déterministe d'une référence au contrat FindMyJob,
//! sur des rendus Typst réels.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::reference_contract::validate_reference;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::ReferenceIssue;

/// Valide une source ; renvoie les avertissements ou les problèmes.
async fn validate(name: &str, source: &str, doc_type: &str) -> Result<Vec<ReferenceIssue>, Vec<ReferenceIssue>> {
    let fonts = unique_temp_dir(name).join("polices");
    match validate_reference(&format!("{name}.typ"), source, doc_type, &fonts).await {
        Ok(validated) => Ok(validated.warnings),
        Err(AppError::InvalidReference(issues)) => Err(issues),
        Err(other) => panic!("erreur inattendue : {other}"),
    }
}

#[tokio::test]
async fn references_de_test_acceptees_sans_avertissement() {
    for (file, doc_type) in [("cv_reference.typ", "cv"), ("lm_reference.typ", "lettre_motivation")] {
        let result = validate(file, &read_reference(file), doc_type).await;
        assert_eq!(result, Ok(Vec::new()), "{file}");
    }
}

/// Codes des problèmes d'un rejet.
fn codes(issues: &[ReferenceIssue]) -> Vec<&'static str> {
    issues.iter().map(|issue| issue.code.code()).collect()
}

/// Référence de CV de test dont un passage est remplacé.
fn cv_with(old: &str, new: &str) -> String {
    let source = read_reference("cv_reference.typ");
    assert!(source.contains(old), "passage absent de la fixture : {old}");
    source.replacen(old, new, 1)
}

#[tokio::test]
async fn nom_ecrit_en_dur_refuse() {
    let source = cv_with(
        "upper(joined((data.identity.prenom, data.identity.nom), \" \"))",
        "upper(\"Camille Exemple\")",
    );
    let issues = validate("cv-nom-en-dur", &source, "cv").await.expect_err("refus attendu");
    assert!(codes(&issues).contains(&"hardcoded_value"), "{issues:?}");
}

#[tokio::test]
async fn titre_de_section_orphelin_refuse() {
    let source = cv_with(
        "#if data.experiences.len() > 0 {\n  heading(L.experiences)",
        "#heading(L.experiences)\n#if data.experiences.len() > 0 {",
    );
    let issues = validate("cv-titre-orphelin", &source, "cv").await.expect_err("refus attendu");
    assert_eq!(codes(&issues), vec!["orphan_heading"], "{issues:?}");
    assert_eq!(issues[0].detail.as_deref(), Some("Expériences"));
}

#[tokio::test]
async fn symbole_de_liste_en_dur_refuse() {
    let source = cv_with("list(marker: data.list_marker, ..e.bullets)", "list(marker: \"•\", ..e.bullets)");
    let issues = validate("cv-symbole-en-dur", &source, "cv").await.expect_err("refus attendu");
    assert_eq!(codes(&issues), vec!["list_marker_wrong"], "{issues:?}");
}
