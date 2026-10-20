//! Tests : validation des deux étapes de la templatisation sur les
//! références réelles (anonymisées et balisées), avec des sorties écrites à
//! la main : données d'exemple (étape A) et template (étape B).

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::example_check::{check_example, CheckedExample, ExampleContext};
use findmyjob_lib::documents::markup::parse_markup;
use findmyjob_lib::documents::template_check::{check_template, TemplateContext, TemplateVerdict};
use findmyjob_lib::documents::typst::{compile_document, CompiledDocument};
use findmyjob_lib::models::ZoneMode;

/// Référence réelle compilée, avec son dossier de compilation.
struct Reference {
    source: String,
    rendered: CompiledDocument,
    doc_type: &'static str,
    root: PathBuf,
    fonts: PathBuf,
}

/// Compile une référence de `references/real/`.
fn reference(name: &str, doc_type: &'static str) -> Reference {
    let root = unique_temp_dir("templatize-validation");
    let fonts = root.join("polices");
    let source = read_reference(&format!("real/{name}"));
    let rendered = compile_document(source.clone(), root.clone(), fonts.clone()).expect("la référence compile");
    Reference { source, rendered, doc_type, root, fonts }
}

/// Valide des données d'exemple (étape A).
fn check(reference: &Reference, example: &serde_json::Value) -> Result<CheckedExample, Vec<String>> {
    let markup = parse_markup(&reference.source).expect("balisage valide");
    let context = ExampleContext {
        doc_type: reference.doc_type,
        reference_text: &reference.rendered.text,
        computes_date: false,
        markup: &markup,
    };
    check_example(example, &context)
}

/// Données d'exemple d'une fixture réelle.
fn example(name: &str) -> serde_json::Value {
    serde_json::from_str(&read_reference(&format!("real/{name}"))).expect("données d'exemple JSON")
}

/// Valide un template (étape B) avec les données d'exemple de la fixture.
async fn verdict(reference: &Reference, example_name: &str, template: &str) -> TemplateVerdict {
    let checked = check(reference, &example(example_name)).expect("données d'exemple valides");
    let context = TemplateContext {
        reference: &reference.rendered,
        computes_date: false,
        doc_type: reference.doc_type,
        root: &reference.root,
        fonts_dir: &reference.fonts,
    };
    check_template(template, &checked, &context).await.expect("validation sans erreur interne")
}

/// Template d'une fixture réelle, fins de ligne normalisées.
fn template(name: &str) -> String {
    read_reference(&format!("real/{name}")).replace("\r\n", "\n")
}

// ======================================================================
// Étape A : données d'exemple
// ======================================================================

#[test]
fn lettre_reelle_donnees_textes_et_consignes() {
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let checked = check(&lm, &example("lm_balisee_example_data.json")).expect("données valides");
    assert_eq!(checked.lang, "fr");
    assert_eq!(checked.texts.values().collect::<Vec<_>>(), ["P.J. : Curriculum vitae"]);
    assert_eq!(checked.paragraphs.len(), 4);
    let fixed = checked.guidance.zones.iter().filter(|z| z.mode == ZoneMode::Fixe).count();
    assert_eq!(fixed, 3, "zones [FIXE] des paragraphes ; celles des formules sont sans effet");
    assert_eq!(checked.example["texts"]["t1"], "P.J. : Curriculum vitae", "textes rangés dans les données");
}

#[test]
fn cv_reel_textes_en_dur_identifies() {
    let cv = reference("cv_balise.typ", "cv");
    let checked = check(&cv, &example("cv_balise_example_data.json")).expect("données valides");
    let texts: Vec<&String> = checked.texts.values().collect();
    assert_eq!(texts, ["Stage de fin d'études de 6 mois, disponible dès mars 2027", "Références disponibles sur demande"]);
}

#[test]
fn valeur_absente_de_l_original_refusee() {
    // Le cas remonté : une ville qui n'est pas celle affichée.
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let mut data = example("lm_balisee_example_data.json");
    data["identity"]["ville"] = "Genève Centre".into();
    let problems = check(&lm, &data).err().expect("valeur inventée refusée");
    assert!(problems.iter().any(|p| p.contains("identity.ville")), "{problems:?}");
}

#[test]
fn ville_du_destinataire_dans_les_lignes_supplementaires_refusee() {
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let mut data = example("lm_balisee_example_data.json");
    data["recipient"]["location"] = "".into();
    data["recipient"]["extra_lines"] =
        serde_json::json!(["Service des ressources humaines", "Rue de l'Exemple 1", "1201 Genève"]);
    let problems = check(&lm, &data).err().expect("lieu mal rangé refusé");
    assert!(problems.iter().any(|p| p.contains("recipient.location")), "{problems:?}");
}

#[test]
fn tous_les_problemes_rapportes_ensemble() {
    let cv = reference("cv_balise.typ", "cv");
    let mut data = example("cv_balise_example_data.json");
    data["lang"] = "de".into();
    data["labels"]["months_short"] = serde_json::json!(["01"]);
    data["identity"]["telephone"] = "+41 00 000 00 00".into();
    let problems = check(&cv, &data).err().expect("données refusées");
    assert!(problems.iter().any(|p| p.starts_with("lang")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("months_short")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("identity.telephone")), "{problems:?}");
}

// ======================================================================
// Étape B : template
// ======================================================================

#[tokio::test]
async fn cv_reel_template_conforme_accepte() {
    let cv = reference("cv_balise.typ", "cv");
    let verdict = verdict(&cv, "cv_balise_example_data.json", &template("cv_balise_template.typ")).await;
    assert!(verdict.problems.is_empty(), "problèmes :\n- {}", verdict.problems.join("\n- "));
    assert_eq!(verdict.page_count, Some(cv.rendered.page_count));
}

#[tokio::test]
async fn lettre_reelle_template_conforme_acceptee() {
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let verdict = verdict(&lm, "lm_balisee_example_data.json", &template("lm_balisee_template.typ")).await;
    assert!(verdict.problems.is_empty(), "problèmes :\n- {}", verdict.problems.join("\n- "));
}

#[tokio::test]
async fn texte_de_la_reference_ecrit_en_dur_refuse() {
    let cv = reference("cv_balise.typ", "cv");
    let template = template("cv_balise_template.typ").replace("data.texts.t2", "\"Références disponibles sur demande\"");
    let verdict = verdict(&cv, "cv_balise_example_data.json", &template).await;
    let expected = |p: &String| p.contains("« Références disponibles sur demande »") && p.contains("data.texts.t2");
    assert!(verdict.problems.iter().any(expected), "{:?}", verdict.problems);
}

#[tokio::test]
async fn texte_retire_situe_dans_sa_phrase() {
    let cv = reference("cv_balise.typ", "cv");
    let template = template("cv_balise_template.typ")
        .replace("#text(size: 9pt, fill: gray, style: \"italic\", data.texts.t2)", "");
    let verdict = verdict(&cv, "cv_balise_example_data.json", &template).await;
    let diff = verdict.problems.iter().find(|p| p.contains("diffère de l'original")).expect("écart signalé");
    assert!(diff.contains("absent du rendu : « Références disponibles sur demande »"), "{diff}");
    assert!(diff.contains("mot "), "position citée : {diff}");
}

#[tokio::test]
async fn nom_en_dur_refuse() {
    let cv = reference("cv_balise.typ", "cv");
    let template = template("cv_balise_template.typ")
        .replace("upper(joined((data.identity.prenom, data.identity.nom), \" \"))", "\"CAMILLE EXEMPLE\"");
    let verdict = verdict(&cv, "cv_balise_example_data.json", &template).await;
    let expected = |p: &String| p.contains("écrites en dur") && p.contains("« Camille »");
    assert!(verdict.problems.iter().any(expected), "{:?}", verdict.problems);
}

#[tokio::test]
async fn code_rendu_en_texte_nomme_le_mode_markup() {
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let template =
        template("lm_balisee_template.typ").replace("#v(16pt)\n", "#v(16pt)\nif data.subject != \"\" { data.subject }\n");
    let verdict = verdict(&lm, "lm_balisee_example_data.json", &template).await;
    let expected = |p: &String| p.contains("code Typst comme du texte") && p.contains("mode markup");
    assert!(verdict.problems.iter().any(expected), "{:?}", verdict.problems);
}

#[tokio::test]
async fn erreur_de_compilation_citee_une_seule_fois_avec_indice() {
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let template = template("lm_balisee_template.typ")
        .replace("  if data.recipient.company != \"\"", "  #if data.recipient.company != \"\"");
    let verdict = verdict(&lm, "lm_balisee_example_data.json", &template).await;
    let compile: Vec<&String> = verdict.problems.iter().filter(|p| p.starts_with("Le template ne compile pas")).collect();
    assert_eq!(compile.len(), 1, "une même erreur pour les 4 rendus : {compile:?}");
    assert!(compile[0].contains("exemple, vides"), "rendus concernés listés : {}", compile[0]);
    assert!(compile[0].contains("Indice"), "indice sur le mode code : {}", compile[0]);
}

#[tokio::test]
async fn paragraphes_colles_refuses() {
    let lm = reference("lm_balisee.typ", "lettre_motivation");
    let template = template("lm_balisee_template.typ")
        .replace("#for p in data.paragraphs {\n  p\n  parbreak()\n}", "#for p in data.paragraphs [#p]");
    let verdict = verdict(&lm, "lm_balisee_example_data.json", &template).await;
    let expected = |p: &String| p.contains("« SA.Lors » colle « SA. » et « Lors »");
    assert!(verdict.problems.iter().any(expected), "{:?}", verdict.problems);
}

#[tokio::test]
async fn separateur_orphelin_designe_et_non_bloquant() {
    let cv = reference("cv_balise.typ", "cv");
    let template = template("cv_balise_template.typ")
        .replace("if l.level != \"\" { l.nom + \" : \" } else { l.nom }", "l.nom + \" : \"");
    let verdict = verdict(&cv, "cv_balise_example_data.json", &template).await;
    assert!(verdict.problems.iter().any(|p| p.contains("`level`")), "{:?}", verdict.problems);
    assert_eq!(verdict.blocking, 0, "seuls des orphelins : template utilisable en dernier recours");
}
