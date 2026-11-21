//! Tests : validation déterministe d'une référence au contrat FindMyJob,
//! sur des rendus Typst réels. Chaque code d'erreur ou d'avertissement est
//! provoqué par une modification ciblée d'une référence conforme ; tous les
//! problèmes d'un même fichier sont rapportés ensemble, avec leur chemin ou
//! leur ligne, et une même erreur de compilation n'est citée qu'une fois.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::font_fixtures::install_reference_font;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::reference_contract::validate_reference;
use findmyjob_lib::error::AppError;
use findmyjob_lib::models::{ReferenceIssue, Severity};

/// Issue d'une validation : avertissements si acceptée, problèmes sinon.
type Outcome = Result<Vec<ReferenceIssue>, Vec<ReferenceIssue>>;

/// Valide une source sous un nom de fichier donné.
async fn validate_file(name: &str, filename: &str, source: &str, doc_type: &str) -> Outcome {
    let fonts = unique_temp_dir(name).join("polices");
    install_reference_font(&fonts);
    match validate_reference(filename, source, doc_type, &fonts).await {
        Ok(validated) => Ok(validated.warnings),
        Err(AppError::InvalidReference(issues)) => Err(issues),
        Err(other) => panic!("erreur inattendue : {other}"),
    }
}

/// Valide une source ; renvoie les avertissements ou les problèmes.
async fn validate(name: &str, source: &str, doc_type: &str) -> Outcome {
    validate_file(name, &format!("{name}.typ"), source, doc_type).await
}

/// Problèmes d'une source qui doit être refusée.
async fn rejected(name: &str, source: &str, doc_type: &str) -> Vec<ReferenceIssue> {
    validate(name, source, doc_type).await.expect_err("refus attendu")
}

/// Avertissements d'une source qui doit être acceptée.
async fn accepted(name: &str, source: &str, doc_type: &str) -> Vec<ReferenceIssue> {
    validate(name, source, doc_type).await.unwrap_or_else(|issues| panic!("refus inattendu : {issues:?}"))
}

/// Codes des problèmes.
fn codes(issues: &[ReferenceIssue]) -> Vec<&'static str> {
    issues.iter().map(|issue| issue.code.code()).collect()
}

/// Fixture dont des passages sont remplacés (chacun doit y figurer).
fn fixture_with(file: &str, edits: &[(&str, &str)]) -> String {
    let mut source = read_reference(file).replace("\r\n", "\n");
    for (old, new) in edits {
        assert!(source.contains(old), "passage absent de {file} : {old}");
        source = source.replacen(old, new, 1);
    }
    source
}

/// Référence de CV de test modifiée.
fn cv_with(edits: &[(&str, &str)]) -> String {
    fixture_with("cv_reference.typ", edits)
}

/// Référence de lettre de test modifiée.
fn lm_with(edits: &[(&str, &str)]) -> String {
    fixture_with("lm_reference.typ", edits)
}

// ======================================================================
// Références conformes
// ======================================================================

#[tokio::test]
async fn references_de_test_acceptees_sans_avertissement() {
    let references = [
        ("cv_reference.typ", "cv"),
        ("lm_reference.typ", "lettre_motivation"),
        ("tagged/cv_balise.typ", "cv"),
        ("tagged/lm_balisee.typ", "lettre_motivation"),
    ];
    for (file, doc_type) in references {
        let result = validate(&file.replace('/', "-"), &read_reference(file), doc_type).await;
        assert_eq!(result, Ok(Vec::new()), "{file}");
    }
}

// ======================================================================
// Fichier et constructions interdites (avant toute compilation)
// ======================================================================

#[tokio::test]
async fn fichier_non_recevable() {
    let source = read_reference("cv_reference.typ");
    let issues = validate_file("ref-extension", "cv.txt", &source, "cv").await.expect_err("refus attendu");
    assert_eq!(codes(&issues), vec!["not_typ"]);

    let large = format!("{source}\n// {}", "x".repeat(600 * 1024));
    assert_eq!(codes(&rejected("ref-taille", &large, "cv").await), vec!["too_large"]);

    let mangled = source.replacen("Camille", "Cam\u{FFFD}lle", 1);
    assert_eq!(codes(&rejected("ref-utf8", &mangled, "cv").await), vec!["not_utf8"]);
}

#[tokio::test]
async fn constructions_interdites_situees_sans_compilation() {
    let source = cv_with(&[("#let data = ", "#import \"autre.typ\": *\n#let logo = image(\"logo.png\")\n#let data = ")]);
    let issues = rejected("ref-interdit", &source, "cv").await;
    assert_eq!(codes(&issues), vec!["forbidden_construct", "forbidden_construct"], "{issues:?}");
    assert!(issues.iter().all(|issue| issue.line.is_some()), "{issues:?}");
    assert!(issues[0].line < issues[1].line);
}

// ======================================================================
// Compilation seule
// ======================================================================

#[tokio::test]
async fn erreur_de_compilation_avec_sa_ligne() {
    let source = cv_with(&[("#set page(", "#fonction_inconnue()\n#set page(")]);
    let issues = rejected("ref-compilation", &source, "cv").await;
    assert_eq!(codes(&issues), vec!["compile_error"], "{issues:?}");
    assert!(issues[0].line.is_some() && issues[0].detail.is_some(), "{issues:?}");
}

#[tokio::test]
async fn police_introuvable_nommee() {
    let source = cv_with(&[("#set text(size: 10pt", "#set text(font: \"PoliceInexistante\")\n#set text(size: 10pt")]);
    let issues = rejected("ref-police", &source, "cv").await;
    assert_eq!(codes(&issues), vec!["missing_font"], "{issues:?}");
    assert!(issues[0].detail.as_deref().is_some_and(|d| d.contains("PoliceInexistante")), "{issues:?}");
}

#[tokio::test]
async fn meme_erreur_de_compilation_citee_une_seule_fois() {
    // Compile seul (exemple), échoue dans chaque rendu de contrôle : une
    // seule mention de la même erreur.
    let source = cv_with(&[("#set page(", "#if \"data\" in sys.inputs { fonction_inconnue() }\n#set page(")]);
    let issues = rejected("ref-erreur-unique", &source, "cv").await;
    assert_eq!(issues.len(), 1, "{issues:?}");
}

// ======================================================================
// Métadonnée
// ======================================================================

#[tokio::test]
async fn metadonnee_absente_ou_dupliquee() {
    let missing = cv_with(&[(")) <findmyjob>", "))")]);
    assert_eq!(codes(&rejected("ref-meta-absente", &missing, "cv").await), vec!["metadata_missing"]);

    let twice = cv_with(&[("#let L = (", "#metadata((contract: \"findmyjob/cv\")) <findmyjob>\n#let L = (")]);
    assert_eq!(codes(&rejected("ref-meta-double", &twice, "cv").await), vec!["metadata_duplicated"]);
}

#[tokio::test]
async fn metadonnee_illisible() {
    let source = cv_with(&[("  contract: \"findmyjob/cv\",\n", "")]);
    let issues = rejected("ref-meta-illisible", &source, "cv").await;
    assert!(codes(&issues).contains(&"metadata_invalid"), "{issues:?}");
}

#[tokio::test]
async fn contrat_version_et_langues_controles() {
    let wrong = cv_with(&[("contract: \"findmyjob/cv\"", "contract: \"findmyjob/lettre\"")]);
    assert!(codes(&rejected("ref-contrat", &wrong, "cv").await).contains(&"wrong_contract"));

    let outdated = cv_with(&[("  version: 1,", "  version: 2,")]);
    assert!(codes(&rejected("ref-version", &outdated, "cv").await).contains(&"outdated_contract_version"));

    let one_language = cv_with(&[("languages: (\"fr\", \"en\"),", "languages: (\"fr\",),")]);
    let issues = rejected("ref-langue", &one_language, "cv").await;
    assert_eq!(codes(&issues), vec!["language_missing"], "{issues:?}");
    assert_eq!(issues[0].detail.as_deref(), Some("en"));
}

// ======================================================================
// Exemple et emplacements
// ======================================================================

#[tokio::test]
async fn chaque_ecart_de_l_exemple_rapporte_avec_son_chemin() {
    // Écarts que le template ne lit pas : il compile, seul le schéma les relève.
    let source = cv_with(&[
        ("    portfolio: \"\",\n", ""),
        ("    permis: false,\n", "    permis: false,\n    surnom: \"Cam\",\n"),
        ("    github: \"\",", "    github: 3,"),
    ]);
    let issues = rejected("ref-exemple", &source, "cv").await;
    assert!(issues.iter().all(|issue| issue.code.code() == "example_invalid"), "{issues:?}");
    let paths: Vec<&str> = issues.iter().filter_map(|issue| issue.path.as_deref()).collect();
    for expected in ["example.identity.portfolio", "example.identity.surnom", "example.identity.github"] {
        assert!(paths.contains(&expected), "{expected} absent de {paths:?}");
    }
}

#[tokio::test]
async fn emplacements_inconnus_invalides_ou_doubles() {
    let source = lm_with(&[(
        "    (field: \"subject\", mode: \"variable\", instruction: \"\"),",
        "    (field: \"subject\", mode: \"variable\", instruction: \"\"),\n\
         \x20   (field: \"summary\", mode: \"variable\", instruction: \"\"),\n\
         \x20   (field: \"paragraphs[0]\", mode: \"fixe\", instruction: \"\"),\n\
         \x20   (field: \"subject\", mode: \"adaptable\", instruction: \"\"),",
    )]);
    let issues = rejected("ref-emplacements", &source, "lettre_motivation").await;
    assert_eq!(codes(&issues), vec!["slot_unknown_field", "slot_invalid_mode", "slot_duplicated"], "{issues:?}");
    let paths: Vec<_> = issues.iter().map(|issue| issue.path.as_deref()).collect();
    assert_eq!(paths, vec![Some("slots[1].field"), Some("slots[2].mode"), Some("slots[3].field")]);
}

// ======================================================================
// Rendus de contrôle
// ======================================================================

#[tokio::test]
async fn langue_qui_ne_compile_pas() {
    let source = cv_with(&[("    profile: \"Profile\",\n", "")]);
    let issues = rejected("ref-langue-compile", &source, "cv").await;
    assert!(codes(&issues).contains(&"language_compile_error"), "{issues:?}");
}

#[tokio::test]
async fn langue_qui_ajoute_une_page() {
    let source = cv_with(&[("    profile: \"Profile\",", "    profile: \"Profile \" * 2500,")]);
    let issues = rejected("ref-langue-pages", &source, "cv").await;
    assert!(codes(&issues).contains(&"page_count_mismatch"), "{issues:?}");
}

#[tokio::test]
async fn donnees_vides_qui_ne_compilent_pas() {
    let source = cv_with(&[("#if data.summary != \"\" {", "#text(data.experiences.at(0).poste)\n#if data.summary != \"\" {")]);
    let issues = rejected("ref-donnees-vides", &source, "cv").await;
    assert!(codes(&issues).contains(&"empty_data_compile_error"), "{issues:?}");
}

#[tokio::test]
async fn nom_ecrit_en_dur_refuse() {
    let source = cv_with(&[(
        "upper(joined((data.identity.prenom, data.identity.nom), \" \"))",
        "upper(\"Camille Exemple\")",
    )]);
    let issues = rejected("cv-nom-en-dur", &source, "cv").await;
    assert!(codes(&issues).contains(&"hardcoded_value"), "{issues:?}");
}

#[tokio::test]
async fn titre_de_section_orphelin_refuse() {
    let source = cv_with(&[(
        "#if data.experiences.len() > 0 {\n  heading(L.experiences)",
        "#heading(L.experiences)\n#if data.experiences.len() > 0 {",
    )]);
    let issues = rejected("cv-titre-orphelin", &source, "cv").await;
    assert_eq!(codes(&issues), vec!["orphan_heading"], "{issues:?}");
    assert_eq!(issues[0].detail.as_deref(), Some("Expériences"));
}

#[tokio::test]
async fn symbole_de_liste_en_dur_refuse() {
    let source = cv_with(&[("list(marker: data.list_marker, ..e.bullets)", "list(marker: \"•\", ..e.bullets)")]);
    let issues = rejected("cv-symbole-en-dur", &source, "cv").await;
    assert_eq!(codes(&issues), vec!["list_marker_wrong"], "{issues:?}");
}

#[tokio::test]
async fn tous_les_problemes_rapportes_ensemble() {
    let source = cv_with(&[
        ("(field: \"summary\", mode: \"adaptable\"", "(field: \"interets\", mode: \"adaptable\""),
        ("upper(joined((data.identity.prenom, data.identity.nom), \" \"))", "upper(\"Camille Exemple\")"),
        ("list(marker: data.list_marker, ..e.bullets)", "list(marker: \"•\", ..e.bullets)"),
    ]);
    let issues = rejected("cv-tous-les-problemes", &source, "cv").await;
    for expected in ["slot_unknown_field", "hardcoded_value", "list_marker_wrong"] {
        assert!(codes(&issues).contains(&expected), "{expected} absent de {issues:?}");
    }
}

// ======================================================================
// Avertissements (import accepté)
// ======================================================================

#[tokio::test]
async fn separateur_orphelin_signale_avec_son_champ() {
    let source = cv_with(&[(
        "block(joined((e.poste, e.entreprise, e.lieu, e.period), \" | \"))",
        "block(e.poste + \" | \" + e.entreprise + \" | \" + e.lieu + \" | \" + e.period)",
    )]);
    let warnings = accepted("cv-separateur", &source, "cv").await;
    assert!(warnings.iter().all(|w| w.severity == Severity::Warning), "{warnings:?}");
    assert!(codes(&warnings).contains(&"orphan_separator"), "{warnings:?}");
    assert!(warnings.iter().any(|w| w.path.as_deref() == Some("lieu")), "{warnings:?}");
}

#[tokio::test]
async fn destinataire_de_largeur_fixe_signale() {
    let source = lm_with(&[
        (
            "#align(right, {\n  for line in (data.recipient.company",
            "#align(right, block(width: 7cm, {\n  set align(left)\n  for line in (data.recipient.company",
        ),
        ("    block(line)\n  }\n})\n\n#if data.date", "    block(line)\n  }\n}))\n\n#if data.date"),
    ]);
    let warnings = accepted("lm-destinataire-fixe", &source, "lettre_motivation").await;
    assert_eq!(codes(&warnings), vec!["recipient_not_right_aligned"], "{warnings:?}");
}

#[tokio::test]
async fn tirets_du_rendu_signales() {
    let source = lm_with(&[("salutations distinguées.\",", "salutations distinguées – merci.\",")]);
    let warnings = accepted("lm-tirets", &source, "lettre_motivation").await;
    assert_eq!(codes(&warnings), vec!["dash_marks"], "{warnings:?}");
}
