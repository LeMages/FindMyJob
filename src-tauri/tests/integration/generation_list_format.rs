//! Tests : format des listes du CV selon la session (`sessions.format_liste`),
//! avec la référence balisée templatisée au format 4 et une compilation
//! Typst réelle. Chaque élément de description est précédé du symbole de la
//! session (« • » ou « - »), l'intro n'en a pas, une suite de ligne passe à
//! la ligne dans le même élément, et une description vide n'affiche aucun
//! symbole orphelin.

// Importation des modules internes
use crate::shared::no_dashes::{assert_no_dash_in_data, assert_no_dash_in_text};
use crate::shared::tagged_references::{app_with_tagged_references, data, generate, render};
use crate::shared::test_state::TestApp;
use findmyjob_lib::db::sessions;
use findmyjob_lib::documents::typst::CompiledDocument;
use findmyjob_lib::models::UpdateSessionListFormatInput;

/// Description d'expérience : intro, élément avec une suite de ligne, élément.
const EXPERIENCE: &str =
    "Équipe de 5 personnes.\n- Pipelines SQL pour le reporting\ndéployés chaque semaine\n- Tests automatisés";

/// Premiers mots des éléments attendus : deux pour l'expérience, un pour
/// la formation (« - Spécialité données » dans le profil de test).
const ITEM_FIRST_WORDS: [&str; 3] = ["Pipelines", "Tests", "Spécialité"];

/// Application dont le profil a une expérience structurée et un projet
/// sans description, dans une session au format demandé.
async fn app_with_format(name: &str, format_liste: &str) -> (TestApp, i64) {
    let (app, sid) = app_with_tagged_references(name).await;
    let pool = &app.state.pool;
    sqlx::query("UPDATE experiences SET description = $1").bind(EXPERIENCE).execute(pool).await.expect("expérience");
    sqlx::query("UPDATE projets SET description = NULL").execute(pool).await.expect("projet");
    let input = UpdateSessionListFormatInput { format_liste: format_liste.into() };
    sessions::update_list_format(pool, sid, &input).await.expect("format de liste");
    (app, sid)
}

/// Lignes rendues, mots joints par un espace.
fn lines(rendered: &CompiledDocument) -> Vec<String> {
    rendered.lines.iter().map(|line| line.words.join(" ")).collect()
}

/// Vérifie le rendu d'un CV généré avec le symbole attendu.
fn assert_list_rendering(rendered: &CompiledDocument, marker: &str, other: &str) {
    let lines = lines(rendered);
    let marked: Vec<&String> = lines.iter().filter(|l| l.split(' ').next() == Some(marker)).collect();
    // Un symbole par élément, devant lui, et aucun symbole seul (projet sans description).
    assert_eq!(marked.len(), ITEM_FIRST_WORDS.len(), "lignes à symbole : {marked:?}\n{lines:#?}");
    for (line, word) in marked.iter().zip(ITEM_FIRST_WORDS) {
        assert!(line.starts_with(&format!("{marker} {word}")), "« {word} » précédé de « {marker} » : {line}");
    }
    assert!(!lines.iter().any(|l| l == marker), "symbole orphelin : {lines:#?}");
    assert!(!lines.iter().any(|l| l.starts_with(&format!("{other} "))), "aucun élément avec « {other} » : {lines:#?}");
    // Intro sans symbole ; suite de ligne sur sa propre ligne, sans symbole.
    assert!(lines.iter().any(|l| l == "Équipe de 5 personnes."), "intro sans symbole : {lines:#?}");
    assert!(lines.iter().any(|l| l == "déployés chaque semaine"), "suite de ligne sur sa ligne : {lines:#?}");
    assert_no_dash_in_text("cv", &rendered.text);
}

#[tokio::test]
async fn format_puce_un_symbole_par_element() {
    let (app, sid) = app_with_format("format-liste-puce", "puce").await;

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    let cv = data(&app, sid, "cv", cid);
    assert_eq!(cv["list_marker"], "•");
    assert_eq!(cv["experiences"][0]["intro"], "Équipe de 5 personnes.");
    assert_eq!(cv["experiences"][0]["bullets"][0], "Pipelines SQL pour le reporting\ndéployés chaque semaine");
    assert_eq!(cv["projects"][0]["bullets"], serde_json::json!([]), "projet sans description");
    assert_no_dash_in_data("cv", &cv);
    assert_list_rendering(&render(&app, sid, "cv", &cv).await, "•", "-");
}

#[tokio::test]
async fn format_tiret_symbole_de_la_session_partout() {
    let (app, sid) = app_with_format("format-liste-tiret", "tiret").await;

    let (cid, error, completed) = generate(&app, sid).await;
    assert!(completed, "génération : {error:?}");

    let cv = data(&app, sid, "cv", cid);
    assert_eq!(cv["list_marker"], "-", "tiret simple, jamais « – » ni « — »");
    assert_no_dash_in_data("cv", &cv);
    // Le séparateur des coordonnées (« • » en dur) n'est pas un symbole de liste : il reste.
    let rendered = render(&app, sid, "cv", &cv).await;
    assert!(rendered.text.contains('•'), "séparateur de l'en-tête inchangé : {}", rendered.text);
    assert_list_rendering(&rendered, "-", "•");
}
