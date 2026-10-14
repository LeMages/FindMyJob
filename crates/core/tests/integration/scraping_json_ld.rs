//! Tests de l'étage JSON-LD de la cascade d'extraction
//! (`findmyjob_core::scraping::json_ld`), sur des pages synthétiques : objet
//! simple, tableau, `@graph`, salaires, lieux multiples, entités HTML,
//! contact de candidature.

// Importation des modules internes
use findmyjob_core::scraping::json_ld::{extract_job_posting_fields, find_job_posting};

// ======================================================================
// Helpers
// ======================================================================

/// Description HTML d'une offre fictive, assez longue pour valider l'étage.
const DESCRIPTION_HTML: &str = "<h2>Missions</h2><ul><li>Concevoir des tableaux de bord pour les équipes métier</li>\
<li>Automatiser la préparation des données de vente</li></ul><h2>Profil recherché</h2>\
<p>Étudiant en école d'ingénieur ou master data, à l'aise avec SQL et Python, curieux et rigoureux.</p>";

/// Construit une page HTML contenant les blocs JSON-LD donnés.
fn page_with_json_ld(blocks: &[&str]) -> String {
    let scripts: String = blocks
        .iter()
        .map(|b| format!("<script type=\"application/ld+json\">{b}</script>"))
        .collect();
    format!("<!doctype html><html><head><title>Offre</title>{scripts}</head><body><div id=\"root\"></div></body></html>")
}

/// Construit un `JobPosting` JSON minimal avec les champs supplémentaires donnés.
fn posting(extra: &str) -> String {
    let description = serde_json::to_string(DESCRIPTION_HTML).expect("chaîne JSON");
    format!(
        r#"{{"@context":"https://schema.org","@type":"JobPosting","title":"Stage Data Analyst",
        "hiringOrganization":{{"@type":"Organization","name":"Exemple SAS"}},
        "description":{description}{extra}}}"#
    )
}

// ======================================================================
// Recherche du JobPosting
// ======================================================================

#[test]
fn objet_simple() {
    let html = page_with_json_ld(&[&posting(
        r#","jobLocation":{"@type":"Place","address":{"@type":"PostalAddress","addressLocality":"Lyon","addressCountry":"FR"}}"#,
    )]);
    let fields = extract_job_posting_fields(&html).expect("étage JSON-LD réussi");
    assert_eq!(fields.poste.as_deref(), Some("Stage Data Analyst"));
    assert_eq!(fields.entreprise.as_deref(), Some("Exemple SAS"));
    assert_eq!(fields.localisation.as_deref(), Some("Lyon"));
    assert_eq!(fields.type_contrat.as_deref(), Some("stage"));
    assert_eq!(fields.remuneration, None);
    let description = fields.description_offre.expect("description");
    assert!(description.contains("Missions"));
    assert!(description.contains("- Concevoir des tableaux de bord"));
}

#[test]
fn tableau_de_blocs_avec_type_en_tableau() {
    let breadcrumb = r#"{"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[]}"#;
    let job = posting("").replace(r#""@type":"JobPosting""#, r#""@type":["JobPosting","Thing"]"#);
    let html = page_with_json_ld(&[&format!("[{breadcrumb},{job}]")]);
    let fields = extract_job_posting_fields(&html).expect("JobPosting trouvé dans le tableau");
    assert_eq!(fields.poste.as_deref(), Some("Stage Data Analyst"));
}

#[test]
fn graph_et_plusieurs_blocs() {
    let organization = r#"{"@context":"https://schema.org","@type":"Organization","name":"Autre"}"#;
    let graph = format!(r#"{{"@context":"https://schema.org","@graph":[{{"@type":"WebPage"}},{}]}}"#, posting(""));
    let html = page_with_json_ld(&[organization, &graph]);
    let fields = extract_job_posting_fields(&html).expect("JobPosting trouvé dans @graph");
    assert_eq!(fields.entreprise.as_deref(), Some("Exemple SAS"));
}

#[test]
fn json_avec_retours_a_la_ligne_bruts() {
    // Retour à la ligne brut dans une chaîne : JSON invalide, toléré.
    let job = posting("").replace("Stage Data Analyst", "Stage Data\nAnalyst");
    let html = page_with_json_ld(&[&job]);
    let fields = extract_job_posting_fields(&html).expect("JSON-LD toléré");
    assert_eq!(fields.poste.as_deref(), Some("Stage Data Analyst"));
}

#[test]
fn page_sans_job_posting() {
    let html = page_with_json_ld(&[r#"{"@type":"Organization","name":"Exemple"}"#]);
    assert!(find_job_posting(&html).is_none());
    assert!(extract_job_posting_fields(&html).is_none());
}

#[test]
fn description_trop_courte_fait_echouer_l_etage() {
    let html = page_with_json_ld(&[
        r#"{"@type":"JobPosting","title":"Stage Data Analyst","description":"<p>Postulez vite.</p>"}"#,
    ]);
    assert!(extract_job_posting_fields(&html).is_none());
}

#[test]
fn titre_absent_fait_echouer_l_etage() {
    let job = posting("").replace(r#""title":"Stage Data Analyst","#, "");
    assert!(extract_job_posting_fields(&page_with_json_ld(&[&job])).is_none());
}

// ======================================================================
// Rémunération
// ======================================================================

#[test]
fn salaire_vide() {
    let html = page_with_json_ld(&[&posting(r#","baseSalary":{}"#)]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(fields.remuneration, None);
}

#[test]
fn salaire_objet_quantitative_value_vide() {
    let html = page_with_json_ld(&[&posting(
        r#","baseSalary":{"@type":"MonetaryAmount","currency":"EUR","value":{"@type":"QuantitativeValue"}}"#,
    )]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(fields.remuneration, None);
}

#[test]
fn salaire_au_format_francais() {
    let html = page_with_json_ld(&[&posting(
        r#","baseSalary":{"@type":"MonetaryAmount","currency":"EUR","value":{"@type":"QuantitativeValue","value":"1 200,50","unitText":"MONTH"}}"#,
    )]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(fields.remuneration.as_deref(), Some("1 200,50 € par mois"));
}

#[test]
fn salaire_en_fourchette() {
    let html = page_with_json_ld(&[&posting(
        r#","baseSalary":{"@type":"MonetaryAmount","currency":"EUR","value":{"@type":"QuantitativeValue","minValue":35000,"maxValue":"40000.00","unitText":"YEAR"}}"#,
    )]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(fields.remuneration.as_deref(), Some("35 000 – 40 000 € par an"));
}

#[test]
fn salaire_nul_vaut_absence() {
    let html = page_with_json_ld(&[&posting(
        r#","baseSalary":{"@type":"MonetaryAmount","currency":"EUR","value":{"@type":"QuantitativeValue","value":0,"unitText":"MONTH"}}"#,
    )]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(fields.remuneration, None);
}

// ======================================================================
// Localisation
// ======================================================================

#[test]
fn lieux_multiples_et_teletravail() {
    let html = page_with_json_ld(&[&posting(
        r#","jobLocationType":"TELECOMMUTE","jobLocation":[
            {"@type":"Place","address":{"addressLocality":"Paris","addressCountry":"FR"}},
            {"@type":"Place","address":{"addressRegion":"Auvergne-Rhône-Alpes"}},
            {"@type":"Place","address":{"addressCountry":{"@type":"Country","name":"Belgique"}}}
        ]"#,
    )]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(
        fields.localisation.as_deref(),
        Some("Télétravail + Paris / Auvergne-Rhône-Alpes / Belgique")
    );
}

#[test]
fn teletravail_seul() {
    let html = page_with_json_ld(&[&posting(r#","jobLocationType":["TELECOMMUTE"]"#)]);
    let fields = extract_job_posting_fields(&html).expect("étage réussi");
    assert_eq!(fields.localisation.as_deref(), Some("Télétravail"));
}

// ======================================================================
// Contact de candidature
// ======================================================================

#[test]
fn contact_de_candidature_nomme() {
    let html = page_with_json_ld(&[&posting(
        r#","applicationContact":{"@type":"ContactPoint","name":" Camille Martin ","contactType":"Recrutement",
            "email":"mailto:camille.martin@example.com","telephone":"+33 6 00 00 00 00"}"#,
    )]);
    let contact = extract_job_posting_fields(&html).expect("étage réussi").contact.expect("contact");
    assert_eq!(contact.nom.as_deref(), Some("Camille Martin"));
    assert_eq!(contact.role.as_deref(), Some("Recrutement"));
    assert_eq!(contact.email.as_deref(), Some("camille.martin@example.com"));
    assert_eq!(contact.telephone.as_deref(), Some("+33 6 00 00 00 00"));
}

#[test]
fn contact_sans_nom_ignore() {
    let html = page_with_json_ld(&[&posting(
        r#","applicationContact":{"@type":"ContactPoint","email":"rh@example.com"}"#,
    )]);
    assert_eq!(extract_job_posting_fields(&html).expect("étage réussi").contact, None);
    let sans_contact = page_with_json_ld(&[&posting("")]);
    assert_eq!(extract_job_posting_fields(&sans_contact).expect("étage réussi").contact, None);
}

// ======================================================================
// Description avec entités HTML
// ======================================================================

#[test]
fn description_html_echappee_avec_entites() {
    let escaped = "&lt;h2&gt;Vos missions&lt;/h2&gt;&lt;ul&gt;&lt;li&gt;Analyser les ventes &amp;amp; les stocks&lt;/li&gt;\
&lt;li&gt;Pr&#233;senter les r&eacute;sultats aux &#xE9;quipes&lt;/li&gt;&lt;/ul&gt;\
&lt;p&gt;Vous &#xEA;tes &#xE9;tudiant en &#xE9;cole de commerce ou d&#39;ing&#233;nieur, avec un go&#xFB;t prononc&#xE9; pour les chiffres, l&#39;analyse et la restitution claire des r&#233;sultats aupr&#232;s des directions m&#233;tier.&lt;/p&gt;";
    let job = format!(
        r#"{{"@type":"JobPosting","title":"Stage Business Analyst","description":"{escaped}"}}"#
    );
    let fields = extract_job_posting_fields(&page_with_json_ld(&[&job])).expect("étage réussi");
    let description = fields.description_offre.expect("description");
    assert!(!description.contains('<'), "balise restante :\n{description}");
    assert!(!description.contains("&lt;") && !description.contains("&#"), "entité restante :\n{description}");
    assert!(description.lines().any(|l| l == "Vos missions"), "titre perdu :\n{description}");
    assert!(description.contains("- Présenter les résultats aux équipes"));
    assert!(description.contains("d'ingénieur"));
}
