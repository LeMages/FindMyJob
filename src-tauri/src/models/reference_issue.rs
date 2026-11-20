//! Problèmes relevés par la validation d'une référence (CV ou lettre) au
//! contrat FindMyJob.
//!
//! Chaque problème porte un code typé, sérialisé en `snake_case` (ex.
//! `"forbidden_construct"`) : c'est l'interface qui choisit le texte affiché,
//! dans sa langue, comme pour `ScrapingFailure`. Le backend ne fournit que
//! le contexte brut (chemin du champ, ligne Typst, détail technique).

// Importation des crates externes
use serde::{Deserialize, Serialize};

/// Code d'un problème de référence. Chaque code est documenté dans
/// `docs/reference-contract/contract.{fr,en}.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceIssueCode {
    /// Le fichier ne finit pas par `.typ`.
    NotTyp,
    /// Le fichier dépasse la taille maximale acceptée.
    TooLarge,
    /// Le fichier n'est pas du texte UTF-8.
    NotUtf8,
    /// Construction interdite (`#import`, `read(`, `image("fichier")`…).
    ForbiddenConstruct,
    /// Le template ne compile pas seul (avec son exemple).
    CompileError,
    /// Une famille de police demandée est introuvable.
    MissingFont,
    /// Aucune métadonnée `<findmyjob>`.
    MetadataMissing,
    /// Plusieurs métadonnées `<findmyjob>`.
    MetadataDuplicated,
    /// Métadonnée illisible (champ manquant ou mal typé).
    MetadataInvalid,
    /// `contract` ne correspond pas au type de document importé.
    WrongContract,
    /// `version` n'est pas la version courante du contrat.
    OutdatedContractVersion,
    /// Une langue prise en charge par l'application manque dans `languages`.
    LanguageMissing,
    /// Champ de l'exemple manquant, en trop ou mal typé.
    ExampleInvalid,
    /// Emplacement rédigé sur un champ inconnu ou absent de l'exemple.
    SlotUnknownField,
    /// Emplacement rédigé avec un mode inconnu.
    SlotInvalidMode,
    /// Deux emplacements rédigés sur le même champ.
    SlotDuplicated,
    /// Le template ne compile pas avec l'exemple dans une des langues.
    LanguageCompileError,
    /// Rendu dans une langue, le document a plus de pages que l'exemple.
    PageCountMismatch,
    /// Le template ne compile pas avec des données vides.
    EmptyDataCompileError,
    /// Une valeur d'exemple d'identité ou du profil est écrite en dur.
    HardcodedValue,
    /// `data.list_marker` ne précède pas exactement chaque élément de liste.
    ListMarkerWrong,
    /// Un titre de section reste affiché alors que sa section est vide.
    OrphanHeading,
    /// Avertissement : séparateur orphelin quand un champ secondaire est vide.
    OrphanSeparator,
    /// Avertissement : bloc destinataire non calé contre la marge droite.
    RecipientNotRightAligned,
    /// Avertissement : tiret cadratin ou demi-cadratin dans le rendu.
    DashMarks,
}

impl ReferenceIssueCode {
    /// Tous les codes, dans l'ordre de la validation (pour les tests de
    /// documentation et l'interface).
    pub const ALL: [ReferenceIssueCode; 25] = [
        ReferenceIssueCode::NotTyp,
        ReferenceIssueCode::TooLarge,
        ReferenceIssueCode::NotUtf8,
        ReferenceIssueCode::ForbiddenConstruct,
        ReferenceIssueCode::CompileError,
        ReferenceIssueCode::MissingFont,
        ReferenceIssueCode::MetadataMissing,
        ReferenceIssueCode::MetadataDuplicated,
        ReferenceIssueCode::MetadataInvalid,
        ReferenceIssueCode::WrongContract,
        ReferenceIssueCode::OutdatedContractVersion,
        ReferenceIssueCode::LanguageMissing,
        ReferenceIssueCode::ExampleInvalid,
        ReferenceIssueCode::SlotUnknownField,
        ReferenceIssueCode::SlotInvalidMode,
        ReferenceIssueCode::SlotDuplicated,
        ReferenceIssueCode::LanguageCompileError,
        ReferenceIssueCode::PageCountMismatch,
        ReferenceIssueCode::EmptyDataCompileError,
        ReferenceIssueCode::HardcodedValue,
        ReferenceIssueCode::ListMarkerWrong,
        ReferenceIssueCode::OrphanHeading,
        ReferenceIssueCode::OrphanSeparator,
        ReferenceIssueCode::RecipientNotRightAligned,
        ReferenceIssueCode::DashMarks,
    ];

    /// Code stable, identique à sa forme sérialisée.
    ///
    /// # Returns
    ///
    /// Le code `snake_case` (ex. `"forbidden_construct"`).
    pub fn code(self) -> &'static str {
        match self {
            ReferenceIssueCode::NotTyp => "not_typ",
            ReferenceIssueCode::TooLarge => "too_large",
            ReferenceIssueCode::NotUtf8 => "not_utf8",
            ReferenceIssueCode::ForbiddenConstruct => "forbidden_construct",
            ReferenceIssueCode::CompileError => "compile_error",
            ReferenceIssueCode::MissingFont => "missing_font",
            ReferenceIssueCode::MetadataMissing => "metadata_missing",
            ReferenceIssueCode::MetadataDuplicated => "metadata_duplicated",
            ReferenceIssueCode::MetadataInvalid => "metadata_invalid",
            ReferenceIssueCode::WrongContract => "wrong_contract",
            ReferenceIssueCode::OutdatedContractVersion => "outdated_contract_version",
            ReferenceIssueCode::LanguageMissing => "language_missing",
            ReferenceIssueCode::ExampleInvalid => "example_invalid",
            ReferenceIssueCode::SlotUnknownField => "slot_unknown_field",
            ReferenceIssueCode::SlotInvalidMode => "slot_invalid_mode",
            ReferenceIssueCode::SlotDuplicated => "slot_duplicated",
            ReferenceIssueCode::LanguageCompileError => "language_compile_error",
            ReferenceIssueCode::PageCountMismatch => "page_count_mismatch",
            ReferenceIssueCode::EmptyDataCompileError => "empty_data_compile_error",
            ReferenceIssueCode::HardcodedValue => "hardcoded_value",
            ReferenceIssueCode::ListMarkerWrong => "list_marker_wrong",
            ReferenceIssueCode::OrphanHeading => "orphan_heading",
            ReferenceIssueCode::OrphanSeparator => "orphan_separator",
            ReferenceIssueCode::RecipientNotRightAligned => "recipient_not_right_aligned",
            ReferenceIssueCode::DashMarks => "dash_marks",
        }
    }

    /// Gravité du code : les avertissements n'empêchent pas l'import.
    pub fn severity(self) -> Severity {
        match self {
            ReferenceIssueCode::OrphanSeparator
            | ReferenceIssueCode::RecipientNotRightAligned
            | ReferenceIssueCode::DashMarks => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

impl std::fmt::Display for ReferenceIssueCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

/// Gravité d'un problème.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Bloque l'import.
    Error,
    /// Signalé, l'import réussit.
    Warning,
}

/// Problème relevé dans une référence.
///
/// # Attributes
///
/// * `code` - Code typé, traduit par l'interface.
/// * `severity` - Bloquant ou avertissement (déduit du code).
/// * `path` - Chemin du champ concerné (`example.experiences[1].period`,
///   `slots[0].field`), s'il y en a un.
/// * `line` - Ligne Typst concernée (à partir de 1), si elle est connue.
/// * `detail` - Contexte brut, jamais traduit (message du compilateur,
///   famille de police, valeur trouvée, langue).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceIssue {
    pub code: ReferenceIssueCode,
    pub severity: Severity,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub detail: Option<String>,
}

impl ReferenceIssue {
    /// Problème sans contexte.
    pub fn new(code: ReferenceIssueCode) -> Self {
        Self { code, severity: code.severity(), path: None, line: None, detail: None }
    }

    /// Même problème, avec le chemin du champ concerné.
    pub fn at_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Même problème, avec sa ligne Typst.
    pub fn at_line(mut self, line: Option<usize>) -> Self {
        self.line = line;
        self
    }

    /// Même problème, avec son détail brut.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Vrai si le problème bloque l'import.
    pub fn is_blocking(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl std::fmt::Display for ReferenceIssue {
    /// Forme lisible pour les journaux : `code [chemin] (ligne N) : détail`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)?;
        if let Some(path) = &self.path {
            write!(f, " [{path}]")?;
        }
        if let Some(line) = self.line {
            write!(f, " (ligne {line})")?;
        }
        if let Some(detail) = &self.detail {
            write!(f, " : {detail}")?;
        }
        Ok(())
    }
}
