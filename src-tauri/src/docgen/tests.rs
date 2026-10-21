//! Tests de l'assemblage déterministe (sans LLM ni base) : construction de
//! `cv_data` / `lm_data` depuis un profil fictif, destinataire, et
//! compilation d'un template de fixture avec données complètes puis avec
//! champs optionnels absents.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use chrono::{NaiveDate, Utc};

// Importation des modules internes
use crate::docgen::build_cv_data::{build_cv_data, CvLayout};
use crate::docgen::build_lm_data::{build_lm_data, LmLayout};
use crate::docgen::entry_texts::source_texts;
use crate::docgen::identity::{display_url, lines};
use crate::docgen::recipient::build_recipient;
use crate::documents::typst::compile_template;
use crate::docgen::selection::{select, Selection};
use crate::models::{
    CandidatureDetail, CvContent, CvData, CvExperience, CvFormation, CvLabels, CvProject, ExperienceView,
    FormationView, Identity, LmContent, LmData, LmLabels, PeriodGranularity, PeriodStyle, ProfileDto, ProjetView,
    Recipient, SkillsGroup, UserView,
};

/// Aucun texte en dur de la référence.
static NO_TEXTS: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();

/// Template de CV de test.
const CV_TEMPLATE: &str = include_str!("../../tests/fixtures/cv_template.typ");

// ======================================================================
// Fixtures anonymisées
// ======================================================================

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date de fixture valide")
}

fn user() -> UserView {
    let now = Utc::now();
    UserView {
        id: 1,
        nom: Some("Martin".to_string()),
        prenom: Some("Camille".to_string()),
        email: Some("camille.martin@example.org".to_string()),
        telephone: None,
        ville: Some("Lyon".to_string()),
        zip_code: Some("69000".to_string()),
        permis: Some(true),
        description: None,
        linkedin: Some("https://www.linkedin.com/in/camille-martin/".to_string()),
        github: None,
        portfolio: None,
        interets: Some("• Escalade\n\nPhotographie".to_string()),
        langue_profil: "fr".to_string(),
        created_at: now,
    }
}

fn profile() -> ProfileDto {
    ProfileDto {
        user: user(),
        experiences: vec![ExperienceView {
            id: 10,
            poste: "Stage data".to_string(),
            entreprise: Some("Société Exemple".to_string()),
            lieu: None,
            description: Some("A conçu un pipeline.
- Mis en production chez le client".to_string()),
            date_debut: Some(date(2025, 11, 1)),
            date_fin: Some(date(2026, 4, 30)),
            type_contrat: None,
        }],
        formations: vec![FormationView {
            id: 30,
            titre: "Diplôme d'ingénieur".to_string(),
            ecole: Some("École Exemple".to_string()),
            description: Some("Spécialité data\n- Échange à l'étranger".to_string()),
            date_debut: Some(date(2022, 9, 1)),
            date_fin: Some(date(2027, 6, 30)),
        }],
        projets: vec![
            ProjetView {
                id: 20,
                titre: "Projet A".to_string(),
                description: Some("• Fait 20.".to_string()),
                stack: Some("Rust, Typst".to_string()),
                url: Some("https://github.com/camille/projet-a/".to_string()),
                pour_qui: Some("École".to_string()),
                date_debut: Some(date(2025, 1, 1)),
                date_fin: Some(date(2025, 6, 1)),
            },
            ProjetView {
                id: 21,
                titre: "Projet B".to_string(),
                description: None,
                stack: None,
                url: None,
                pour_qui: None,
                date_debut: None,
                date_fin: None,
            },
        ],
        skills: Vec::new(),
        langues: Vec::new(),
    }
}

/// Assemble le CV d'un profil, textes des entrées tirés du profil.
fn cv_data(profile: &ProfileDto, selection: &Selection, document: &CvData) -> CvData {
    let (labels, style) = (cv_labels(), period_style());
    let layout = CvLayout { document, labels: &labels, period_style: &style, separator: " - ", lang: "fr", texts: &NO_TEXTS };
    build_cv_data(profile, selection, &source_texts(profile, selection), &cv_content(), &layout)
}

fn cv_labels() -> CvLabels {
    CvLabels {
        section_profile: "Profil".to_string(),
        section_experiences: "Expérience".to_string(),
        section_projects: "Projets".to_string(),
        interets_label: "Intérêts".to_string(),
        permis: "Permis B".to_string(),
        present: "aujourd'hui".to_string(),
        months_short: ["Janv.", "Févr.", "Mars", "Avr.", "Mai", "Juin", "Juil.", "Août", "Sept.", "Oct.", "Nov.", "Déc."]
            .iter()
            .map(|m| m.to_string())
            .collect(),
        ..Default::default()
    }
}

fn period_style() -> PeriodStyle {
    PeriodStyle {
        experiences: PeriodGranularity::Month,
        projects: PeriodGranularity::Year,
        formations: PeriodGranularity::Year,
    }
}

fn cv_content() -> CvContent {
    CvContent {
        title: "Ingénieure data".to_string(),
        summary: "Résumé avec un @ et un # et des *étoiles*.".to_string(),
        skills_groups: vec![SkillsGroup { category: "Langages".to_string(), skills: vec!["Rust".to_string()] }],
    }
}

/// Compile le template de fixture avec des données.
fn compile_cv(data: &impl serde::Serialize) -> crate::documents::typst::CompiledDocument {
    let json = serde_json::to_vec(data).expect("données sérialisables");
    let root = std::env::temp_dir();
    let fonts = PathBuf::from("introuvable");
    compile_template(CV_TEMPLATE.to_string(), json, root, fonts).expect("le template de fixture compile")
}

// ======================================================================
// Identité, lignes du profil, destinataire
// ======================================================================

#[test]
fn liens_affichables_et_lignes_du_profil() {
    let url = Some("https://www.linkedin.com/in/camille-martin/".to_string());
    assert_eq!(display_url(url.as_ref()), "linkedin.com/in/camille-martin");
    assert_eq!(display_url(None), "");
    let text = Some("• Escalade\n\n - Photographie \n".to_string());
    assert_eq!(lines(text.as_ref()), vec!["Escalade".to_string(), "Photographie".to_string()]);
}

#[test]
fn destinataire_depuis_la_candidature_sans_contact() {
    let candidature: CandidatureDetail = serde_json::from_value(serde_json::json!({
        "id": 1, "session_id": 1, "poste": "Stage", "entreprise": " Société Exemple ", "localisation": "Lyon",
        "status": "a_postuler", "source": null, "type_contrat": null, "remuneration": null, "url_offre": null,
        "description_offre": null, "notes": null, "date_candidature": null, "cv_path": null,
        "lettre_motivation_path": null, "cv_reference_version": null, "lm_reference_version": null,
        "generation_warnings": [],
        "created_at": "2026-01-01T00:00:00Z"
    }))
    .expect("candidature de fixture");

    let recipient = build_recipient(&candidature);
    assert_eq!(recipient.company, "Société Exemple");
    assert_eq!(recipient.location, "Lyon");
    assert_eq!(recipient.contact_name, "");
    assert_eq!(recipient.contact_role, "");
}

// ======================================================================
// build_cv_data / build_lm_data
// ======================================================================

#[test]
fn cv_data_fusionne_profil_contenu_et_libelles() {
    let profile = profile();
    let selection = select(&profile, &[20]);

    let data = cv_data(&profile, &selection, &CvData::default());

    assert_eq!(data.identity.linkedin, "linkedin.com/in/camille-martin");
    assert_eq!(data.experiences[0].poste, "Stage data");
    assert_eq!(data.experiences[0].entreprise, "Société Exemple");
    assert_eq!(data.experiences[0].period, "Nov. 2025 - Avr. 2026");
    assert_eq!(
        data.experiences[0].bullets,
        vec!["A conçu un pipeline.".to_string(), "Mis en production chez le client".to_string()],
        "bullets = lignes exactes de la description"
    );
    assert_eq!(data.projects.len(), 1);
    assert_eq!(data.projects[0].titre, "Projet A");
    assert_eq!(data.projects[0].period, "2025");
    assert_eq!(data.projects[0].bullets, vec!["Fait 20.".to_string()]);
    assert_eq!(data.projects[0].stack, "Rust, Typst");
    assert_eq!(data.projects[0].url, "github.com/camille/projet-a");
    assert_eq!(data.formations[0].period, "2022 - 2027");
    assert_eq!(data.formations[0].details, vec!["Spécialité data".to_string(), "Échange à l'étranger".to_string()]);
    assert_eq!(data.interets, vec!["Escalade".to_string(), "Photographie".to_string()]);
    assert_eq!(data.lang, "fr");
}

#[test]
fn lm_data_date_et_destinataire() {
    let recipient = Recipient { company: "Société Exemple".to_string(), ..Default::default() };
    let content = LmContent { subject: "Candidature".to_string(), paragraphs: vec!["Paragraphe.".to_string()] };
    let labels = LmLabels { date_prefix: "le ".to_string(), ..Default::default() };

    let document = LmData::default();
    let data = build_lm_data(&profile(), &recipient, &content, &lm_layout(&document, &labels));

    assert_eq!(data.date, "9 octobre 2026");
    assert_eq!(data.recipient.company, "Société Exemple");
    assert_eq!(data.identity.ville, "Lyon");
    assert_eq!(data.identity.title, "Ingénieure data", "sous-titre repris du CV");
    assert_eq!(data.identity.zip_code, "69000");
    assert_eq!(data.paragraphs, vec!["Paragraphe.".to_string()]);
}

/// Mise en page de la lettre de test : sous-titre du CV, 9 octobre 2026.
fn lm_layout<'a>(document: &'a LmData, labels: &'a LmLabels) -> LmLayout<'a> {
    LmLayout { document, labels, title: "Ingénieure data", date: date(2026, 10, 9), lang: "fr", texts: &NO_TEXTS }
}

// ======================================================================
// Repli sur le document de référence
// ======================================================================

#[test]
fn le_document_complete_l_identite_que_le_profil_n_a_pas() {
    let mut profile = profile();
    profile.user.telephone = None;
    profile.user.ville = Some("Thyez".to_string());
    let document = CvData {
        identity: Identity {
            telephone: "+41 00 000 00 00".to_string(),
            ville: "Genève".to_string(),
            portfolio: "exemple.invalid".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let data = cv_data(&profile, &select(&profile, &[]), &document);

    assert_eq!(data.identity.ville, "Thyez", "le profil prime sur le document");
    assert_eq!(data.identity.telephone, "+41 00 000 00 00", "profil vide : valeur du document");
    assert_eq!(data.identity.portfolio, "exemple.invalid");
}

#[test]
fn sections_vides_sans_contenu_de_la_reference() {
    let mut profile = profile();
    profile.experiences.clear();
    profile.projets.clear();
    profile.formations.clear();
    let document = CvData {
        experiences: vec![CvExperience {
            poste: "Poste du document".to_string(),
            bullets: vec!["Bullet du document".to_string()],
            ..Default::default()
        }],
        projects: vec![CvProject { titre: "Projet du document".to_string(), ..Default::default() }],
        formations: vec![CvFormation { titre: "Formation du document".to_string(), ..Default::default() }],
        ..Default::default()
    };

    let data = cv_data(&profile, &select(&profile, &[]), &document);

    assert!(data.experiences.is_empty(), "profil sans expérience : section vide");
    assert!(data.projects.is_empty(), "profil sans projet : section vide");
    assert!(data.formations.is_empty(), "profil sans formation : section vide");
    let json = serde_json::to_string(&data).expect("données sérialisables");
    assert!(!json.contains("du document"), "aucun contenu d'entrée de la référence : {json}");
}

#[test]
fn entree_sans_description_reste_sans_bullet() {
    let profile = profile();
    let data = cv_data(&profile, &select(&profile, &[21]), &CvData::default());
    assert_eq!(data.projects[0].titre, "Projet B");
    assert!(data.projects[0].bullets.is_empty(), "rien de rédigé à la place d'une description vide");
    assert_eq!((data.projects[0].stack.as_str(), data.projects[0].url.as_str()), ("", ""));
}

#[test]
fn textes_traduits_injectes_sans_toucher_aux_noms() {
    let profile = profile();
    let selection = select(&profile, &[20]);
    let mut texts = source_texts(&profile, &selection);
    texts.experiences[0].lines = vec!["Built a pipeline.".to_string(), "Deployed it for the client".to_string()];
    texts.projects[0].lines = vec!["Did 20.".to_string()];
    texts.projects[0].pour_qui = "School".to_string();
    texts.formations[0].lines = vec!["Data major".to_string(), "Exchange abroad".to_string()];
    let (labels, style) = (cv_labels(), period_style());
    let layout =
        CvLayout { document: &CvData::default(), labels: &labels, period_style: &style, separator: " - ", lang: "en", texts: &NO_TEXTS };

    let data = build_cv_data(&profile, &selection, &texts, &cv_content(), &layout);

    assert_eq!(data.experiences[0].bullets, texts.experiences[0].lines);
    assert_eq!(data.projects[0].bullets, vec!["Did 20.".to_string()]);
    assert_eq!(data.projects[0].pour_qui, "School");
    assert_eq!(data.formations[0].details, texts.formations[0].lines);
    assert_eq!(data.experiences[0].poste, "Stage data", "poste jamais traduit");
    assert_eq!(data.experiences[0].entreprise, "Société Exemple");
    assert_eq!(data.projects[0].titre, "Projet A", "titre de projet jamais traduit");
    assert_eq!(data.projects[0].stack, "Rust, Typst");
    assert_eq!(data.formations[0].titre, "Diplôme d'ingénieur");
    assert_eq!(data.formations[0].ecole, "École Exemple");
}

// ======================================================================
// Compilation d'un template de fixture
// ======================================================================

#[test]
fn template_compile_avec_donnees_completes() {
    let profile = profile();
    let data = cv_data(&profile, &select(&profile, &[20, 21]), &CvData::default());

    let doc = compile_cv(&data);

    assert_eq!(doc.page_count, 1);
    assert!(doc.text.contains("CAMILLE MARTIN"), "{}", doc.text);
    assert!(doc.text.contains("Permis B"), "{}", doc.text);
    assert!(doc.text.contains("@"), "le @ du résumé reste du texte : {}", doc.text);
    assert!(doc.text.contains("Projet B"), "{}", doc.text);
    assert!(doc.text.contains("Fait 20."), "{}", doc.text);
}

#[test]
fn stack_et_url_affiches_seulement_si_le_template_les_lit() {
    let profile = profile();
    let data = cv_data(&profile, &select(&profile, &[20]), &CvData::default());

    // Le template de fixture ne lit ni stack ni url : rien n'est ajouté.
    let without = compile_cv(&data);
    assert!(!without.text.contains("Typst"), "{}", without.text);
    assert!(!without.text.contains("github.com"), "{}", without.text);

    // Un template dont la référence affichait ces informations les lit.
    let template = "#let data = json(\"data.json\")\n#for p in data.projects [#p.titre (#p.stack) #p.url]";
    let json = serde_json::to_vec(&data).expect("données sérialisables");
    let with = compile_template(template.to_string(), json, std::env::temp_dir(), PathBuf::from("introuvable"))
        .expect("le template de test compile");
    assert!(with.text.contains("Rust, Typst"), "{}", with.text);
    assert!(with.text.contains("github.com/camille/projet-a"), "{}", with.text);
}

#[test]
fn template_compile_sans_champs_optionnels() {
    let mut profile = profile();
    profile.user.email = None;
    profile.user.linkedin = None;
    profile.user.permis = Some(false);
    profile.user.interets = None;
    let data = cv_data(&profile, &select(&profile, &[]), &CvData::default());

    let doc = compile_cv(&data);

    assert!(doc.text.contains("Lyon"), "{}", doc.text);
    assert!(!doc.text.contains("Permis"), "{}", doc.text);
    assert!(!doc.text.contains("Projets"), "section vide masquée : {}", doc.text);
    assert!(!doc.text.contains("Intérêts"), "{}", doc.text);
    assert!(!doc.text.trim_end().ends_with('•'), "pas de séparateur orphelin : {}", doc.text);
}

// ======================================================================
// Lieu de la lettre : en-tête et ligne de date
// ======================================================================

/// Template de lettre de test.
const LM_TEMPLATE: &str = include_str!("../../tests/fixtures/references/lm_template.typ");

/// Compile la lettre de fixture pour un lieu donné, texte rendu aux espaces
/// normalisés.
fn lm_text(ville: &str, zip_code: &str) -> String {
    let mut profile = profile();
    profile.user.ville = Some(ville.to_string());
    profile.user.zip_code = Some(zip_code.to_string());
    let recipient = Recipient { company: "Société Exemple".to_string(), ..Default::default() };
    let content = LmContent { subject: "Candidature".to_string(), paragraphs: vec!["Paragraphe.".to_string()] };
    let labels = LmLabels { date_prefix: "le".to_string(), ..Default::default() };
    let data = build_lm_data(&profile, &recipient, &content, &lm_layout(&LmData::default(), &labels));

    let json = serde_json::to_vec(&data).expect("données sérialisables");
    let doc = compile_template(LM_TEMPLATE.to_string(), json, std::env::temp_dir(), PathBuf::from("introuvable"))
        .expect("le template de fixture compile");
    doc.text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn lettre_ville_et_code_postal_dans_l_en_tete_ville_seule_dans_la_date() {
    let text = lm_text("Thyez", "74300");
    assert!(text.contains("Thyez, 74300"), "{text}");
    assert!(text.contains("Thyez, le 9 octobre 2026"), "{text}");
    assert!(!text.contains("74300, le"), "le code postal n'est pas dans la ligne de date : {text}");
}

#[test]
fn lettre_sans_ville_ni_separateur_orphelin() {
    let text = lm_text("", "74300");
    assert!(text.contains("le 9 octobre 2026"), "{text}");
    assert!(!text.contains(", le"), "pas de virgule orpheline devant la date : {text}");
    assert!(!text.contains(", 74300"), "pas de virgule orpheline devant le code postal : {text}");
}

#[test]
fn reference_d_une_version_precedente_a_reimporter() {
    use crate::documents::reference_template::load_texts;
    use crate::error::AppError;

    let dir = std::env::temp_dir().join(format!("findmyjob-reference-texts-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dossier temporaire");
    let missing = dir.join("absent.json");
    assert!(matches!(load_texts(&missing, "CV"), Err(AppError::OutdatedReference(_))), "fichier absent : à réimporter");

    let old = dir.join("ancien.json");
    std::fs::write(&old, r#"{"format_version": 1, "lang": "fr", "texts": {}, "guidance": {"general": "", "zones": []}}"#)
        .expect("écriture");
    assert!(matches!(load_texts(&old, "CV"), Err(AppError::OutdatedReference(_))), "autre format : à réimporter");

    let current = dir.join("courant.json");
    std::fs::write(&current, r#"{"format_version": 2, "lang": "fr", "texts": {"t1": "P.J."}, "guidance": {"general": "", "zones": []}}"#)
        .expect("écriture");
    assert_eq!(load_texts(&current, "CV").expect("format courant").texts["t1"], "P.J.");
    std::fs::remove_dir_all(&dir).expect("nettoyage");
}
