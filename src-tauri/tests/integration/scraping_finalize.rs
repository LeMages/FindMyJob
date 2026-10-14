//! Tests du nettoyage final de la description d'offre
//! (`findmyjob_lib::scraping::finalize`), pour chacun des trois chemins :
//! JSON-LD, LLM sur la page et texte collé. Dans les trois cas : ni balise,
//! ni emoji, ni décoration Markdown, titres et listes conservés.

// Importation des modules internes
use findmyjob_lib::llm::scraping_extract::{ScrapedContact, ScrapedOfferFields};
use findmyjob_lib::scraping::finalize::finalize_fields;
use findmyjob_lib::scraping::json_ld::extract_job_posting_fields;

// ======================================================================
// Helpers
// ======================================================================

/// Indique si un caractère est un emoji courant (pictogrammes, symboles).
fn is_emoji(c: char) -> bool {
    matches!(c as u32, 0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0xFE0F)
}

/// Vérifie qu'une description est propre : sans balise, emoji ni Markdown,
/// avec les titres et puces attendus.
fn assert_clean(description: &str, headings: &[&str], bullets: &[&str]) {
    assert!(!description.contains('<'), "balise restante :\n{description}");
    assert!(!description.chars().any(is_emoji), "emoji restant :\n{description}");
    for marker in ["**", "__", "##", "```"] {
        assert!(!description.contains(marker), "Markdown restant ({marker}) :\n{description}");
    }
    assert!(!description.lines().any(|l| l.starts_with('#')), "titre Markdown :\n{description}");
    let lines: Vec<&str> = description.lines().collect();
    for heading in headings {
        assert!(lines.contains(heading), "titre perdu {heading:?} :\n{description}");
    }
    for bullet in bullets {
        assert!(lines.contains(&format!("- {bullet}").as_str()), "puce perdue {bullet:?} :\n{description}");
    }
}

/// Champs simulés tels que renvoyés par le LLM, avec la description donnée.
fn llm_fields(description: &str) -> ScrapedOfferFields {
    ScrapedOfferFields {
        poste: Some("  **Stage Développeur**  ".to_string()),
        entreprise: Some("Exemple &amp; Cie".to_string()),
        localisation: Some("Lyon".to_string()),
        type_contrat: Some("Stage".to_string()),
        remuneration: Some("1 200 € par mois".to_string()),
        description_offre: Some(description.to_string()),
        contact: None,
    }
}

// ======================================================================
// Chemin 1 : JSON-LD
// ======================================================================

#[test]
fn chemin_json_ld() {
    let description = "<h2>🚀 Vos missions</h2><ul><li><strong>Concevoir</strong> des API REST ✅</li>\
<li>Écrire des tests **automatisés**</li></ul><h2>Profil recherché</h2>\
<p>Étudiant en informatique, à l'aise avec Rust et SQL, curieux et autonome, vous aimez \
travailler en équipe et partager vos connaissances avec les autres développeurs.</p>";
    let json = serde_json::json!({
        "@type": "JobPosting",
        "title": "Stage Développeur Backend",
        "description": description,
    });
    let html = format!("<html><head><script type=\"application/ld+json\">{json}</script></head><body></body></html>");
    let fields = extract_job_posting_fields(&html).expect("étage JSON-LD réussi");
    assert_clean(
        &fields.description_offre.expect("description"),
        &["Profil recherché"],
        &["Concevoir des API REST", "Écrire des tests automatisés"],
    );
}

// ======================================================================
// Chemin 2 : LLM sur le texte de la page
// ======================================================================

#[test]
fn chemin_llm() {
    let description = "## Vos missions\n\n* **Concevoir** des API REST 🚀\n* Écrire des tests\n\n\
### Profil recherché\n\nÉtudiant en informatique ✨, à l'aise avec __Rust__.";
    let fields = finalize_fields(llm_fields(description));
    assert_clean(
        &fields.description_offre.expect("description"),
        &["Vos missions", "Profil recherché"],
        &["Concevoir des API REST", "Écrire des tests"],
    );
    assert_eq!(fields.poste.as_deref(), Some("Stage Développeur"));
    assert_eq!(fields.entreprise.as_deref(), Some("Exemple & Cie"));
    assert_eq!(fields.type_contrat.as_deref(), Some("stage"));
    assert_eq!(fields.remuneration.as_deref(), Some("1 200 € par mois"));
}

#[test]
fn chemin_llm_avec_balises_residuelles() {
    let description = "<p>Vos missions</p><ul><li>Concevoir des API REST</li></ul>";
    let fields = finalize_fields(llm_fields(description));
    assert_clean(&fields.description_offre.expect("description"), &["Vos missions"], &["Concevoir des API REST"]);
}

// ======================================================================
// Chemin 3 : texte collé (analysé par le LLM, puis même nettoyage)
// ======================================================================

#[test]
fn chemin_texte_colle() {
    // Texte collé depuis une page : puces « • », emojis de section, titre
    // décoré, espaces multiples — repris tel quel par le LLM.
    let description = "*** Vos missions ***\n\n• Concevoir des API REST\n•   Écrire des tests  \n\n\
👉 Profil recherché\nÉtudiant en informatique.";
    let fields = finalize_fields(llm_fields(description));
    assert_clean(
        &fields.description_offre.expect("description"),
        &["Vos missions"],
        &["Concevoir des API REST", "Écrire des tests", "Profil recherché"],
    );
}

#[test]
fn champs_vides_et_type_hors_formulaire() {
    let fields = finalize_fields(ScrapedOfferFields {
        poste: Some("   ".to_string()),
        entreprise: None,
        localisation: Some("<p></p>".to_string()),
        type_contrat: Some("freelance".to_string()),
        remuneration: Some("null".to_string()),
        description_offre: Some("  ".to_string()),
        contact: None,
    });
    assert_eq!(fields, ScrapedOfferFields::default());
}

#[test]
fn contact_nettoye() {
    let mut fields = llm_fields("Missions");
    fields.contact = Some(ScrapedContact {
        nom: Some("  Camille   Martin ".to_string()),
        role: Some("Responsable <b>data</b>".to_string()),
        email: Some(" mailto:camille.martin@example.com ".to_string()),
        telephone: Some("tel:+33 6 00 00 00 00".to_string()),
    });
    let contact = finalize_fields(fields).contact.expect("contact");
    assert_eq!(contact.nom.as_deref(), Some("Camille Martin"));
    assert_eq!(contact.role.as_deref(), Some("Responsable data"));
    assert_eq!(contact.email.as_deref(), Some("camille.martin@example.com"));
    assert_eq!(contact.telephone.as_deref(), Some("+33 6 00 00 00 00"));
}

#[test]
fn contact_sans_nom_ecarte() {
    let mut fields = llm_fields("Missions");
    fields.contact = Some(ScrapedContact {
        nom: Some("   ".to_string()),
        role: None,
        email: Some("rh@example.com".to_string()),
        telephone: None,
    });
    assert_eq!(finalize_fields(fields).contact, None);
}
