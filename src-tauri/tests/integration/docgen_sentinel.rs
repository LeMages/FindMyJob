//! Tests de la recherche de valeurs écrites en dur dans un rendu sentinelle
//! (`findmyjob_lib::docgen::sentinel_data`).

// Importation des modules internes
use findmyjob_lib::docgen::sentinel_data::hardcoded_values;

#[test]
fn le_texte_en_dur_est_libre_sauf_s_il_reproduit_une_valeur() {
    let values = vec!["Antoine".to_string(), "Dupont".to_string(), "Lyon".to_string(), "Transvia Logistique".to_string()];
    let rendered = "__F1__ __F2__ | __F3__ Références disponibles sur demande __f4__ Stage dès mars 2027";
    assert!(hardcoded_values(rendered, &values).is_empty(), "texte en dur sans valeur du schéma : accepté");

    let rendered = "ANTOINE DUPONT __F3__ • Lyon, __F4__";
    assert_eq!(
        hardcoded_values(rendered, &values),
        vec!["Antoine".to_string(), "Dupont".to_string(), "Lyon".to_string()],
        "nom et ville écrits en dur, quelle que soit la casse"
    );
}

#[test]
fn une_valeur_de_plusieurs_mots_doit_apparaitre_a_la_suite() {
    let values = vec!["Transvia Logistique".to_string()];
    assert!(hardcoded_values("Transvia __F1__ Logistique", &values).is_empty());
    assert_eq!(hardcoded_values("Entreprise : Transvia Logistique.", &values), values);
}
