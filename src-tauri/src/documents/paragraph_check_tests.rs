//! Tests du contrôle structurel des paragraphes, sur des références
//! synthétiques réellement compilées : séparateurs différents, avec et sans
//! balises de consignes, deux zones dans un même paragraphe, nombre de
//! paragraphes variable.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use super::*;
use crate::documents::typst::{compile_document, compile_template};

/// Paragraphes de test : longs (plusieurs lignes) et courts (une ligne).
const TEXTS: [&str; 5] = [
    "Actuellement étudiant en dernière année du cycle ingénieur, je suis à la recherche d'un stage de fin \
     d'études de six mois à partir de mars, dans une équipe qui construit ses plateformes de données.",
    "Lors de mon stage chez Exemple, j'ai conçu un pipeline de prévision sur plusieurs milliers de séries \
     temporelles et développé les services qui mettent ces modèles en production.",
    "Je suis disponible dès le printemps.",
    "Intéressé par la fiabilité des données, je souhaite rejoindre votre équipe afin de contribuer à la \
     construction de sa plateforme, et mettre mes compétences au service de vos projets.",
    "Je reste à votre disposition pour un entretien.",
];
/// Préambule commun : page étroite pour que les paragraphes longs passent
/// sur plusieurs lignes, texte justifié comme une lettre usuelle.
const PREAMBLE: &str = "#set page(width: 12cm, height: auto, margin: 1cm)\n#set par(justify: true)\n";
const SALUTATION: &str = "Madame, Monsieur,";
const CLOSING: &str = "Veuillez agréer mes salutations distinguées.";

/// Façon dont une référence sépare ses paragraphes.
#[derive(Clone, Copy, Debug)]
enum Separator {
    BlankLine,
    VSpace,
    Backslash,
}

impl Separator {
    /// Séparateur dans le markup de la référence.
    fn markup(self) -> &'static str {
        match self {
            Separator::BlankLine => "\n\n",
            Separator::VSpace => "\n#v(14pt)\n",
            Separator::Backslash => " \\\n",
        }
    }

    /// Boucle correcte d'un template qui reproduit ce séparateur.
    fn correct_loop(self) -> &'static str {
        match self {
            Separator::BlankLine => "#for p in data.paragraphs { p; parbreak() }",
            Separator::VSpace => "#for (i, p) in data.paragraphs.enumerate() { if i > 0 { v(14pt) }; p }",
            Separator::Backslash => "#data.paragraphs.join(linebreak())",
        }
    }
}

/// Construit une référence : formule d'appel, paragraphes, formule de
/// politesse, séparés par `separator`. Avec `tags`, chaque paragraphe est
/// précédé d'une balise ; une `zone_split` coupe ce paragraphe en deux zones
/// séparées seulement par un commentaire.
fn reference(separator: Separator, texts: &[&str], tags: bool, zone_split: Option<usize>) -> String {
    let body: Vec<String> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let text = match zone_split.filter(|&k| k == i) {
                Some(_) => split_in_two(text).join("\n// [VARIABLE] Deuxième zone.\n"),
                None => text.to_string(),
            };
            if tags { format!("// [FIXE]\n{text}") } else { text }
        })
        .collect();
    format!("{PREAMBLE}{SALUTATION}\n\n{}\n\n{CLOSING}\n", body.join(separator.markup()))
}

/// Template qui rend les paragraphes avec `paragraphs_markup`.
fn template(paragraphs_markup: &str) -> String {
    format!("#let data = json(\"data.json\")\n{PREAMBLE}{SALUTATION}\n\n{paragraphs_markup}\n\n{CLOSING}\n")
}

/// Coupe un texte en deux moitiés, à l'espace la plus proche du milieu.
fn split_in_two(text: &str) -> [String; 2] {
    let words: Vec<&str> = text.split_whitespace().collect();
    let half = words.len() / 2;
    [words[..half].join(" "), words[half..].join(" ")]
}

/// Dossier racine sans fichier annexe, et dossier de polices inexistant.
fn dirs() -> (PathBuf, PathBuf) {
    (std::env::temp_dir(), std::env::temp_dir().join("findmyjob-no-fonts"))
}

/// Compile la référence et le template, puis lance le contrôle.
fn check(reference_typ: &str, template_typ: &str, paragraphs: &[String]) -> Vec<String> {
    let (root, fonts) = dirs();
    let reference = compile_document(reference_typ.to_string(), root.clone(), fonts.clone()).expect("la référence compile");
    let data = serde_json::to_vec(&serde_json::json!({ "paragraphs": paragraphs })).expect("données sérialisables");
    let rendered = compile_template(template_typ.to_string(), data, root, fonts).expect("le template compile");
    paragraph_problems(&reference.lines, &rendered.lines, paragraphs)
}

/// Les entrées attendues : un paragraphe par entrée.
fn entries(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|t| t.to_string()).collect()
}

/// Cas couverts : chaque séparateur, avec et sans balises, 2 et 5
/// paragraphes (3 dans les tests de zones).
fn cases() -> Vec<(Separator, bool, &'static [&'static str])> {
    let mut cases = Vec::new();
    for separator in [Separator::BlankLine, Separator::VSpace, Separator::Backslash] {
        for tags in [false, true] {
            cases.push((separator, tags, &TEXTS[..2]));
            cases.push((separator, tags, &TEXTS[..]));
        }
    }
    cases
}

#[test]
fn template_correct_accepte() {
    for (separator, tags, texts) in cases() {
        let problems = check(&reference(separator, texts, tags, None), &template(separator.correct_loop()), &entries(texts));
        assert!(problems.is_empty(), "{separator:?}, balises {tags}, {} paragraphes : {problems:?}", texts.len());
    }
}

#[test]
fn deux_zones_dans_un_paragraphe_acceptees_en_une_entree() {
    for separator in [Separator::BlankLine, Separator::VSpace, Separator::Backslash] {
        let reference_typ = reference(separator, &TEXTS[..3], true, Some(1));
        let problems = check(&reference_typ, &template(separator.correct_loop()), &entries(&TEXTS[..3]));
        assert!(problems.is_empty(), "{separator:?} : {problems:?}");
    }
}

#[test]
fn paragraphes_colles_refuses_avec_feedback_cible() {
    for (separator, tags, texts) in cases() {
        let problems =
            check(&reference(separator, texts, tags, None), &template("#for p in data.paragraphs [#p]"), &entries(texts));
        for i in 1..texts.len() {
            let expected = format!("paragraphs[{}] et paragraphs[{i}] sont rendus sans séparateur", i - 1);
            assert!(
                problems.iter().any(|p| p.contains(&expected)),
                "{separator:?}, balises {tags}, frontière {i} : {problems:?}"
            );
        }
    }
}

#[test]
fn paragraphes_joints_en_un_seul_bloc_refuses() {
    for (separator, tags, texts) in cases() {
        let problems =
            check(&reference(separator, texts, tags, None), &template("#data.paragraphs.join(\" \")"), &entries(texts));
        assert!(
            problems.iter().any(|p| p.contains("est rendu à la suite de paragraphs")),
            "{separator:?}, balises {tags}, {} paragraphes : {problems:?}",
            texts.len()
        );
    }
}

#[test]
fn lettre_en_une_seule_entree_refusee() {
    for (separator, tags, texts) in cases() {
        let single = vec![texts.join(" ")];
        let problems = check(&reference(separator, texts, tags, None), &template(separator.correct_loop()), &single);
        let expected = format!("compte {} paragraphe(s), mais paragraphs a 1 entrée(s)", texts.len());
        assert!(problems.iter().any(|p| p.contains(&expected)), "{separator:?}, balises {tags} : {problems:?}");
    }
}

#[test]
fn decoupage_par_zones_refuse() {
    for separator in [Separator::BlankLine, Separator::VSpace, Separator::Backslash] {
        let reference_typ = reference(separator, &TEXTS[..3], true, Some(1));
        let [first_zone, second_zone] = split_in_two(TEXTS[1]);
        let by_zones = vec![TEXTS[0].to_string(), first_zone, second_zone, TEXTS[2].to_string()];
        let problems = check(&reference_typ, &template(separator.correct_loop()), &by_zones);
        assert!(
            problems.iter().any(|p| p.contains("paragraphs[2] commence au milieu d'un paragraphe de la référence")),
            "{separator:?} : {problems:?}"
        );
        assert!(problems.iter().any(|p| p.contains("compte 3 paragraphe(s), mais paragraphs a 4")), "{separator:?} : {problems:?}");
    }
}

#[test]
fn espacement_different_de_la_reference_refuse() {
    let reference_typ = reference(Separator::BlankLine, &TEXTS[..3], false, None);
    let problems = check(&reference_typ, &template(Separator::Backslash.correct_loop()), &entries(&TEXTS[..3]));
    assert!(problems.iter().any(|p| p.contains("l'espacement vertical rendu")), "{problems:?}");
}

#[test]
fn corps_introuvable_ne_signale_rien() {
    // Texte des entrées absent de la référence : l'écart de texte le signale déjà.
    let reference_typ = reference(Separator::BlankLine, &TEXTS[..2], false, None);
    let other = vec!["Rien à voir.".to_string(), "Toujours rien.".to_string()];
    assert!(check(&reference_typ, &template(Separator::BlankLine.correct_loop()), &other).is_empty());
}
