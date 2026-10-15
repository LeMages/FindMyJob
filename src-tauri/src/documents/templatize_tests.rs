//! Tests de la validation des templates (`documents::templatize`) :
//! contrôle sentinelle, oscillation des retries, et non-régression sur deux
//! références réelles (anonymisées) avec un template écrit à la main, qui
//! doit passer `validate` en entier.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use super::*;
use crate::documents::typst::compile_document;

/// Dossier des références réelles et de leurs templates écrits à la main.
const REAL_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/references/real");

/// Dossier racine sans fichier annexe, et dossier de polices inexistant.
fn dirs() -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir();
    let fonts = root.join("findmyjob-no-fonts");
    (root, fonts)
}

// ======================================================================
// Contrôle sentinelle et oscillation
// ======================================================================

/// Compile un template de CV avec les données sentinelles et renvoie le
/// problème relevé pour les valeurs données.
fn sentinel_check(template: &str, values: &[&str]) -> Option<String> {
    let data = sentinel_data(&serde_json::json!({ "lang": "fr" }), "cv").expect("données sentinelles");
    let (root, fonts) = dirs();
    let values: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    sentinel_problem(compile_template(template.to_string(), data, root, fonts), &values)
}

#[test]
fn texte_en_dur_sans_valeur_accepte() {
    // Une mention de la référence sans champ reste en dur, même conditionnée.
    let template = "#let data = json(\"data.json\")\n\
        #if data.summary != \"\" {\n\
          text(weight: \"bold\", \"Stage de fin d'études de 6 mois, disponible dès mars 2027\")\n\
          linebreak()\n\
          text(data.summary)\n\
        }\n\
        #text(\"Références disponibles sur demande\")\n";
    assert_eq!(sentinel_check(template, &["Camille", "Thyez"]), None);
}

#[test]
fn valeur_en_dur_refusee_meme_conditionnee() {
    let template = "#let data = json(\"data.json\")\n\
        #if data.summary != \"\" { text(\"CAMILLE EXEMPLE\"); text(data.summary) }\n\
        #text(\"Résidant à Thyez\")\n";
    let problem = sentinel_check(template, &["Camille", "Thyez", "Lyon"]).expect("valeurs en dur détectées");
    assert!(problem.contains("\"Camille\"") && problem.contains("\"Thyez\""), "problème : {problem}");
    assert!(!problem.contains("\"Lyon\""), "problème : {problem}");
}

#[test]
fn oscillation_signalee_au_retour_des_memes_problemes() {
    let mut previous = Vec::new();
    let first = flag_oscillation(vec!["b".into(), "a".into()], &mut previous);
    assert!(!first.contains(&OSCILLATION_HINT.to_string()));
    let second = flag_oscillation(vec!["c".into()], &mut previous);
    assert!(!second.contains(&OSCILLATION_HINT.to_string()));
    let third = flag_oscillation(vec!["a".into(), "b".into()], &mut previous);
    assert_eq!(third.last().map(String::as_str), Some(OSCILLATION_HINT));
}

// ======================================================================
// Non-régression : références réelles, template conforme écrit à la main
// ======================================================================

/// Lit un fichier de `tests/fixtures/references/real/`.
fn read_real(name: &str) -> String {
    let path = PathBuf::from(REAL_DIR).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {} illisible : {e}", path.display()))
}

/// Compile la référence, puis passe un template et des données d'exemple
/// dans `validate`, comme une sortie du LLM.
///
/// # Returns
///
/// Problèmes relevés (vide si le template est accepté).
async fn validate_real(reference: &str, template: String, example_data: serde_json::Value, doc_type: &str) -> Vec<String> {
    match validate_real_outcome(reference, template, example_data, doc_type).await {
        Validation::Accepted { .. } => Vec::new(),
        Validation::Rejected { problems, .. } => problems,
    }
}

/// Comme `validate_real`, mais renvoie l'issue complète de la validation.
async fn validate_real_outcome(
    reference: &str,
    template: String,
    example_data: serde_json::Value,
    doc_type: &str,
) -> Validation {
    let (root, fonts) = dirs();
    let reference_typ = read_real(reference);
    let compiled = compile_document(reference_typ.clone(), root.clone(), fonts.clone()).expect("la référence compile");
    let example_data_json = serde_json::to_string_pretty(&example_data).expect("example_data sérialisable");
    let output = TemplatizeOutput { template_typ: template, example_data };
    let context = ValidationContext {
        reference: &compiled,
        computes_date: reference_typ.contains("datetime.today"),
        doc_type,
        root: &root,
        fonts_dir: &fonts,
    };
    let outcome = validate(&output, &example_data_json, &context).await.expect("validation sans erreur interne");
    if let Validation::Accepted { page_count } = outcome {
        assert!(page_count <= compiled.page_count, "{page_count} page(s), référence : {}", compiled.page_count);
    }
    outcome
}

/// Données d'exemple d'une fixture réelle.
fn real_example(name: &str) -> serde_json::Value {
    serde_json::from_str(&read_real(name)).expect("example_data JSON")
}

#[tokio::test]
async fn cv_reel_template_conforme_accepte() {
    // Rien n'est retiré : la ligne de disponibilité et la mention de fin de
    // page, sans champ dans le schéma, restent en dur dans le template.
    let example = real_example("cv_suisse_example_data.json");
    let problems = validate_real("cv_suisse.typ", read_real("cv_suisse_template.typ"), example, "cv").await;
    assert!(problems.is_empty(), "problèmes :\n- {}", problems.join("\n- "));
}

#[tokio::test]
async fn lettre_reelle_template_conforme_acceptee() {
    // Service et rue du destinataire dans `recipient.extra_lines`, consignes
    // [FIXE] / [VARIABLE] / [ADAPTABLE] dans `guidance`.
    let example = real_example("lm_exemple_example_data.json");
    let problems =
        validate_real("lm_exemple.typ", read_real("lm_exemple_template.typ"), example, "lettre_motivation").await;
    assert!(problems.is_empty(), "problèmes :\n- {}", problems.join("\n- "));
}

#[tokio::test]
async fn cv_reel_texte_retire_refuse() {
    // Retirer un texte de la référence fait échouer la comparaison.
    let template = read_real("cv_suisse_template.typ")
        .replace("#text(size: 9pt, fill: gray, style: \"italic\", \"Références disponibles sur demande\")", "");
    let example = real_example("cv_suisse_example_data.json");
    let problems = validate_real("cv_suisse.typ", template, example, "cv").await;
    assert!(problems.iter().any(|p| p.contains("\"Références\"")), "problèmes : {problems:?}");
}

#[tokio::test]
async fn cv_reel_nom_en_dur_refuse() {
    // Le nom écrit en dur ne serait jamais remplacé par celui du profil.
    let template = read_real("cv_suisse_template.typ").replace(
        "upper(joined((data.identity.prenom, data.identity.nom), \" \"))",
        "\"CAMILLE EXEMPLE\"",
    );
    let example = real_example("cv_suisse_example_data.json");
    let problems = validate_real("cv_suisse.typ", template, example, "cv").await;
    assert!(problems.iter().any(|p| p.contains("marqueur unique") && p.contains("\"Camille\"")), "problèmes : {problems:?}");
}

#[tokio::test]
async fn lettre_reelle_consignes_incoherentes_refusees() {
    // Une zone qui ne redonne pas son paragraphe est signalée.
    let mut example = real_example("lm_exemple_example_data.json");
    example["guidance"]["zones"][6]["text"] = serde_json::Value::String("Résidant à Thyez.".to_string());
    let problems =
        validate_real("lm_exemple.typ", read_real("lm_exemple_template.typ"), example, "lettre_motivation").await;
    assert!(problems.iter().any(|p| p.contains("paragraphs[3]")), "problèmes : {problems:?}");
}

#[tokio::test]
async fn cv_reel_separateur_de_niveau_orphelin_designe_et_tolere() {
    // Le cas remonté : « : » affiché même quand le niveau de langue est vide.
    let template = read_real("cv_suisse_template.typ")
        .replace("if l.level != \"\" { l.nom + \" : \" } else { l.nom }", "l.nom + \" : \"");
    let example = real_example("cv_suisse_example_data.json");
    match validate_real_outcome("cv_suisse.typ", template, example, "cv").await {
        Validation::Rejected { problems, tolerable } => {
            assert!(problems.iter().any(|p| p.contains("`level`") && p.contains(": |")), "problèmes : {problems:?}");
            assert!(tolerable.is_some(), "seuls des orphelins : template utilisable en dernier recours");
        }
        Validation::Accepted { .. } => panic!("séparateur orphelin non détecté"),
    }
}

// ======================================================================
// Structure en paragraphes de la lettre
// ======================================================================

/// Boucle des paragraphes du template réel de la lettre.
const REAL_LM_LOOP: &str = "#for p in data.paragraphs {\n  p\n  parbreak()\n}";

/// Valide le template réel de la lettre avec une autre boucle de paragraphes.
async fn validate_real_letter_loop(paragraphs_loop: &str) -> Vec<String> {
    // Fins de ligne normalisées : la fixture peut être extraite en CRLF.
    let template = read_real("lm_exemple_template.typ").replace("\r\n", "\n");
    assert!(template.contains(REAL_LM_LOOP), "boucle des paragraphes introuvable dans le template réel");
    let example = real_example("lm_exemple_example_data.json");
    validate_real("lm_exemple.typ", template.replace(REAL_LM_LOOP, paragraphs_loop), example, "lettre_motivation").await
}

#[tokio::test]
async fn lettre_reelle_paragraphes_colles_refuses() {
    // Le cas remonté : « SA.Lors », « Docker).Intéressé », « international.Résidant ».
    let problems = validate_real_letter_loop("#for p in data.paragraphs [#p]").await;
    for i in 1..4 {
        let expected = format!("paragraphs[{}] et paragraphs[{i}] sont rendus sans séparateur", i - 1);
        assert!(problems.iter().any(|p| p.contains(&expected)), "frontière {i} : {problems:?}");
    }
    assert!(problems.iter().any(|p| p.contains("\"SA.Lors\" colle \"SA.\" et \"Lors\"")), "{problems:?}");
}

#[tokio::test]
async fn lettre_reelle_en_un_seul_bloc_refusee() {
    // Le texte est identique mot pour mot : seul le contrôle structurel le voit.
    let problems = validate_real_letter_loop("#data.paragraphs.join(\" \")\n").await;
    assert!(problems.iter().any(|p| p.contains("est rendu à la suite de paragraphs")), "{problems:?}");
}

#[test]
fn mot_colle_designe_ses_deux_mots() {
    let missing = ["chez", "SA.", "Lors", "de"].map(String::from);
    let problems = glued_words(&missing, &["SA.Lors".to_string(), "autre".to_string()]);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("\"SA.Lors\" colle \"SA.\" et \"Lors\""), "{}", problems[0]);
}
