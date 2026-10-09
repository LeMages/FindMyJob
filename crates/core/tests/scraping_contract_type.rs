//! Tests de la correspondance des types de contrat
//! (`findmyjob_core::scraping::contract_type`) : `employmentType` INTERN,
//! mots-clés du titre, et refus de deviner.

// Importation des crates externes
use serde_json::json;

// Importation des modules internes
use findmyjob_core::scraping::contract_type::{contract_type_from_title, resolve_contract_type};

#[test]
fn intern_donne_stage() {
    assert_eq!(resolve_contract_type(Some(&json!("INTERN")), Some("Data Analyst")), Some("stage"));
    assert_eq!(resolve_contract_type(Some(&json!(["FULL_TIME", "INTERN"])), None), Some("stage"));
}

#[test]
fn autres_employment_types_passent_par_le_titre() {
    assert_eq!(resolve_contract_type(Some(&json!("FULL_TIME")), Some("Développeur CDI H/F")), Some("cdi"));
    assert_eq!(resolve_contract_type(Some(&json!("FULL_TIME")), Some("Développeur backend")), None);
    assert_eq!(resolve_contract_type(Some(&json!("CONTRACTOR")), None), None);
    assert_eq!(resolve_contract_type(None, None), None);
}

#[test]
fn mots_cles_du_titre() {
    let cases = [
        ("Stage - Assistant marketing", Some("stage")),
        ("Stagiaire contrôle de gestion", Some("stage")),
        ("Software Engineer Intern", Some("stage")),
        ("Internship Data Science", Some("stage")),
        ("Alternance Chargé de communication", Some("alternance")),
        ("Contrat d'apprentissage - Comptable", Some("alternance")),
        ("CDD 6 mois - Juriste", Some("cdd")),
        ("Mission en Intérim - Cariste", Some("interim")),
        ("CDI : Product Owner", Some("cdi")),
    ];
    for (title, expected) in cases {
        assert_eq!(contract_type_from_title(title), expected, "titre : {title}");
    }
}

#[test]
fn jamais_de_devinette() {
    // Mot-clé seulement contenu dans un autre mot.
    assert_eq!(contract_type_from_title("Internal Auditor"), None);
    assert_eq!(contract_type_from_title("Backstage manager"), None);
    // Plusieurs types différents cités.
    assert_eq!(contract_type_from_title("Stage ou alternance - Data"), None);
    assert_eq!(contract_type_from_title("CDD pouvant évoluer en CDI"), None);
    // Aucun mot-clé.
    assert_eq!(contract_type_from_title("Chef de projet digital"), None);
}

#[test]
fn meme_type_cite_plusieurs_fois() {
    assert_eq!(contract_type_from_title("Stage - stagiaire RH"), Some("stage"));
}
