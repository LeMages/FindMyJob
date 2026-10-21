//! Tests du linter de la prose générée (`findmyjob_lib::docgen::lint`) et
//! de la normalisation des tirets (`docgen::dash_normalization`,
//! `docgen::data_normalization`).

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use crate::common;
use findmyjob_lib::docgen::budgets::{cv_budget, lm_budget};
use findmyjob_lib::docgen::dash_normalization::{
    dash_marks, normalize_application_message, normalize_dashes, DashContext, DashMark,
};
use findmyjob_lib::docgen::data_normalization::normalize_document_data;
use findmyjob_lib::docgen::lint::{lint_cv_budget, lint_lm_budget, lint_prose, strip_dashes};
use findmyjob_lib::models::{CvContent, LmContent, ProseField, SkillsGroup};

/// Champ de prose de test.
fn field(path: &str, text: &str) -> ProseField {
    ProseField { path: path.to_string(), text: text.to_string() }
}

/// Texte de `n` mots.
fn words(n: usize) -> String {
    vec!["mot"; n].join(" ")
}

#[test]
fn caracteres_interdits_releves_par_champ() {
    let fields = [
        field("a", "Conçu un pipeline — en production"),
        field("b", "Période 2024 – 2025"),
        field("c", "Un **gras** et un `code`"),
        field("d", "Texte propre, sans marque."),
    ];
    let violations = lint_prose(&fields, "");
    let paths: Vec<&str> = violations.iter().map(|v| v.field.as_str()).collect();
    assert_eq!(paths, vec!["a", "b", "c", "c"]);
}

#[test]
fn formule_interdite_sauf_si_presente_dans_l_exemple() {
    let fields = [field("p", "Une stack directement transférable au poste.")];
    assert_eq!(lint_prose(&fields, "").len(), 1);
    assert!(lint_prose(&fields, "Mon expérience est Directement Transférable.").is_empty());
}

#[test]
fn formule_anglaise_detectee() {
    let fields = [field("p", "This experience aligns with your needs.")];
    assert_eq!(lint_prose(&fields, "").len(), 1);
}

#[test]
fn budget_cv_respecte_sans_violation() {
    let budget = cv_budget(&common::cv_example(), 1);
    let content = CvContent {
        title: "Titre".to_string(),
        summary: words(10),
        skills_groups: vec![
            SkillsGroup { category: "A".to_string(), skills: vec!["Rust".to_string()] },
            SkillsGroup { category: "B".to_string(), skills: vec!["SQL".to_string()] },
        ],
    };
    assert!(lint_cv_budget(&content, &budget).is_empty(), "{:?}", lint_cv_budget(&content, &budget));
}

#[test]
fn budget_cv_depasse_releve_chaque_ecart() {
    let budget = cv_budget(&common::cv_example(), 1);
    let content = CvContent { title: "Titre".to_string(), summary: words(30), skills_groups: Vec::new() };
    let fields: Vec<String> = lint_cv_budget(&content, &budget).into_iter().map(|v| v.field).collect();
    assert_eq!(fields, vec!["summary".to_string(), "skills_groups".to_string()]);
}

#[test]
fn budget_lettre_nombre_de_paragraphes_et_mots() {
    let budget = lm_budget(&common::lm_example(), 1);
    let ok = LmContent { subject: words(6), paragraphs: vec![words(20), words(50), words(10)] };
    assert!(lint_lm_budget(&ok, &budget).is_empty());

    let too_long = LmContent { subject: words(6), paragraphs: vec![words(20), words(80)] };
    let fields: Vec<String> = lint_lm_budget(&too_long, &budget).into_iter().map(|v| v.field).collect();
    assert_eq!(fields, vec!["paragraphs".to_string(), "paragraphs[1]".to_string()]);
}

#[test]
fn remplacement_mecanique_des_tirets() {
    assert_eq!(strip_dashes("Un pipeline — en production — robuste"), "Un pipeline, en production, robuste");
    assert_eq!(strip_dashes("Data–ML et **gras** `x`"), "Data-ML et gras x");
    assert_eq!(strip_dashes("Un pipeline -- en production --- robuste"), "Un pipeline, en production, robuste");
    assert!(lint_prose(&[field("x", &strip_dashes("a — b – c -- d --- e"))], "").is_empty());
}

// ======================================================================
// Tirets ASCII, messages de relance et catégories de compétences
// ======================================================================

#[test]
fn doubles_et_triples_traits_d_union_releves() {
    let fields = [field("a", "Un pipeline -- en production"), field("b", "Un pipeline --- robuste")];
    let violations = lint_prose(&fields, "");
    let paths: Vec<&str> = violations.iter().map(|v| v.field.as_str()).collect();
    assert_eq!(paths, vec!["a", "b"], "« --- » n'est pas relevé une seconde fois comme « -- » : {violations:?}");
    assert!(violations[0].message.contains("« -- »"));
    assert!(violations[1].message.contains("« --- »"));
}

#[test]
fn message_de_relance_avec_marque_et_substitut() {
    let violations = lint_prose(&[field("p", "Un pipeline — robuste")], "");
    assert_eq!(violations.len(), 1);
    let message = &violations[0].message;
    assert!(message.contains("« — »") && message.contains("virgule") && message.contains("parenthèses"), "{message}");
}

#[test]
fn categorie_de_competences_lintee() {
    let content = CvContent {
        title: "Data engineer".to_string(),
        summary: "Résumé.".to_string(),
        skills_groups: vec![SkillsGroup { category: "Data — ML".to_string(), skills: vec!["SQL".to_string()] }],
    };
    let violations = lint_prose(&content.prose_fields(), "");
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].field, "skills_groups[0].category");

    let mut cleaned = content.clone();
    cleaned.map_prose(|_, text| strip_dashes(text));
    assert_eq!(cleaned.skills_groups[0].category, "Data, ML");
}

// ======================================================================
// Normalisation déterministe (docgen::dash_normalization)
// ======================================================================

#[test]
fn normalisation_des_incises_en_paragraphe() {
    let p = |text: &str| normalize_dashes(text, DashContext::Paragraph);
    assert_eq!(p("Un pipeline — en production — robuste."), "Un pipeline, en production, robuste.");
    assert_eq!(p("Un pipeline -- en production -- robuste."), "Un pipeline, en production, robuste.");
    assert_eq!(p("Un pipeline --- robuste."), "Un pipeline, robuste.");
    // Fin d'incise devant une ponctuation, ou après une virgule : jamais de
    // double virgule ni d'espace avant la virgule.
    assert_eq!(p("Un pipeline — en production —."), "Un pipeline, en production.");
    assert_eq!(p("Un pipeline, — en production"), "Un pipeline, en production");
    assert_eq!(p("Un projet (— interne) livré"), "Un projet (interne) livré");
    assert_eq!(p("Un pipeline —en production"), "Un pipeline, en production");
    // Tiret en bord de texte (puce) : retiré.
    assert_eq!(p("— Un pipeline robuste"), "Un pipeline robuste");
    assert_eq!(p("Un pipeline robuste —"), "Un pipeline robuste");
}

#[test]
fn normalisation_des_champs_courts() {
    let s = |text: &str| normalize_dashes(text, DashContext::Short);
    assert_eq!(s("Fondateur — Framework de forecasting"), "Fondateur - Framework de forecasting");
    assert_eq!(s("ALGN – Autonomous Legal Graph Navigator"), "ALGN - Autonomous Legal Graph Navigator");
    assert_eq!(s("Stage -- 6 mois"), "Stage - 6 mois");
    assert_eq!(s("Stage --- 6 mois"), "Stage - 6 mois");
    assert_eq!(s("Data–ML"), "Data-ML");
}

#[test]
fn normalisation_des_intervalles_numeriques() {
    for context in [DashContext::Short, DashContext::Paragraph] {
        assert_eq!(normalize_dashes("2024–2025", context), "2024-2025");
        assert_eq!(normalize_dashes("2024--2025", context), "2024-2025");
        assert_eq!(normalize_dashes("de 2022 – 2027", context), "de 2022 - 2027");
        assert_eq!(normalize_dashes("10–15 %", context), "10-15 %");
    }
}

#[test]
fn normalisation_epargne_les_traits_d_union_et_coordonnees() {
    let preserved = [
        "Jean-Marie Dupont",
        "scikit-learn et Next.js",
        "2024-2025",
        "AI-Powered CRM",
        "Internship - AI Engineer",
        "https://example.com/a--b---c",
        "github.com/foo--bar",
        "www.foo--bar.fr",
        "jean--marie.dupont@example.com",
        "06-12-34-56-78",
        "+33 6 12 34 56 78",
    ];
    for text in preserved {
        for context in [DashContext::Short, DashContext::Paragraph] {
            assert_eq!(normalize_dashes(text, context), text, "{context:?}");
        }
        assert!(dash_marks(text).is_empty(), "{text}");
    }
    assert_eq!(normalize_dashes("Voir https://a.fr/x--y — merci", DashContext::Paragraph), "Voir https://a.fr/x--y, merci");
}

#[test]
fn marques_relevees_une_fois_chacune() {
    assert_eq!(dash_marks("a — b — c"), vec![DashMark::EmDash]);
    assert_eq!(dash_marks("a – b --- c -- d"), vec![DashMark::EnDash, DashMark::TripleHyphen, DashMark::DoubleHyphen]);
    assert_eq!(dash_marks("a --- b"), vec![DashMark::TripleHyphen]);
    assert!(dash_marks("a - b").is_empty());
}

#[test]
fn normalisation_de_toutes_les_chaines_du_data_json() {
    let mut data = json!({
        "identity": { "title": "Data — ML", "email": "a--b@example.com", "linkedin": "linkedin.com/in/a--b" },
        "summary": "Un profil — rigoureux — et curieux.",
        "experiences": [{ "poste": "Fondateur — Framework", "period": "2022 – 2027", "bullets": ["Un pipeline -- robuste."] }],
        "formations": [{ "titre": "Diplôme — Spécialité", "details": ["Échange – Université Concordia"] }],
        "skills_groups": [{ "category": "Data -- ML", "skills": ["SQL"] }],
        "paragraphs": ["Mon stage — en production — a duré six mois."],
        "subject": "Candidature – Stage",
        "texts": { "t1": "P.J. — CV" },
        "labels": { "months_short": ["Janv.–"] }
    });
    normalize_document_data(&mut data);
    assert_eq!(data["identity"]["title"], "Data - ML");
    assert_eq!(data["identity"]["email"], "a--b@example.com");
    assert_eq!(data["identity"]["linkedin"], "linkedin.com/in/a--b");
    assert_eq!(data["summary"], "Un profil, rigoureux, et curieux.");
    assert_eq!(data["experiences"][0]["poste"], "Fondateur - Framework");
    assert_eq!(data["experiences"][0]["period"], "2022 - 2027");
    assert_eq!(data["experiences"][0]["bullets"][0], "Un pipeline, robuste.");
    assert_eq!(data["formations"][0]["titre"], "Diplôme - Spécialité");
    assert_eq!(data["formations"][0]["details"][0], "Échange, Université Concordia");
    assert_eq!(data["skills_groups"][0]["category"], "Data - ML");
    assert_eq!(data["paragraphs"][0], "Mon stage, en production, a duré six mois.");
    assert_eq!(data["subject"], "Candidature - Stage");
    assert_eq!(data["texts"]["t1"], "P.J. - CV");
    assert_eq!(data["labels"]["months_short"][0], "Janv.-");
}

#[test]
fn normalisation_du_message_de_candidature() {
    let message = "Objet : Candidature stage AI Engineer — Loïc Cupif\n\nMadame, Monsieur,\nMon stage — en production — a duré six mois.";
    assert_eq!(
        normalize_application_message(message),
        "Objet : Candidature stage AI Engineer - Loïc Cupif\n\nMadame, Monsieur,\nMon stage, en production, a duré six mois."
    );
}
