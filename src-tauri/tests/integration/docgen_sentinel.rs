//! Tests des données sentinelles et de la recherche de mots écrits en dur
//! (`findmyjob_lib::docgen::sentinel_data`).

// Importation des modules internes
use findmyjob_lib::docgen::sentinel_data::{hardcoded_values, sentinel_cv_data, sentinel_lm_data};
use findmyjob_lib::models::{CvData, LmData};

#[test]
fn chaque_champ_du_cv_porte_un_marqueur_unique() {
    let value = sentinel_cv_data("fr").expect("données sentinelles");
    let data: CvData = serde_json::from_value(value.clone()).expect("schéma du CV");
    assert_eq!(data.lang, "fr", "la langue n'est pas marquée");
    assert!(data.identity.permis);
    assert!(data.summary.starts_with("__F") && data.summary.ends_with("__"));
    assert_eq!(data.experiences.len(), 1);
    assert_eq!(data.experiences[0].bullets.len(), 1);
    assert_eq!(data.labels.months_short.len(), 12);

    let text = value.to_string();
    let markers: Vec<&str> = text.split('"').filter(|s| s.starts_with("__F")).collect();
    let unique: std::collections::HashSet<&str> = markers.iter().copied().collect();
    assert_eq!(markers.len(), unique.len(), "marqueurs en double : {markers:?}");
}

#[test]
fn la_lettre_a_un_paragraphe_marque() {
    let data: LmData = serde_json::from_value(sentinel_lm_data("en").expect("données")).expect("schéma");
    assert_eq!(data.lang, "en");
    assert_eq!(data.paragraphs.len(), 1);
    assert!(data.identity.ville.starts_with("__F"));
}

#[test]
fn le_texte_en_dur_est_libre_sauf_s_il_reproduit_une_valeur() {
    let values = vec!["Camille".to_string(), "Exemple".to_string(), "Thyez".to_string(), "Need4Viz".to_string()];
    let rendered = "__F1__ __F2__ | __F3__ Références disponibles sur demande __f4__ Stage dès mars 2027";
    assert!(hardcoded_values(rendered, &values).is_empty(), "texte en dur sans valeur du schéma : accepté");

    let rendered = "CAMILLE EXEMPLE __F3__ • Thyez, __F4__";
    assert_eq!(
        hardcoded_values(rendered, &values),
        vec!["Camille".to_string(), "Exemple".to_string(), "Thyez".to_string()],
        "nom et ville écrits en dur, quelle que soit la casse"
    );
}

#[test]
fn une_valeur_de_plusieurs_mots_doit_apparaitre_a_la_suite() {
    let values = vec!["Efrei Paris".to_string()];
    assert!(hardcoded_values("Efrei __F1__ Paris", &values).is_empty());
    assert_eq!(hardcoded_values("École : Efrei Paris.", &values), values);
}
