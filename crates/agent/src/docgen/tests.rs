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
use crate::docgen::build_lm_data::build_lm_data;
use crate::docgen::identity::{display_url, lines};
use crate::docgen::recipient::build_recipient;
use crate::documents::typst::compile_template;
use findmyjob_core::docgen::selection::select;
use findmyjob_core::models::{
    CandidatureDetail, CvContent, CvLabels, ExperienceView, FormationView, ItemBullets, LmContent, LmLabels,
    MatchingResult, OfferView, PeriodGranularity, PeriodStyle, ProfileDto, ProjetView, Recipient, SkillsGroup,
    UserView,
};

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
        adresse: Some("1 rue de l'Exemple".to_string()),
        code_postal: Some("69000".to_string()),
        permis: Some(true),
        description: None,
        linkedin: Some("https://www.linkedin.com/in/camille-martin/".to_string()),
        github: None,
        portfolio: None,
        interets: Some("• Escalade\n\nPhotographie".to_string()),
        created_at: now,
        updated_at: now,
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
            description: None,
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
                description: None,
                stack: None,
                url: None,
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

fn empty_matching() -> MatchingResult {
    MatchingResult {
        relevant_experiences: Vec::new(),
        relevant_projects: Vec::new(),
        relevant_skills: Vec::new(),
        summary_strategy: String::new(),
        gaps: Vec::new(),
    }
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

fn cv_content(project_ids: &[i64]) -> CvContent {
    CvContent {
        title: "Ingénieure data".to_string(),
        summary: "Résumé avec un @ et un # et des *étoiles*.".to_string(),
        experiences: vec![ItemBullets { id: 10, bullets: vec!["A conçu un pipeline.".to_string()] }],
        projects: project_ids.iter().map(|id| ItemBullets { id: *id, bullets: vec![format!("Fait {id}.")] }).collect(),
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
fn destinataire_repli_sur_l_offre() {
    let candidature: CandidatureDetail = serde_json::from_value(serde_json::json!({
        "id": 1, "session_id": 1, "offer_id": 5, "poste": "Stage", "entreprise": null, "localisation": "  ",
        "status": "a_postuler", "source": null, "type_contrat": null, "remuneration": null, "url_offre": null,
        "description_offre": null, "notes": null, "date_candidature": null, "cv_path": null,
        "lettre_motivation_path": null, "cv_reference_version": null, "lm_reference_version": null,
        "created_at": "2026-01-01T00:00:00Z"
    }))
    .expect("candidature de fixture");
    let offer: OfferView = serde_json::from_value(serde_json::json!({
        "id": 5, "session_id": 1, "source": "test", "title": "Stage", "company": "Société Exemple",
        "location": "Lyon", "url": "https://example.org/offre", "contract_type": null, "posted_at": null,
        "description": null, "status": "new", "score": null, "created_at": "2026-01-01T00:00:00Z"
    }))
    .expect("offre de fixture");

    let recipient = build_recipient(&candidature, Some(&offer), &[]);
    assert_eq!(recipient.company, "Société Exemple");
    assert_eq!(recipient.location, "Lyon");
    assert_eq!(recipient.contact_name, "");
}

// ======================================================================
// build_cv_data / build_lm_data
// ======================================================================

#[test]
fn cv_data_fusionne_profil_contenu_et_libelles() {
    let profile = profile();
    let selection = select(&profile, &empty_matching(), 1);
    let (labels, style) = (cv_labels(), period_style());
    let layout = CvLayout { labels: &labels, period_style: &style, separator: " – ", lang: "fr" };

    let data = build_cv_data(&profile, &selection, &cv_content(&[20]), &layout);

    assert_eq!(data.identity.linkedin, "linkedin.com/in/camille-martin");
    assert_eq!(data.experiences[0].poste, "Stage data");
    assert_eq!(data.experiences[0].entreprise, "Société Exemple");
    assert_eq!(data.experiences[0].period, "Nov. 2025 – Avr. 2026");
    assert_eq!(data.experiences[0].bullets, vec!["A conçu un pipeline.".to_string()]);
    assert_eq!(data.projects.len(), 1);
    assert_eq!(data.projects[0].titre, "Projet A");
    assert_eq!(data.projects[0].period, "2025");
    assert_eq!(data.formations[0].period, "2022 – 2027");
    assert_eq!(data.formations[0].details, vec!["Spécialité data".to_string(), "Échange à l'étranger".to_string()]);
    assert_eq!(data.interets, vec!["Escalade".to_string(), "Photographie".to_string()]);
    assert_eq!(data.lang, "fr");
}

#[test]
fn lm_data_date_et_destinataire() {
    let recipient = Recipient { company: "Société Exemple".to_string(), ..Default::default() };
    let content = LmContent { subject: "Candidature".to_string(), paragraphs: vec!["Paragraphe.".to_string()] };
    let labels = LmLabels { date_prefix: "le ".to_string(), ..Default::default() };

    let data = build_lm_data(&profile(), &recipient, &content, &labels, date(2026, 10, 9), "fr");

    assert_eq!(data.date, "9 octobre 2026");
    assert_eq!(data.recipient.company, "Société Exemple");
    assert_eq!(data.identity.adresse, "1 rue de l'Exemple");
    assert_eq!(data.paragraphs, vec!["Paragraphe.".to_string()]);
}

// ======================================================================
// Compilation d'un template de fixture
// ======================================================================

#[test]
fn template_compile_avec_donnees_completes() {
    let profile = profile();
    let selection = select(&profile, &empty_matching(), 2);
    let (labels, style) = (cv_labels(), period_style());
    let layout = CvLayout { labels: &labels, period_style: &style, separator: " – ", lang: "fr" };
    let data = build_cv_data(&profile, &selection, &cv_content(&[20, 21]), &layout);

    let doc = compile_cv(&data);

    assert_eq!(doc.page_count, 1);
    assert!(doc.text.contains("CAMILLE MARTIN"), "{}", doc.text);
    assert!(doc.text.contains("Permis B"), "{}", doc.text);
    assert!(doc.text.contains("@"), "le @ du résumé reste du texte : {}", doc.text);
    assert!(doc.text.contains("Projet B"), "{}", doc.text);
}

#[test]
fn template_compile_sans_champs_optionnels() {
    let mut profile = profile();
    profile.user.email = None;
    profile.user.linkedin = None;
    profile.user.permis = Some(false);
    profile.user.interets = None;
    let selection = select(&profile, &empty_matching(), 0);
    let (labels, style) = (cv_labels(), period_style());
    let layout = CvLayout { labels: &labels, period_style: &style, separator: " – ", lang: "fr" };
    let data = build_cv_data(&profile, &selection, &cv_content(&[]), &layout);

    let doc = compile_cv(&data);

    assert!(doc.text.contains("Lyon"), "{}", doc.text);
    assert!(!doc.text.contains("Permis"), "{}", doc.text);
    assert!(!doc.text.contains("Projets"), "section vide masquée : {}", doc.text);
    assert!(!doc.text.contains("Intérêts"), "{}", doc.text);
    assert!(!doc.text.trim_end().ends_with('•'), "pas de séparateur orphelin : {}", doc.text);
}
