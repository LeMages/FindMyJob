//! Tests de l'assemblage déterministe (sans LLM ni base) : construction de
//! `cv_data` / `lm_data` depuis un profil fictif, destinataire, et
//! compilation des références de test avec données complètes puis avec
//! champs optionnels absents.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use chrono::{NaiveDate, Utc};

// Importation des modules internes
use crate::docgen::build_cv_data::{build_cv_data, CvLayout};
use crate::docgen::build_lm_data::{build_lm_data, LmLayout};
use crate::docgen::contact_fields::{missing_contacts, ContactField};
use crate::docgen::entry_texts::source_texts;
use crate::docgen::identity::{display_url, validate_sender_name, SenderLocation};
use crate::docgen::recipient::{build_recipient, split_location};
use crate::docgen::period_format::PeriodFormat;
use crate::docgen::selection::{select, Selection};
use crate::documents::typst::{compile_contract, CompiledDocument};
use crate::models::{
    CandidatureDetail, CvContent, CvData, ExperienceView, FormationView, Identity, LmContent, MonthStyle,
    PeriodGranularity, PeriodGranularityBySection, PeriodSpec, ProfileDto, ProjetView, Recipient, SkillsGroup,
    UserView,
};

/// Localisation de l'expéditeur des tests (celle d'une session).
fn sender() -> SenderLocation {
    SenderLocation { ville: "Lyon".to_string(), zip_code: "69000".to_string() }
}

/// Référence de CV de test, au contrat FindMyJob.
const CV_TEMPLATE: &str = include_str!("../../tests/fixtures/references/cv_reference.typ");

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
        permis: Some(true),
        description: None,
        linkedin: Some("https://www.linkedin.com/in/camille-martin/".to_string()),
        github: None,
        portfolio: None,
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
            contexte: None,
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
                description: Some("- Fait 20.".to_string()),
                stack: Some("Rust, Typst".to_string()),
                url: Some("https://github.com/camille/projet-a/".to_string()),
                pour_qui: Some("École".to_string()),
                date_debut: Some(date(2025, 1, 1)),
                date_fin: Some(date(2025, 6, 1)),
                contexte: None,
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
                contexte: None,
            },
        ],
        skills: Vec::new(),
        langues: Vec::new(),
    }
}

/// Assemble le CV d'un profil, textes des entrées tirés du profil.
fn cv_data(profile: &ProfileDto, selection: &Selection) -> CvData {
    let (spec, location) = (period_spec(), sender());
    let format = PeriodFormat::new(&spec, "fr");
    let layout = CvLayout {
        location: &location,
        granularity: &spec.granularity,
        period_format: &format,
        lang: "fr",
        list_marker: "•",
    };
    build_cv_data(profile, selection, &source_texts(profile, selection), &cv_content(), &layout)
}

/// Format des périodes de la référence de test : mois abrégé pour les
/// expériences, année ailleurs.
fn period_spec() -> PeriodSpec {
    PeriodSpec {
        granularity: PeriodGranularityBySection {
            experiences: PeriodGranularity::Month,
            projects: PeriodGranularity::Year,
            formations: PeriodGranularity::Year,
        },
        separator: " – ".to_string(),
        month_style: MonthStyle::Short,
        numeric_separator: "/".to_string(),
    }
}

fn cv_content() -> CvContent {
    CvContent {
        title: "Ingénieure data".to_string(),
        summary: "Résumé avec un @ et un # et des *étoiles*.".to_string(),
        skills_groups: vec![SkillsGroup { category: "Langages".to_string(), skills: vec!["Rust".to_string()] }],
    }
}

/// Compile une référence de test avec des données.
fn compile_with(template: &str, data: &impl serde::Serialize) -> CompiledDocument {
    let json = serde_json::to_string(data).expect("données sérialisables");
    compile_contract(template, Some(&json), Path::new("introuvable"))
        .unwrap_or_else(|failure| panic!("la référence de test compile : {}", failure.message()))
}

/// Compile la référence de CV de test avec des données.
fn compile_cv(data: &impl serde::Serialize) -> CompiledDocument {
    compile_with(CV_TEMPLATE, data)
}

// ======================================================================
// Identité, lignes du profil, destinataire
// ======================================================================

#[test]
fn liens_affichables() {
    let url = Some("https://www.linkedin.com/in/camille-martin/".to_string());
    assert_eq!(display_url(url.as_ref()), "linkedin.com/in/camille-martin");
    assert_eq!(display_url(None), "");
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
    assert!(recipient.extra_lines.is_empty());
}

#[test]
fn destinataire_adresse_decoupee_et_localisation_absente_sans_ligne_vide() {
    let candidature = |localisation: serde_json::Value| -> CandidatureDetail {
        serde_json::from_value(serde_json::json!({
            "id": 1, "session_id": 1, "poste": "Stage", "entreprise": "Société Exemple", "localisation": localisation,
            "status": "a_postuler", "source": null, "type_contrat": null, "remuneration": null, "url_offre": null,
            "description_offre": null, "notes": null, "date_candidature": null, "cv_path": null,
            "lettre_motivation_path": null, "cv_reference_version": null, "lm_reference_version": null,
            "generation_warnings": [],
            "created_at": "2026-01-01T00:00:00Z"
        }))
        .expect("candidature de fixture")
    };

    let recipient = build_recipient(&candidature(serde_json::json!("12 rue de la Paix, 75002 Paris")));
    assert_eq!(recipient.extra_lines, vec!["12 rue de la Paix".to_string()]);
    assert_eq!(recipient.location, "75002 Paris");

    let recipient = build_recipient(&candidature(serde_json::Value::Null));
    assert!(recipient.extra_lines.is_empty() && recipient.location.is_empty(), "{recipient:?}");
}

// ======================================================================
// build_cv_data / build_lm_data
// ======================================================================

#[test]
fn cv_data_fusionne_profil_et_contenu() {
    let profile = profile();
    let selection = select(&profile, &[20]);

    let data = cv_data(&profile, &selection);

    assert_eq!(data.identity.linkedin, "linkedin.com/in/camille-martin");
    assert_eq!(data.experiences[0].poste, "Stage data");
    assert_eq!(data.experiences[0].entreprise, "Société Exemple");
    assert_eq!(data.experiences[0].period, "Nov. 2025 - Avr. 2026", "séparateur déclaré, tiret normalisé");
    assert_eq!(data.experiences[0].intro, "A conçu un pipeline.", "ligne sans tiret avant la liste = intro");
    assert_eq!(
        data.experiences[0].bullets,
        vec!["Mis en production chez le client".to_string()],
        "bullets = éléments « - » de la description, sans le tiret"
    );
    assert_eq!((data.formations[0].intro.as_str(), data.formations[0].details.as_slice()), ("Spécialité data", &["Échange à l'étranger".to_string()][..]));
    assert_eq!(data.list_marker, "•", "symbole du format de la session");
    assert_eq!(data.projects.len(), 1);
    assert_eq!(data.projects[0].titre, "Projet A");
    assert_eq!(data.projects[0].period, "2025");
    assert_eq!(data.projects[0].bullets, vec!["Fait 20.".to_string()]);
    assert_eq!(data.projects[0].stack, "Rust, Typst");
    assert_eq!(data.projects[0].url, "github.com/camille/projet-a");
    assert_eq!(data.formations[0].period, "2022 - 2027");
    assert_eq!(data.lang, "fr");
}

#[test]
fn localisation_decoupee_en_lignes_d_adresse() {
    let cases: [(&str, &[&str], &str); 6] = [
        ("Paris", &[], "Paris"),
        ("12 rue de la Paix, 75002 Paris", &["12 rue de la Paix"], "75002 Paris"),
        ("Paris, France", &["Paris"], "France"),
        ("", &[], ""),
        (" , Paris ,, ", &[], "Paris"),
        ("Rue A,  Bât. B , 69003 Lyon", &["Rue A", "Bât. B"], "69003 Lyon"),
    ];
    for (input, extra, location) in cases {
        let (lines, last) = split_location(input);
        assert_eq!((lines, last.as_str()), (extra.iter().map(|s| s.to_string()).collect::<Vec<_>>(), location), "{input:?}");
    }
}

#[test]
fn lm_data_date_et_destinataire() {
    let recipient = Recipient { company: "Société Exemple".to_string(), ..Default::default() };
    let content = LmContent { subject: "Candidature".to_string(), paragraphs: vec!["Paragraphe.".to_string()] };

    let location = sender();
    let data = build_lm_data(&profile(), &recipient, &content, &lm_layout(&location));

    assert_eq!(data.date, "9 octobre 2026");
    assert_eq!(data.recipient.company, "Société Exemple");
    assert_eq!(data.identity.ville, "Lyon");
    assert_eq!(data.identity.title, "Ingénieure data", "sous-titre repris du CV");
    assert_eq!(data.identity.zip_code, "69000");
    assert_eq!(data.paragraphs, vec!["Paragraphe.".to_string()]);
}

/// Mise en page de la lettre de test : sous-titre du CV, 9 octobre 2026.
fn lm_layout(location: &SenderLocation) -> LmLayout<'_> {
    LmLayout { location, title: "Ingénieure data", date: date(2026, 10, 9), lang: "fr" }
}

// ======================================================================
// Identité sans repli sur le document de référence
// ======================================================================

/// Identité d'exemple d'une référence : toutes les coordonnées affichées.
fn example_identity() -> Identity {
    Identity {
        prenom: "Camille".to_string(),
        nom: "Exemple".to_string(),
        email: "camille@exemple.invalid".to_string(),
        telephone: "00 00 00 00 00".to_string(),
        linkedin: "linkedin.com/in/exemple".to_string(),
        github: "github.com/exemple".to_string(),
        portfolio: "exemple.invalid".to_string(),
        ..Default::default()
    }
}

#[test]
fn les_coordonnees_vides_du_profil_restent_vides_dans_le_cv_et_la_lettre() {
    let mut profile = profile();
    profile.user.telephone = None;
    profile.user.email = None;

    let cv = cv_data(&profile, &select(&profile, &[]));
    let content = LmContent { subject: "Objet".to_string(), paragraphs: vec!["P.".to_string()] };
    let location = sender();
    let lm = build_lm_data(&profile, &Recipient::default(), &content, &lm_layout(&location));

    for identity in [&cv.identity, &lm.identity] {
        assert_eq!(identity.prenom, "Camille", "le profil fournit le prénom");
        assert_eq!((identity.telephone.as_str(), identity.email.as_str()), ("", ""));
        assert_eq!((identity.github.as_str(), identity.portfolio.as_str()), ("", ""));
    }
    let json = serde_json::to_string(&(&cv, &lm)).expect("données sérialisables");
    assert!(!json.contains("00 00 00 00 00") && !json.contains("exemple.invalid"), "valeur d'exemple : {json}");
}

#[test]
fn avertissement_seulement_si_la_reference_affiche_la_coordonnee() {
    let mut user = user();
    user.telephone = None;
    user.portfolio = None;
    let affiche = example_identity();
    let sans_telephone = Identity { telephone: String::new(), ..example_identity() };

    let missing = missing_contacts(&[&affiche], &user);
    assert_eq!(missing, vec![ContactField::Telephone, ContactField::Github, ContactField::Portfolio]);

    let missing = missing_contacts(&[&sans_telephone], &user);
    assert_eq!(missing, vec![ContactField::Github, ContactField::Portfolio], "téléphone non affiché : pas d'avertissement");
    assert!(missing_contacts(&[&Identity::default()], &user).is_empty(), "référence sans coordonnée");
}

#[test]
fn un_seul_avertissement_par_coordonnee_pour_le_cv_et_la_lettre() {
    let mut user = user();
    user.telephone = None;
    let cv = example_identity();
    let lm = example_identity();

    let missing = missing_contacts(&[&cv, &lm], &user);

    assert_eq!(missing.iter().filter(|f| **f == ContactField::Telephone).count(), 1);
    assert!(!ContactField::Telephone.warning().to_lowercase().contains("lettre"), "ne nomme pas le document");
}

#[test]
fn prenom_ou_nom_vide_refuse_la_generation() {
    let mut user = user();
    assert!(validate_sender_name(&user).is_ok());
    user.prenom = Some("  ".to_string());
    let error = validate_sender_name(&user).expect_err("prénom vide");
    assert!(error.to_string().contains("prénom"), "{error}");
    user.prenom = Some("Camille".to_string());
    user.nom = None;
    let error = validate_sender_name(&user).expect_err("nom absent");
    assert!(error.to_string().contains("nom"), "{error}");
}

#[test]
fn permis_non_renseigne_retire_sans_repli_et_faux_retire_en_silence() {
    let mut profile = profile();
    profile.user.permis = None;
    assert!(!cv_data(&profile, &select(&profile, &[])).identity.permis, "NULL : jamais repris de la référence");
    profile.user.permis = Some(false);
    assert!(!cv_data(&profile, &select(&profile, &[])).identity.permis);
    profile.user.permis = Some(true);
    assert!(cv_data(&profile, &select(&profile, &[])).identity.permis);
}

#[test]
fn le_lieu_vient_de_la_session() {
    let profile = profile();
    let spec = period_spec();
    let format = PeriodFormat::new(&spec, "fr");
    let build = |location: &SenderLocation| {
        let layout = CvLayout {
            location,
            granularity: &spec.granularity,
            period_format: &format,
            lang: "fr",
            list_marker: "•",
        };
        build_cv_data(&profile, &select(&profile, &[]), &source_texts(&profile, &select(&profile, &[])), &cv_content(), &layout)
    };

    let avec_lieu = build(&sender());
    assert_eq!((avec_lieu.identity.ville.as_str(), avec_lieu.identity.zip_code.as_str()), ("Lyon", "69000"));
    let sans_lieu = build(&SenderLocation::default());
    assert_eq!((sans_lieu.identity.ville.as_str(), sans_lieu.identity.zip_code.as_str()), ("", ""), "pas de lieu inventé");
}

#[test]
fn sections_vides_sans_contenu_de_la_reference() {
    let mut profile = profile();
    profile.experiences.clear();
    profile.projets.clear();
    profile.formations.clear();

    let data = cv_data(&profile, &select(&profile, &[]));

    assert!(data.experiences.is_empty(), "profil sans expérience : section vide");
    assert!(data.projects.is_empty(), "profil sans projet : section vide");
    assert!(data.formations.is_empty(), "profil sans formation : section vide");
}

#[test]
fn suite_de_ligne_dans_le_meme_element() {
    let mut profile = profile();
    profile.experiences[0].description = Some("Intro 1\nIntro 2\n- A\nsuite de A\n- B".to_string());
    let data = cv_data(&profile, &select(&profile, &[]));
    assert_eq!(data.experiences[0].intro, "Intro 1\nIntro 2", "retours à la ligne de l'intro conservés");
    assert_eq!(data.experiences[0].bullets, vec!["A\nsuite de A".to_string(), "B".to_string()]);
}

#[test]
fn description_sans_tiret_reste_un_bloc() {
    let mut profile = profile();
    profile.experiences[0].description = Some("Une seule phrase.\nEt une autre.".to_string());
    let data = cv_data(&profile, &select(&profile, &[]));
    assert_eq!(data.experiences[0].intro, "Une seule phrase.\nEt une autre.");
    assert!(data.experiences[0].bullets.is_empty(), "aucun élément sans tiret");
}

#[test]
fn entree_sans_description_reste_sans_bullet() {
    let profile = profile();
    let data = cv_data(&profile, &select(&profile, &[21]));
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
    let (spec, location) = (period_spec(), sender());
    let format = PeriodFormat::new(&spec, "en");
    let layout = CvLayout {
        location: &location,
        granularity: &spec.granularity,
        period_format: &format,
        lang: "en",
        list_marker: "-",
    };

    let data = build_cv_data(&profile, &selection, &texts, &cv_content(), &layout);

    // Les lignes traduites gardent la forme de l'original : intro, puis éléments.
    assert_eq!(data.experiences[0].intro, "Built a pipeline.");
    assert_eq!(data.experiences[0].bullets, vec!["Deployed it for the client".to_string()]);
    assert_eq!(data.projects[0].bullets, vec!["Did 20.".to_string()]);
    assert_eq!(data.projects[0].pour_qui, "School");
    assert_eq!(data.formations[0].intro, "Data major");
    assert_eq!(data.formations[0].details, vec!["Exchange abroad".to_string()]);
    assert_eq!(data.list_marker, "-");
    assert_eq!(data.experiences[0].period, "Nov 2025 - Apr 2026", "mois dans la langue du document");
    assert_eq!(data.experiences[0].poste, "Stage data", "poste jamais traduit");
    assert_eq!(data.experiences[0].entreprise, "Société Exemple");
    assert_eq!(data.projects[0].titre, "Projet A", "titre de projet jamais traduit");
    assert_eq!(data.projects[0].stack, "Rust, Typst");
    assert_eq!(data.formations[0].titre, "Diplôme d'ingénieur");
    assert_eq!(data.formations[0].ecole, "École Exemple");
}

// ======================================================================
// Compilation de la référence de test
// ======================================================================

#[test]
fn template_compile_avec_donnees_completes() {
    let profile = profile();
    let data = cv_data(&profile, &select(&profile, &[20, 21]));

    let doc = compile_cv(&data);

    assert_eq!(doc.page_count, 1);
    assert!(doc.text.contains("CAMILLE MARTIN"), "{}", doc.text);
    assert!(doc.text.contains("Permis B"), "{}", doc.text);
    assert!(doc.text.contains("@"), "le @ du résumé reste du texte : {}", doc.text);
    assert!(doc.text.contains("Projet B"), "{}", doc.text);
    assert!(doc.text.contains("Fait 20."), "{}", doc.text);
}

#[test]
fn stack_et_url_affiches_seulement_s_ils_sont_renseignes() {
    let profile = profile();
    let with = compile_cv(&cv_data(&profile, &select(&profile, &[20])));
    assert!(with.text.contains("Rust, Typst"), "{}", with.text);
    assert!(with.text.contains("github.com/camille/projet-a"), "{}", with.text);

    let without = compile_cv(&cv_data(&profile, &select(&profile, &[21])));
    assert!(!without.text.contains("Typst"), "{}", without.text);
    assert!(!without.text.contains("github.com"), "{}", without.text);
}

#[test]
fn template_compile_sans_champs_optionnels() {
    let mut profile = profile();
    profile.user.email = None;
    profile.user.linkedin = None;
    profile.user.permis = Some(false);
    let data = cv_data(&profile, &select(&profile, &[]));

    let doc = compile_cv(&data);

    assert!(doc.text.contains("Lyon"), "{}", doc.text);
    assert!(!doc.text.contains("Permis"), "{}", doc.text);
    assert!(!doc.text.contains("Projets"), "section vide masquée : {}", doc.text);
    assert!(!doc.text.trim_end().ends_with('•'), "pas de séparateur orphelin : {}", doc.text);
}

// ======================================================================
// Lieu de la lettre : en-tête et ligne de date
// ======================================================================

/// Référence de lettre de test, au contrat FindMyJob.
const LM_TEMPLATE: &str = include_str!("../../tests/fixtures/references/lm_reference.typ");

/// Compile la lettre de fixture pour un lieu donné, texte rendu aux espaces
/// normalisés.
fn lm_text(ville: &str, zip_code: &str) -> String {
    let profile = profile();
    let location = SenderLocation { ville: ville.to_string(), zip_code: zip_code.to_string() };
    let recipient = Recipient { company: "Société Exemple".to_string(), ..Default::default() };
    let content = LmContent { subject: "Candidature".to_string(), paragraphs: vec!["Paragraphe.".to_string()] };
    let data = build_lm_data(&profile, &recipient, &content, &lm_layout(&location));

    compile_with(LM_TEMPLATE, &data).text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn lettre_ville_et_code_postal_dans_l_en_tete_ville_seule_dans_la_date() {
    let text = lm_text("Lyon", "69003");
    assert!(text.contains("Lyon, 69003"), "{text}");
    assert!(text.contains("Lyon, le 9 octobre 2026"), "{text}");
    assert!(!text.contains("69003, le"), "le code postal n'est pas dans la ligne de date : {text}");
}

#[test]
fn lettre_sans_ville_ni_separateur_orphelin() {
    let text = lm_text("", "69003");
    assert!(text.contains("le 9 octobre 2026"), "{text}");
    assert!(!text.contains(", le"), "pas de virgule orpheline devant la date : {text}");
    assert!(!text.contains(", 69003"), "pas de virgule orpheline devant le code postal : {text}");
}

#[test]
fn reference_d_une_version_precedente_a_reimporter() {
    use crate::documents::reference_template::load_contract;
    use crate::error::AppError;

    let dir = std::env::temp_dir().join(format!("findmyjob-contract-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dossier temporaire");
    let load = |path: &Path| load_contract::<serde_json::Value>(path, "CV");

    assert!(matches!(load(&dir.join("absent.json")), Err(AppError::OutdatedReference(_))), "fichier absent : à réimporter");

    let other = dir.join("autre_version.json");
    std::fs::write(&other, r#"{"contract_version": 99}"#).expect("écriture");
    assert!(matches!(load(&other), Err(AppError::OutdatedReference(_))), "autre version du contrat : à réimporter");

    let corrupted = dir.join("corrompu.json");
    std::fs::write(&corrupted, "{ pas du json").expect("écriture");
    assert!(matches!(load(&corrupted), Err(AppError::Validation(_))), "JSON illisible : corrompu, pas obsolète");

    let truncated = dir.join("incomplet.json");
    std::fs::write(&truncated, r#"{"contract_version": 1}"#).expect("écriture");
    assert!(matches!(load(&truncated), Err(AppError::Validation(_))), "champs manquants : corrompu");

    let current = dir.join("courant.json");
    std::fs::write(
        &current,
        r#"{"contract_version": 1, "languages": ["fr", "en"], "example": {}, "guidance": "", "slots": [],
            "period": null, "page_count": 1, "example_text": {"fr": "Texte"}}"#,
    )
    .expect("écriture");
    assert_eq!(load(&current).expect("version courante").example_text["fr"], "Texte");
    std::fs::remove_dir_all(&dir).expect("nettoyage");
}
