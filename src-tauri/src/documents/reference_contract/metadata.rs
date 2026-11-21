//! Lecture et contrôle de la métadonnée `<findmyjob>` d'une référence :
//! unique, lisible, du bon contrat et de la version courante, couvrant
//! toutes les langues prises en charge, avec un format de périodes pour un CV.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::models::doc_lang::{is_supported, SUPPORTED_LANGS};
use crate::models::{expected_contract, RawMetadata, ReferenceIssue, ReferenceIssueCode, CONTRACT_VERSION};

/// Métadonnée lue, et problèmes relevés.
///
/// # Attributes
///
/// * `metadata` - Métadonnée désérialisée, `None` si elle est absente,
///   dupliquée ou illisible (la validation s'arrête alors).
/// * `issues` - Problèmes relevés (bloquants).
pub struct MetadataReading {
    pub metadata: Option<RawMetadata>,
    pub issues: Vec<ReferenceIssue>,
}

/// Lit la métadonnée `<findmyjob>` parmi celles du rendu et la contrôle.
///
/// # Arguments
///
/// * `found` - Valeurs des métadonnées `<findmyjob>` du rendu (voir
///   `documents::typst_metadata`).
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `MetadataReading`.
pub fn read_metadata(found: &[Result<Value, String>], doc_type: &str) -> MetadataReading {
    let unreadable = |issue: ReferenceIssue| MetadataReading { metadata: None, issues: vec![issue] };
    // ------------------------------------------------------------------
    // 1. Une seule métadonnée, convertible et désérialisable
    // ------------------------------------------------------------------
    let value = match found {
        [] => return unreadable(ReferenceIssue::new(ReferenceIssueCode::MetadataMissing)),
        [single] => single,
        many => {
            return unreadable(
                ReferenceIssue::new(ReferenceIssueCode::MetadataDuplicated).with_detail(many.len().to_string()),
            )
        }
    };
    let value = match value {
        Ok(value) => value,
        Err(e) => return unreadable(ReferenceIssue::new(ReferenceIssueCode::MetadataInvalid).with_detail(e.clone())),
    };
    let metadata: RawMetadata = match serde_json::from_value(value.clone()) {
        Ok(metadata) => metadata,
        Err(e) => {
            return unreadable(ReferenceIssue::new(ReferenceIssueCode::MetadataInvalid).with_detail(e.to_string()))
        }
    };

    // ------------------------------------------------------------------
    // 2. Contrat, version, langues et format des périodes
    // ------------------------------------------------------------------
    let mut issues = Vec::new();
    let expected = expected_contract(doc_type);
    if metadata.contract != expected {
        issues.push(
            ReferenceIssue::new(ReferenceIssueCode::WrongContract).at_path("contract").with_detail(&metadata.contract),
        );
    }
    if metadata.version != CONTRACT_VERSION {
        issues.push(
            ReferenceIssue::new(ReferenceIssueCode::OutdatedContractVersion)
                .at_path("version")
                .with_detail(metadata.version.to_string()),
        );
    }
    issues.extend(language_issues(&metadata.languages));
    if doc_type == "cv" && metadata.period.is_none() {
        issues.push(
            ReferenceIssue::new(ReferenceIssueCode::MetadataInvalid).at_path("period").with_detail("missing field"),
        );
    }
    MetadataReading { metadata: Some(metadata), issues }
}

/// Langues déclarées : chaque langue prise en charge doit y être, et
/// aucune autre.
fn language_issues(languages: &[String]) -> Vec<ReferenceIssue> {
    let mut issues: Vec<ReferenceIssue> = SUPPORTED_LANGS
        .iter()
        .filter(|lang| !languages.iter().any(|declared| declared == *lang))
        .map(|lang| ReferenceIssue::new(ReferenceIssueCode::LanguageMissing).at_path("languages").with_detail(*lang))
        .collect();
    issues.extend(languages.iter().filter(|lang| !is_supported(lang)).map(|lang| {
        ReferenceIssue::new(ReferenceIssueCode::MetadataInvalid)
            .at_path("languages")
            .with_detail(format!("unsupported language: {lang}"))
    }));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn codes(reading: &MetadataReading) -> Vec<ReferenceIssueCode> {
        reading.issues.iter().map(|i| i.code).collect()
    }

    fn metadata(contract: &str, version: u32, languages: &[&str]) -> Value {
        json!({ "contract": contract, "version": version, "languages": languages, "example": {}, "slots": [] })
    }

    #[test]
    fn absente_dupliquee_ou_illisible() {
        assert_eq!(codes(&read_metadata(&[], "lettre_motivation")), vec![ReferenceIssueCode::MetadataMissing]);
        let one = Ok(metadata("findmyjob/lettre", 1, &["fr", "en"]));
        assert_eq!(
            codes(&read_metadata(&[one.clone(), one], "lettre_motivation")),
            vec![ReferenceIssueCode::MetadataDuplicated]
        );
        let reading = read_metadata(&[Ok(json!({ "contract": "findmyjob/lettre" }))], "lettre_motivation");
        assert_eq!(codes(&reading), vec![ReferenceIssueCode::MetadataInvalid]);
        assert!(reading.metadata.is_none());
    }

    #[test]
    fn contrat_version_et_langues() {
        let reading = read_metadata(&[Ok(metadata("findmyjob/lettre", 2, &["fr"]))], "cv");
        assert_eq!(
            codes(&reading),
            vec![
                ReferenceIssueCode::WrongContract,
                ReferenceIssueCode::OutdatedContractVersion,
                ReferenceIssueCode::LanguageMissing,
                ReferenceIssueCode::MetadataInvalid,
            ]
        );
        assert_eq!(reading.issues[2].detail.as_deref(), Some("en"));
        assert_eq!(reading.issues[3].path.as_deref(), Some("period"));
    }

    #[test]
    fn lettre_conforme() {
        let reading = read_metadata(&[Ok(metadata("findmyjob/lettre", 1, &["fr", "en"]))], "lettre_motivation");
        assert!(reading.issues.is_empty(), "{:?}", reading.issues);
        assert!(reading.metadata.is_some());
    }
}
