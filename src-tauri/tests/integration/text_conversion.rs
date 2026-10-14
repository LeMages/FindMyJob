//! Tests de la conversion des textes d'offres en texte brut
//! (`findmyjob_lib::text`), sur des fixtures d'offres fictives (une en
//! Markdown décoré, une en HTML) et des cas construits (null, listes
//! imbriquées, Markdown décoré).

// Importation des modules standard
use std::collections::HashMap;

// Importation des crates externes
use scraper::Html;

// Importation des modules internes
use findmyjob_lib::text::html_to_plain;

const OFFER_MARKDOWN: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/offer_markdown.txt"));
const OFFER_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/offer_html.txt"));

// ======================================================================
// Helpers
// ======================================================================

/// Compte les mots (suites de caractères alphanumériques Unicode) d'un texte.
fn word_counts(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }
    counts
}

/// Vérifie que chaque mot de la source apparaît au moins autant de fois
/// dans la sortie.
fn assert_no_word_lost(source_text: &str, output: &str) {
    let expected = word_counts(source_text);
    let actual = word_counts(output);
    for (word, count) in &expected {
        let found = actual.get(word).copied().unwrap_or(0);
        assert!(found >= *count, "mot perdu : {word:?} ({found}/{count})\n--- sortie ---\n{output}");
    }
}

/// Texte visible d'un fragment HTML, référence pour la perte de mots.
fn html_visible_text(html: &str) -> String {
    Html::parse_fragment(html).root_element().text().collect::<Vec<_>>().join(" ")
}

fn is_emoji(c: char) -> bool {
    matches!(c as u32, 0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0xFE0F | 0x200D)
}

/// Vérifie l'absence de tout résidu HTML, Markdown ou emoji.
fn assert_clean(text: &str) {
    let bytes = text.as_bytes();
    let has_tag = bytes
        .windows(2)
        .any(|p| p[0] == b'<' && (p[1].is_ascii_alphabetic() || p[1] == b'/'));
    assert!(!has_tag, "balise HTML résiduelle :\n{text}");

    let has_entity = text.match_indices('&').any(|(i, _)| {
        let rest = &text[i + 1..];
        let name_len = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '#').count();
        name_len > 0 && rest[name_len..].starts_with(';')
    });
    assert!(!has_entity, "entité HTML résiduelle :\n{text}");

    assert!(!text.contains('*'), "astérisque Markdown résiduel :\n{text}");
    assert!(!text.contains("__"), "emphase `__` résiduelle :\n{text}");
    assert!(!text.lines().any(|l| l.trim_start().starts_with('#')), "titre `#` résiduel :\n{text}");
    assert!(!text.chars().any(is_emoji), "emoji résiduel :\n{text}");
    assert!(!text.contains("\n\n\n"), "plus d'une ligne vide consécutive :\n{text}");
    assert_eq!(text, text.trim(), "espaces en bord de texte");
}

// ======================================================================
// Fixtures
// ======================================================================

#[test]
fn markdown_fixture_is_cleaned_without_word_loss() {
    let output = html_to_plain(OFFER_MARKDOWN).expect("texte non vide");

    assert_clean(&output);
    assert_no_word_lost(OFFER_MARKDOWN, &output);
    assert!(output.starts_with("Ton rôle\n\n- Étudiant(e) en école d'ingénieur ou université (Bac+4/5)"));
    assert!(output.contains("\n- Curiosité et rigueur, avec une première expérience en Python. SQL est un plus.\n"));
    assert!(output.contains("Ce que nous offrons\n\n- Télétravail partiel\n- Une équipe de douze personnes\n- Des projets variés"));
    assert!(output.contains("\nPrêt(e) à rejoindre l’aventure Société Exemple ? Postule dès maintenant !\n"));
    assert!(output.ends_with("sans discrimination."));
}

#[test]
fn html_fixture_is_cleaned_without_word_loss() {
    let output = html_to_plain(OFFER_HTML).expect("texte non vide");

    assert_clean(&output);
    assert_no_word_lost(&html_visible_text(OFFER_HTML), &output);
    assert!(output.starts_with("Descriptif du poste\n\nAu sein de l'équipe Data de Société Exemple"));
    // `<li><p>…</p></li>` : puces consécutives, sans ligne vide intercalée.
    assert!(output.contains(
        "\n- Participer au cadrage des cas d’usage : qualification et priorisation des besoins.\n\
         - Analyser et structurer la donnée : modélisation, nettoyage.\n\
         - Documenter les traitements pour l'équipe.\n"
    ));
    assert!(output.contains("Data & Analytics, vous êtes curieux(se) et autonome."));
    assert!(output.contains("Vos atouts :\n\n- Curiosité, esprit d’analyse et capacité"));
    assert!(output.contains("\n- Attrait pour l'entreprenariat et l'innovation."));
    assert!(output.contains("\n\nAdditional Information\n\n"));
    assert!(output.ends_with("à partir de janvier 2027.\nBasé à Lyon."));
}

// ======================================================================
// Cas construits
// ======================================================================

#[test]
fn null_and_blank_inputs_become_none() {
    for input in ["null", "NULL", "<p>null</p>", "", "   \n\t", "<p>&#xa0;</p>", "<p></p><p> </p>"] {
        assert_eq!(html_to_plain(input), None, "entrée {input:?}");
    }
}

#[test]
fn nested_lists_are_indented_two_spaces_per_level() {
    let html = "<ul><li>A<ul><li>B<ul><li>C</li></ul></li><li>B2</li></ul></li><li>D</li></ul><ol><li>E</li></ol>";

    assert_eq!(html_to_plain(html).as_deref(), Some("- A\n  - B\n    - C\n  - B2\n- D\n\n- E"));
}

#[test]
fn blocks_breaks_and_entities_are_rendered() {
    let html = "<h2>Missions</h2><p>Ligne&#xa0;1<br>Ligne&nbsp;2 &amp; suite</p><div>Bloc   final</div>";

    assert_eq!(
        html_to_plain(html).as_deref(),
        Some("Missions\n\nLigne 1\nLigne 2 & suite\n\nBloc final")
    );
}

#[test]
fn script_and_style_are_ignored() {
    let html = "<p>Avant</p><script>alert('x')</script><style>p { color: red; }</style><p>Après</p>";

    assert_eq!(html_to_plain(html).as_deref(), Some("Avant\n\nAprès"));
}

#[test]
fn markdown_decorations_headings_and_bullets_are_removed() {
    let markdown = "*** Missions ***\n## Profil **recherché**\n=== Titre ===\n• item\n* autre\n---\n__Gras__ et *italique* et \\- échappé\n#1 en France";

    assert_eq!(
        html_to_plain(markdown).as_deref(),
        Some("Missions\nProfil recherché\nTitre\n- item\n- autre\n\nGras et italique et - échappé\n#1 en France")
    );
}

#[test]
fn leading_emoji_becomes_bullet_and_others_are_removed() {
    let markdown = "✅ Autonomie\n🚀 Ambition 🇫🇷\n- 💡 Idées\nFin ❤️";

    assert_eq!(
        html_to_plain(markdown).as_deref(),
        Some("- Autonomie\n- Ambition\n- Idées\nFin")
    );
}
