//! Tests : cohérence entre le code et la documentation du contrat
//! (`docs/reference-contract/contract.{fr,en}.md`), et assemblage du prompt
//! de conversion copié par l'interface. Chaque code d'erreur, chaque champ
//! des données et chaque champ de la métadonnée sont documentés dans les
//! deux langues ; le prompt colle le contrat et l'exemple en entier, avec
//! le lien GitHub de la langue demandée.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use findmyjob_lib::documents::reference_contract::example_schema::{cv_shape, field_paths, lm_shape};
use findmyjob_lib::models::ReferenceIssueCode;
use findmyjob_lib::services::reference_prompt::{reference_doc_links, reference_prompt, CONTRACT_DOC_BASE_URL};

/// Champs de la métadonnée `<findmyjob>`, de ses emplacements et de ses périodes.
const METADATA_FIELDS: [&str; 14] = [
    "contract",
    "version",
    "languages",
    "example",
    "guidance",
    "slots",
    "period",
    "field",
    "mode",
    "instruction",
    "granularity",
    "separator",
    "month_style",
    "numeric_separator",
];

/// Lit un fichier de `docs/reference-contract/`.
fn read_doc(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("docs").join("reference-contract").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} illisible : {e}", path.display()))
        .replace("\r\n", "\n")
}

/// Contrat dans les deux langues, nommé.
fn contracts() -> [(&'static str, String); 2] {
    [("contract.fr.md", read_doc("contract.fr.md")), ("contract.en.md", read_doc("contract.en.md"))]
}

#[test]
fn chaque_code_documente_dans_les_deux_langues() {
    for (name, contract) in contracts() {
        let missing: Vec<&str> =
            ReferenceIssueCode::ALL.iter().map(|code| code.code()).filter(|code| !contract.contains(&format!("`{code}`"))).collect();
        assert!(missing.is_empty(), "{name} : codes non documentés {missing:?}");
    }
}

#[test]
fn chaque_champ_documente_dans_les_deux_langues() {
    let mut fields: Vec<String> = [cv_shape(), lm_shape()]
        .iter()
        .flat_map(field_paths)
        .map(|path| path.rsplit('.').next().unwrap_or(&path).trim_end_matches("[]").to_string())
        .collect();
    fields.extend(METADATA_FIELDS.iter().map(ToString::to_string));
    fields.sort();
    fields.dedup();
    for (name, contract) in contracts() {
        // Un champ est cité seul (`summary`, `bullets[]`) ou dans son chemin (`identity.prenom`, `recipient.extra_lines[]`).
        let documented = |field: &str| {
            [format!("`{field}"), format!(".{field}`"), format!(".{field}[]")].iter().any(|form| contract.contains(form))
        };
        let missing: Vec<&String> = fields.iter().filter(|field| !documented(field)).collect();
        assert!(missing.is_empty(), "{name} : champs non documentés {missing:?}");
    }
}

#[test]
fn prompt_complet_dans_chaque_langue() {
    for doc_type in ["cv", "lettre_motivation"] {
        for lang in ["fr", "en"] {
            let prompt = reference_prompt(doc_type, lang).expect("prompt");

            assert!(!prompt.contains("{{"), "{doc_type}/{lang} : emplacement non remplacé");
            let example = read_doc(if doc_type == "cv" { "cv_exemple.typ" } else { "lm_exemple.typ" });
            assert!(prompt.contains(example.trim_end()), "{doc_type}/{lang} : exemple incomplet");
            let contract = read_doc(&format!("contract.{lang}.md"));
            assert!(prompt.contains(contract.trim_end()), "{doc_type}/{lang} : contrat incomplet");
            let link = format!("{CONTRACT_DOC_BASE_URL}contract.{lang}.md");
            assert!(prompt.contains(&link), "{doc_type}/{lang} : lien {link} absent");
            assert!(prompt.contains("Typst 0.15"), "{doc_type}/{lang}");
        }
    }
    assert!(reference_prompt("cv", "fr").expect("prompt").starts_with("Voici mon CV actuel"));
    assert!(reference_prompt("lettre_motivation", "en").expect("prompt").starts_with("Here is my current cover letter"));
}

#[test]
fn type_ou_langue_inconnus_refuses() {
    assert!(reference_prompt("photo", "fr").is_err());
    assert!(reference_prompt("cv", "de").is_err());
    assert!(reference_doc_links("de").is_err());
}

#[test]
fn liens_vers_le_contrat_et_les_exemples() {
    let links = reference_doc_links("en").expect("liens");
    assert_eq!(links.contract, "https://github.com/LeMages/FindMyJob/blob/main/docs/reference-contract/contract.en.md");
    assert!(links.cv_example.ends_with("/docs/reference-contract/cv_exemple.typ"));
    assert!(links.lm_example.ends_with("/docs/reference-contract/lm_exemple.typ"));
}
