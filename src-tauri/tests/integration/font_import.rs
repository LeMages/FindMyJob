//! Tests : import d'un dossier de polices depuis Paramètres. Parcours des
//! sous-dossiers, collections `.ttc`, refus des formats web, nom de famille
//! lu dans les fichiers (pas celui du dossier) et avertissement d'homonyme.

// Importation des modules internes
use crate::shared::font_fixtures::copy_font_fixture;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::{delete_font, import_font_folder, list_fonts};
use findmyjob_lib::models::FontFamilyInfo;

#[test]
fn sous_dossiers_parcourus_et_vrai_nom_de_famille() {
    let dir = unique_temp_dir("font-import-sous-dossiers");
    // Dossier parent d'une archive : les fichiers sont sous `static/`.
    let source = dir.join("Arimo_Archive");
    copy_font_fixture("arimo", &source);
    let fonts_dir = dir.join("polices");

    let imported = import_font_folder(&fonts_dir, source.to_str().unwrap()).expect("import");

    assert_eq!(imported.folder, "Arimo_Archive");
    assert_eq!(imported.families, vec![FontFamilyInfo { name: "Arimo".to_string(), variants: vec!["400".to_string()] }]);
    assert_eq!(imported.file_count, 1, "OFL.txt est ignoré");
    assert!(imported.shadowed_families.is_empty());
    assert!(fonts_dir.join("Arimo_Archive").join("static").join("Arimo-Regular.ttf").is_file());

    let listed = list_fonts(&fonts_dir).expect("liste");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].families, imported.families);

    delete_font(&fonts_dir, "Arimo_Archive").expect("suppression");
    assert!(list_fonts(&fonts_dir).expect("liste").is_empty());
}

#[test]
fn collection_ttc_acceptee() {
    let dir = unique_temp_dir("font-import-ttc");
    let source = dir.join("collection");
    copy_font_fixture("collection", &source);

    let imported = import_font_folder(&dir.join("polices"), source.to_str().unwrap()).expect("import");

    assert_eq!(imported.file_count, 1);
    assert_eq!(
        imported.families,
        vec![FontFamilyInfo { name: "Arimo".to_string(), variants: vec!["400".to_string(), "400 italique".to_string()] }]
    );
}

#[test]
fn woff_refuse_avec_message_clair() {
    let dir = unique_temp_dir("font-import-woff");
    let source = dir.join("web");
    copy_font_fixture("web", &source);
    let fonts_dir = dir.join("polices");

    let error = import_font_folder(&fonts_dir, source.to_str().unwrap()).expect_err("refus attendu");

    assert_eq!(
        error.to_string(),
        "Les polices WOFF et WOFF2 ne sont pas prises en charge : utilisez un fichier .ttf ou .otf."
    );
    assert!(!fonts_dir.join("web").exists(), "rien n'est copié");
}

#[test]
fn homonyme_d_une_police_embarquee_importee_avec_avertissement() {
    let dir = unique_temp_dir("font-import-homonyme");
    let source = dir.join("Libertinus_Serif");
    copy_font_fixture("libertinus", &source);
    let fonts_dir = dir.join("polices");

    let imported = import_font_folder(&fonts_dir, source.to_str().unwrap()).expect("l'import réussit");

    assert_eq!(imported.families[0].name, "Libertinus Serif", "nom de famille, pas Libertinus_Serif");
    assert_eq!(imported.shadowed_families, vec!["Libertinus Serif".to_string()]);
    let listed = list_fonts(&fonts_dir).expect("liste");
    assert_eq!(listed[0].shadowed_families, vec!["Libertinus Serif".to_string()]);
}
