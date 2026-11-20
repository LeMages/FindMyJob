// Référence de CV balisée (Antoine Dupont, fictif), au contrat FindMyJob :
// valeurs d'exemple, données lues dans sys.inputs.data, textes fixes FR/EN,
// consignes et emplacements dans la métadonnée.

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
    linkedin: "linkedin.com/in/antoine-dupont-exemple",
    github: "github.com/adupont-exemple",
    portfolio: "antoine-dupont.exemple.fr",
    permis: true,
  ),
  summary: "Étudiant ingénieur en dernière année à l'École Supérieure du Numérique de Lyon (niveau Master), spécialisé en Data et Cloud. 6 mois d'expérience en entreprise sur des pipelines de données logistiques et des tableaux de bord de pilotage, de l'ingestion jusqu'à la restitution.",
  skills_groups: (
    (category: "Langages", skills: ("Python", "SQL", "Java", "TypeScript (React, Node.js)")),
    (category: "Data & ML", skills: ("Pandas", "NumPy", "Scikit-learn", "XGBoost", "Prophet", "Hugging Face", "OpenCV", "Power BI")),
    (category: "Data Engineering", skills: ("ETL", "Airflow", "dbt", "PostgreSQL", "MongoDB", "Snowflake", "Spark", "Kafka")),
    (category: "Cloud & DevOps", skills: ("Azure (Data Factory, Blob Storage)", "Docker", "Git", "GitLab CI", "Terraform", "Linux")),
    (category: "Développement", skills: ("FastAPI", "Spring Boot", "API REST")),
  ),
  experiences: (
    (
      poste: "Stage Data Engineer",
      entreprise: "Transvia Logistique",
      lieu: "Lyon",
      period: "09.2025 - 02.2026",
      intro: "",
      bullets: (
        "Refonte du pipeline d'ingestion de 300+ entrepôts : traitement 40% plus rapide, < 1% d'erreurs (Airflow, dbt)",
        "Création de tableaux de bord de suivi des livraisons : adoptés par 5 équipes opérationnelles",
        "Développement d'API REST (FastAPI) exposant les indicateurs aux outils internes de planification",
      ),
    ),
  ),
  projects: (
    (
      titre: "Prévision de fréquentation - Vélos en libre-service",
      pour_qui: "École Supérieure du Numérique",
      period: "2025 - 2026",
      stack: "",
      url: "",
      intro: "",
      bullets: (
        "Modélisation de la demande horaire sur 400 stations à partir de données ouvertes et météo, livrée à une collectivité",
        "Mise en place du pipeline de collecte et d'entraînement des modèles (Python, XGBoost, PostgreSQL, Airflow)",
      ),
    ),
    (
      titre: "DocFinder - Recherche sémantique dans des notices",
      pour_qui: "Projet personnel open source",
      period: "2026",
      stack: "",
      url: "",
      intro: "",
      bullets: (
        "Conception d'une application web (TypeScript, FastAPI) : recherche en langage naturel dans des notices techniques",
        "Déploiement conteneurisé sur Azure avec Terraform, tests automatisés et CI/CD GitLab",
      ),
    ),
    (
      titre: "QualiVision - Détection de défauts par vision",
      pour_qui: "École Supérieure du Numérique",
      period: "2024 - 2025",
      stack: "",
      url: "",
      intro: "",
      bullets: (
        "Entraînement d'un modèle de classification OpenCV (92% de précision sur 2 500 images) et interface de contrôle",
      ),
    ),
  ),
  formations: (
    (
      titre: "Diplôme d'ingénieur (niveau Master, CTI) - Data & Cloud",
      ecole: "École Supérieure du Numérique",
      period: "2022 - 2027",
      intro: "",
      details: ("Semestre d'échange - Université de Lisbonne, Portugal (09.2024 - 01.2025)",),
    ),
  ),
  langues: (
    (nom: "Français", level: "Langue maternelle"),
    (nom: "Anglais", level: "C1, TOEIC 905/990 (semestre d'échange au Portugal)"),
  ),
  list_marker: "•",
)

#let data = if "data" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }

#metadata((
  contract: "findmyjob/cv",
  version: 1,
  languages: ("fr", "en"),
  example: example,
  guidance: "Le CV tient sur une page. Le titre et le résumé reprennent le vocabulaire de l'offre, sans rien inventer.",
  slots: (
    (field: "identity.title", mode: "variable", instruction: "Intitulé proche du poste visé, dans le vocabulaire de l'offre."),
    (field: "summary", mode: "adaptable", instruction: "Commence par la formation et la spécialité, puis l'expérience la plus pertinente pour l'offre, avec les chiffres du profil."),
  ),
  period: (
    granularity: (experiences: "month", projects: "year", formations: "year"),
    separator: " - ",
    month_style: "numeric",
    numeric_separator: ".",
  ),
)) <findmyjob>

#let L = (
  fr: (
    availability: "Stage de fin d'études de 6 mois, disponible dès mars 2027",
    permis: "Permis de conduire B",
    profile: "Profil",
    skills: "Compétences Techniques",
    experiences: "Expérience Professionnelle",
    projects: "Projets",
    formations: "Formation",
    langues: "Langues & Centres d'Intérêt",
    interests_label: "Intérêts :",
    interests: "Course à pied, Photographie, Contributions à des logiciels libres en data",
    references: "Références disponibles sur demande",
  ),
  en: (
    availability: "6-month end-of-studies internship, available from March 2027",
    permis: "Driving licence B",
    profile: "Profile",
    skills: "Technical Skills",
    experiences: "Professional Experience",
    projects: "Projects",
    formations: "Education",
    langues: "Languages & Interests",
    interests_label: "Interests:",
    interests: "Running, Photography, Open source data contributions",
    references: "References available on request",
  ),
).at(data.lang)

// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 1.2cm, bottom: 1.0cm, left: 1.59cm, right: 1.59cm),
)

#set text(font: "Arimo", size: 9.5pt, fill: rgb("2B2B2B"), lang: data.lang)
#set text(top-edge: "ascender", bottom-edge: "descender")
#set par(leading: 0.4em)

// === Spacing ===
#let space-section = 14pt
#let space-entry = 8pt
#let space-line = 4pt

// === Colors ===
#let accent = rgb("2E5A88")
#let dark = rgb("2B2B2B")
#let gray = rgb("555555")
#let light-gray = rgb("AAAAAA")

// === Helper functions ===

// Valeurs non vides jointes par un séparateur (aucun séparateur orphelin).
#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

// Titre de section : un heading, mis en forme par cette règle.
// L'espacement autour du heading est fixé ici (et non hérité de ses
// voisins) : la mise en page reste celle d'un en-tête bloc + filet.
#show heading: set block(above: 12pt, below: 0pt)
#show heading: title => {
  v(space-section)
  block(
    width: 100%,
    above: 0pt,
    below: space-entry,
    {
      text(size: 10pt, weight: "bold", fill: accent, upper(title.body))
      v(-5pt)
      line(length: 100%, stroke: 0.5pt + accent)
    }
  )
}

// Organisation et date facultatives : chacune n'apparaît, avec sa barre,
// que si elle est renseignée.
#let entry-header(title, org, date) = {
  v(space-entry, weak: true)
  block(below: 3pt, {
    text(size: 9.5pt, weight: "bold", fill: dark, title)
    if org != "" { text(size: 9pt, fill: gray)[  |  #org] }
    if date != "" {
      text(size: 9pt, fill: gray)[  |  ]
      text(size: 9pt, fill: gray, style: "italic", date)
    }
  })
}

#let skill-line(label, value) = {
  block(below: space-line, {
    text(weight: "bold", fill: dark, label + " : ")
    text(fill: dark, value)
  })
}

// Puce en liste native ; le symbole vient de data.list_marker.
#let bullet-item(content) = {
  block(below: space-line, {
    list(
      marker: text(fill: dark, data.list_marker),
      indent: 2pt,
      body-indent: 6pt,
      text(fill: dark, content),
    )
  })
}

// Séparateur des coordonnées : pas un symbole de liste, reste en dur.
#let sep = text(size: 9pt, fill: light-gray, "  •  ")

#let intro-block(content) = {
  if content != "" { block(below: space-line, text(fill: dark, content)) }
}

#let contact-line(values) = values.filter(v => v != "").map(v => text(size: 9pt, fill: gray, v)).join(sep)

// === Document ===

#align(center, {
  text(size: 18pt, weight: "bold", fill: dark, upper(joined((data.identity.prenom, data.identity.nom), " ")))
  if data.identity.title != "" {
    v(2pt)
    text(size: 10.5pt, fill: accent, data.identity.title)
  }
  v(2pt)
  text(size: 9.5pt, weight: "bold", fill: dark, L.availability)
  let permis = if data.identity.permis { L.permis } else { "" }
  let first = (joined((data.identity.ville, data.identity.zip_code), ", "), data.identity.email, data.identity.telephone, permis)
  if first.filter(v => v != "").len() > 0 {
    v(2pt)
    contact-line(first)
  }
  let second = (data.identity.linkedin, data.identity.github, data.identity.portfolio)
  if second.filter(v => v != "").len() > 0 {
    v(1pt)
    contact-line(second)
  }
})

#v(1.2em)
#if data.summary != "" {
  heading(L.profile)
  block(below: 1.2em, {
    set par(leading: 0.45em, justify: false)
    text(size: 9.5pt, fill: gray, data.summary)
  })
}

#if data.skills_groups.len() > 0 {
  heading(L.skills)
  for g in data.skills_groups {
    skill-line(g.category, g.skills.join(", "))
  }
}

#if data.experiences.len() > 0 {
  heading(L.experiences)
  for e in data.experiences {
    entry-header(e.poste, joined((e.entreprise, e.lieu), ", "), e.period)
    intro-block(e.intro)
    for b in e.bullets { bullet-item(b) }
  }
}

#if data.projects.len() > 0 {
  heading(L.projects)
  for p in data.projects {
    entry-header(p.titre, p.pour_qui, p.period)
    intro-block(p.intro)
    for b in p.bullets { bullet-item(b) }
  }
}

#if data.formations.len() > 0 {
  heading(L.formations)
  for f in data.formations {
    entry-header(f.titre, f.ecole, f.period)
    intro-block(f.intro)
    for d in f.details { bullet-item(d) }
  }
}

// La ligne des intérêts est un texte fixe : la section reste affichée sans langue.
#heading(L.langues)
#if data.langues.len() > 0 {
  block(below: space-line, data.langues.map(l => {
    text(weight: "bold", fill: dark, if l.level != "" { l.nom + " : " } else { l.nom })
    text(fill: dark, l.level)
  }).join(text(fill: light-gray, "  |  ")))
}
#block(below: space-line, {
  text(weight: "bold", fill: dark, L.interests_label + " ")
  text(fill: dark, L.interests)
})

#v(8pt)
#text(size: 9pt, fill: gray, style: "italic", L.references)
