//! Tests : lecture déterministe du balisage des références et rattachement
//! des zones aux champs de prose.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use findmyjob_lib::documents::markup::{attach_zones, parse_markup, MarkupError};
use findmyjob_lib::models::{LmExampleData, ProseField, ProseZone, ZoneMode};

/// Champs de prose (objet, paragraphes) des données d'exemple de la lettre réelle.
fn letter_fields() -> (Vec<ProseField>, LmExampleData) {
    let example: LmExampleData =
        serde_json::from_str(&read_reference("real/lm_exemple_example_data.json")).expect("données d'exemple");
    let mut fields = vec![ProseField { path: "subject".into(), text: example.data.subject.clone() }];
    fields.extend(
        example.data.paragraphs.iter().enumerate().map(|(i, p)| ProseField { path: format!("paragraphs[{i}]"), text: p.clone() }),
    );
    (fields, example)
}

/// Zone sans sa consigne, pour comparer le découpage.
fn shape(zone: &ProseZone) -> (String, ZoneMode, String) {
    (zone.field.clone(), zone.mode, zone.text.clone())
}

/// Lignes des erreurs d'un balisage refusé.
fn error_lines(source: &str) -> Vec<usize> {
    parse_markup(source).expect_err("balisage refusé").iter().map(|e: &MarkupError| e.line).collect()
}

#[test]
fn lettre_balisee_redonne_les_zones_de_la_reference() {
    let markup = parse_markup(&read_reference("real/lm_balisee.typ")).expect("balisage valide");
    assert!(markup.general.starts_with("- Toutes les valeurs « Exemple »"));
    assert!(markup.general.contains("La lettre doit tenir sur une page."));
    assert_eq!(markup.zones.len(), 9);

    let (fields, example) = letter_fields();
    let guidance = attach_zones(&markup.general, &markup.zones, &fields).expect("zones rattachées");
    let expected: Vec<_> = example.guidance.zones.iter().map(shape).collect();
    assert_eq!(guidance.zones.iter().map(shape).collect::<Vec<_>>(), expected);
    // Consigne sur plusieurs lignes, recollée ligne à ligne.
    let adaptable = guidance.zones.iter().find(|z| z.mode == ZoneMode::Adaptable).expect("zone adaptable");
    assert!(adaptable.instruction.starts_with("Choisir et ordonner"));
    assert!(adaptable.instruction.ends_with("n'est pas nommé."));
}

#[test]
fn cv_balise_decoupe_titre_et_resume() {
    let markup = parse_markup(&read_reference("real/cv_balise.typ")).expect("balisage valide");
    let fields = vec![
        ProseField { path: "identity.title".into(), text: "Étudiant Ingénieur Data Engineering & Machine Learning".into() },
        ProseField {
            path: "summary".into(),
            text: "Étudiant ingénieur en dernière année à l'Efrei Paris (niveau Master), spécialisé en Data \
                   Engineering et Machine Learning. 6 mois d'expérience en entreprise sur le forecasting de séries \
                   temporelles à grande échelle et des systèmes RAG en production, de la donnée brute jusqu'à l'API."
                .into(),
        },
    ];
    let guidance = attach_zones(&markup.general, &markup.zones, &fields).expect("zones rattachées");
    let modes: Vec<(&str, ZoneMode)> = guidance.zones.iter().map(|z| (z.field.as_str(), z.mode)).collect();
    assert_eq!(
        modes,
        [("identity.title", ZoneMode::Variable), ("summary", ZoneMode::Fixe), ("summary", ZoneMode::Adaptable)]
    );
    assert!(markup.visible_texts.iter().any(|t| t.text == "Références disponibles sur demande"));
    assert!(markup.visible_texts.iter().any(|t| t.text == "Stage de fin d'études de 6 mois, disponible dès mars 2027"));
    // Les arguments nommés (police, couleurs) ne sont pas du texte visible.
    assert!(!markup.visible_texts.iter().any(|t| t.text.contains("Arimo") || t.text.contains("2E5A88")));
}

#[test]
fn prose_avant_la_premiere_balise_est_variable() {
    let source = "Bonjour à tous, voici ma lettre.\n// [FIXE]\nCette phrase ne change jamais.\n";
    let markup = parse_markup(source).expect("balisage valide");
    let fields = [ProseField {
        path: "paragraphs[0]".into(),
        text: "Bonjour à tous, voici ma lettre. Cette phrase ne change jamais.".into(),
    }];
    let guidance = attach_zones("", &markup.zones, &fields).expect("zones rattachées");
    let modes: Vec<ZoneMode> = guidance.zones.iter().map(|z| z.mode).collect();
    assert_eq!(modes, [ZoneMode::Variable, ZoneMode::Fixe]);
    assert_eq!(guidance.zones[0].text, "Bonjour à tous, voici ma lettre.");
}

#[test]
fn balise_hors_debut_de_commentaire_refusee() {
    assert_eq!(error_lines("Texte.\n// --- Corps --- [FIXE]\nAutre texte.\n"), [2]);
}

#[test]
fn balise_dans_un_commentaire_de_bloc_refusee() {
    assert_eq!(error_lines("/* [VARIABLE] */\nTexte.\n"), [1]);
}

#[test]
fn second_bloc_de_consignes_refuse() {
    assert_eq!(error_lines("// [CONSIGNES]\n// Une page.\n\n// [consignes] Encore.\n\nTexte.\n"), [4]);
}

#[test]
fn balise_sans_texte_refusee() {
    assert_eq!(error_lines("// [ADAPTABLE] Rien ne suit.\n#v(8pt)\n"), [1]);
}

#[test]
fn zone_variable_hors_prose_refusee_avec_sa_ligne() {
    let source = "// [VARIABLE] Le destinataire.\nExemple SA\n\n// [FIXE]\nMadame, Monsieur,\n\nCorps de la lettre.\n";
    let markup = parse_markup(source).expect("syntaxe valide");
    let fields = [ProseField { path: "paragraphs[0]".into(), text: "Corps de la lettre.".into() }];
    let errors = attach_zones("", &markup.zones, &fields).expect_err("zone hors prose");
    // La zone [FIXE] sur la formule de politesse est sans effet, pas une erreur.
    assert_eq!(errors.iter().map(|e| e.line).collect::<Vec<_>>(), [1]);
}

#[test]
fn consignes_generales_jusqu_a_la_premiere_ligne_de_code() {
    let source = "// [CONSIGNES] Une page.\n// Ton sobre.\n#set text(lang: \"fr\")\n// Commentaire libre.\nTexte.\n";
    let markup = parse_markup(source).expect("balisage valide");
    assert_eq!(markup.general, "Une page.\nTon sobre.");
}
