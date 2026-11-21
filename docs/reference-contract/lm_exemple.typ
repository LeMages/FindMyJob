// Exemple de lettre de motivation au contrat FindMyJob (Antoine Dupont, personnage fictif).
//
// Structure du fichier :
// 1. `example` : les valeurs d'exemple, dans la forme exacte de `data`.
// 2. `data` : les données injectées par FindMyJob (sys.inputs.data), ou
//    l'exemple quand le fichier est compilé seul.
// 3. La métadonnée <findmyjob> : contrat, langues, consignes, emplacements
//    rédigés.
// 4. `L` : les textes fixes (titres, libellés, phrases) dans chaque langue.
// 5. La mise en page, libre, qui ne lit que `data` et `L`.
//
// Voir contract.fr.md pour la description complète.

#let example = (
  lang: "fr",
  identity: (
    prenom: "Antoine",
    nom: "Dupont",
    title: "Étudiant Ingénieur Informatique, spécialité Data & Cloud",
    email: "antoine.dupont@exemple.fr",
    telephone: "+33 6 00 00 00 00",
    ville: "Lyon",
    zip_code: "69003",
    linkedin: "",
    github: "",
    portfolio: "",
    permis: false,
  ),
  recipient: (
    company: "Exemple SA",
    location: "1201 Genève",
    extra_lines: ("Service des ressources humaines", "Rue de l'Exemple 1"),
  ),
  date: "13 octobre 2026",
  subject: "Candidature pour le poste de Stagiaire Data Engineer",
  paragraphs: (
    "C'est dans ce cadre que je vous adresse ma candidature pour le poste de Stagiaire Data Engineer chez Exemple SA.",
    "Lors de mon stage de 6 mois chez Transvia Logistique, j'ai refondu le pipeline d'ingestion de plus de 300 entrepôts avec Airflow et dbt, ce qui a réduit le temps de traitement de 40% tout en maintenant un taux d'erreur inférieur à 1%. J'y ai également conçu des tableaux de bord de suivi des livraisons adoptés par cinq équipes opérationnelles, ainsi que les API REST exposant ces indicateurs aux outils internes. En parallèle, j'ai développé un projet personnel de recherche sémantique dans des notices techniques, déployé sur Azure avec Terraform et une chaîne d'intégration continue complète.",
    "Intéressé par la fiabilité et la valorisation des données, je souhaite rejoindre l'équipe Data d'Exemple SA afin de contribuer à la construction de sa plateforme de données. Les missions proposées, de l'industrialisation des pipelines d'ingestion à la mise en production de modèles prédictifs, correspondent pleinement à mes compétences et à mes aspirations professionnelles.",
  ),
)

#let data = if "data" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }

#metadata((
  contract: "findmyjob/lettre",
  version: 1,
  languages: ("fr", "en"),
  example: example,
  guidance: "Les faits (expériences, chiffres, technologies, dates) viennent uniquement du profil : ne jamais inventer un fait, un chiffre ou une compétence. La lettre tient sur une page. Le contact recruteur n'est jamais nommé.",
  slots: (
    (field: "subject", mode: "variable", instruction: "Intitulé exact du poste tiré de l'offre."),
    (field: "paragraphs[0]", mode: "variable", instruction: "Intitulé exact du poste et nom de l'entreprise, tirés de l'offre."),
    (field: "paragraphs[1]", mode: "adaptable", instruction: "Choisir et ordonner, parmi les expériences et projets du profil, les réalisations les plus pertinentes pour l'offre. Garder les chiffres exacts du profil. Le projet personnel n'est jamais présenté comme un projet entrepreneurial et n'est pas nommé."),
    (field: "paragraphs[2]", mode: "variable", instruction: "Enjeu de l'entreprise et deux missions reprises de l'offre. Garder la structure : « Intéressé par… je souhaite rejoindre… afin de… Les missions proposées, de … à …, correspondent pleinement… »."),
  ),
)) <findmyjob>

#let L = (
  fr: (
    date_prefix: "le",
    subject_prefix: "Objet :",
    subject_suffix: "Stage de fin d'études de 6 mois dès février 2027",
    salutation: "Madame, Monsieur,",
    intro: "Actuellement étudiant en dernière année du cycle ingénieur, spécialisé en Data et Cloud, je suis à la recherche d'un stage de fin d'études de 6 mois à partir de février 2027. Ce stage, obligatoire pour l'obtention de mon diplôme, peut se dérouler en France ou à l'étranger et fera l'objet d'une convention établie par mon école.",
    international: "Mon semestre d'échange à l'Université de Lisbonne et mon niveau d'anglais (TOEIC 905) me permettent par ailleurs d'évoluer sans difficulté dans un environnement international.",
    closing_paragraph: "Mobile et prêt à m'installer à proximité de votre site, je serai pleinement disponible dès février 2027 pour toute la durée du stage. Autonome, rigoureux et curieux, je serais heureux de mettre mes compétences au service de vos projets et d'en discuter avec vous lors d'un entretien, en présentiel ou à distance.",
    closing: "Avec mes meilleures salutations,",
  ),
  en: (
    date_prefix: "",
    subject_prefix: "Subject:",
    subject_suffix: "6-month end-of-studies internship from February 2027",
    salutation: "Dear Sir or Madam,",
    intro: "I am currently in the final year of an engineering degree, specialising in Data and Cloud, and I am looking for a 6-month end-of-studies internship starting in February 2027. This internship is required to graduate, can take place in France or abroad, and will be covered by an agreement issued by my school.",
    international: "My exchange semester at the University of Lisbon and my level of English (TOEIC 905) also allow me to work with ease in an international environment.",
    closing_paragraph: "I am willing to relocate near your site and will be fully available from February 2027 for the whole internship. Autonomous, rigorous and curious, I would be glad to put my skills to work on your projects and to discuss them with you in an interview, in person or remotely.",
    closing: "Yours faithfully,",
  ),
).at(data.lang)

// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 2cm, bottom: 2cm, left: 2.2cm, right: 2.2cm),
)

#set text(font: "Arimo", size: 10.5pt, fill: rgb("2B2B2B"), lang: data.lang)
#set par(leading: 0.75em, justify: true)
#set text(hyphenate: false)

// === Colors ===
#let accent = rgb("2E5A88")
#let dark = rgb("2B2B2B")
#let gray = rgb("555555")

// === Spacing ===
#let space-para = 15pt

#set par(spacing: space-para)

#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")
#let name = joined((data.identity.prenom, data.identity.nom), " ")

// === Document ===

// --- Expéditeur (gauche) ---
#block({
  set par(justify: false, leading: 0.55em)
  let lines = (
    text(size: 14pt, weight: "bold", fill: dark, name),
    if data.identity.title != "" { text(size: 10pt, fill: accent, data.identity.title) },
    if joined((data.identity.ville, data.identity.zip_code), ", ") != "" {
      text(size: 10pt, fill: gray, joined((data.identity.ville, data.identity.zip_code), ", "))
    },
    if data.identity.telephone != "" { text(size: 10pt, fill: gray, data.identity.telephone) },
    if data.identity.email != "" { text(size: 10pt, fill: gray, data.identity.email) },
  ).filter(line => line != none)
  lines.join(linebreak())
})

#v(10pt)

// --- Destinataire (droite, largeur de son contenu) ---
#align(right, box({
  set align(left)
  set par(justify: false, leading: 0.55em)
  let lines = (
    if data.recipient.company != "" { text(weight: "bold", data.recipient.company) },
    ..data.recipient.extra_lines.filter(line => line != ""),
    if data.recipient.location != "" { data.recipient.location },
  ).filter(line => line != none)
  lines.join(linebreak())
}))

#v(24pt)

// --- Lieu et date ---
#joined((data.identity.ville, joined((L.date_prefix, data.date), " ")), ", ")

#v(8pt)

// --- Objet ---
#if data.subject != "" {
  text(weight: "bold", L.subject_prefix + " " + data.subject + " - " + L.subject_suffix)
}

#v(8pt)

// --- Corps ---
#L.salutation

#L.intro #data.paragraphs.at(0, default: "")

#data.paragraphs.at(1, default: "")

#data.paragraphs.at(2, default: "") #L.international

#L.closing_paragraph

#L.closing

#v(16pt)

#name

