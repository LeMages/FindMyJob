// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 1.2cm, bottom: 1.0cm, left: 1.59cm, right: 1.59cm),
)

#set text(font: "Arimo", size: 9.5pt, fill: rgb("2B2B2B"), lang: "fr")
// Note: Liberation Sans is metrically identical to Arial.
// Replace with "Arial" if Arial is available on your system.

// Espacement mesuré sur les vrais contours des glyphes (évite les chevauchements entre blocs)
#set text(top-edge: "ascender", bottom-edge: "descender")
#set par(leading: 0.4em)

// === Spacing ===
#let space-section = 14pt   // avant chaque titre de section
#let space-entry = 8pt      // avant chaque entrée (poste, projet, diplôme)
#let space-line = 4pt       // entre deux lignes de compétences / puces

// === Colors ===
#let accent = rgb("2E5A88")
#let dark = rgb("2B2B2B")
#let gray = rgb("555555")
#let light-gray = rgb("AAAAAA")

// === Helper functions ===

#let section-header(title) = {
  v(space-section)
  block(
    width: 100%,
    below: space-entry,
    {
      text(size: 10pt, weight: "bold", fill: accent, upper(title))
      v(-5pt)
      line(length: 100%, stroke: 0.5pt + accent)
    }
  )
}

#let entry-header(title, org, date) = {
  v(space-entry, weak: true)
  block(below: 3pt, {
    text(size: 9.5pt, weight: "bold", fill: dark, title)
    text(size: 9pt, fill: gray)[  |  #org  |  ]
    text(size: 9pt, fill: gray, style: "italic", date)
  })
}

#let skill-line(label, value) = {
  block(below: space-line, {
    text(weight: "bold", fill: dark, label + " : ")
    text(fill: dark, value)
  })
}

// Puce en liste native : ordre de lecture du texte garanti pour les ATS
#let bullet-item(content) = {
  block(below: space-line, {
    list(
      marker: text(fill: dark, "•"),
      indent: 2pt,
      body-indent: 6pt,
      text(fill: dark, content),
    )
  })
}

#let sep = text(size: 9pt, fill: light-gray, "  •  ")

// === Document ===

// --- Header ---
#align(center, {
  text(size: 18pt, weight: "bold", fill: dark, "ANTOINE DUPONT")
  v(2pt)
  text(size: 10.5pt, fill: accent, "Étudiant Ingénieur Informatique, spécialité Data & Cloud")
  v(2pt)
  text(size: 9.5pt, weight: "bold", fill: dark, "Stage de fin d'études de 6 mois, disponible dès février 2027")
  v(2pt)
  {
    text(size: 9pt, fill: gray, "Lyon, 69003")
    sep
    text(size: 9pt, fill: gray, "antoine.dupont@exemple.fr")
    sep
    text(size: 9pt, fill: gray, "+33 6 00 00 00 00")
    sep
    text(size: 9pt, fill: gray, "Permis de conduire B")
  }
  v(1pt)
  {
    text(size: 9pt, fill: gray, "linkedin.com/in/antoine-dupont-exemple")
    sep
    text(size: 9pt, fill: gray, "github.com/adupont-exemple")
    sep
    text(size: 9pt, fill: gray, "antoine-dupont.exemple.fr")
  }
})

// --- Profil ---
#section-header("Profil")
#block({
  set par(leading: 0.45em, justify: false)
  text(size: 9.5pt, fill: gray)[Étudiant ingénieur en dernière année à l'École Supérieure du Numérique de Lyon (niveau Master), spécialisé en Data et Cloud. 6 mois d'expérience en entreprise sur des pipelines de données logistiques et des tableaux de bord de pilotage, de l'ingestion jusqu'à la restitution.]
})

// --- Compétences Techniques ---
#section-header("Compétences Techniques")
#skill-line("Langages", "Python, SQL, Java, TypeScript (React, Node.js)")
#skill-line("Data & ML", "Pandas, NumPy, Scikit-learn, XGBoost, Prophet, Hugging Face, OpenCV, Power BI")
#skill-line("Data Engineering", "ETL, Airflow, dbt, PostgreSQL, MongoDB, Snowflake, Spark, Kafka")
#skill-line("Cloud & DevOps", "Azure (Data Factory, Blob Storage), Docker, Git, GitLab CI, Terraform, Linux")
#skill-line("Développement", "FastAPI, Spring Boot, API REST")

// --- Expérience Professionnelle ---
#section-header("Expérience Professionnelle")
#entry-header("Stage Data Engineer", "Transvia Logistique, Lyon", "09.2025 – 02.2026")
#bullet-item[Refonte du pipeline d'ingestion de 300+ entrepôts : traitement 40% plus rapide, < 1% d'erreurs (Airflow, dbt)]
#bullet-item[Création de tableaux de bord de suivi des livraisons : adoptés par 5 équipes opérationnelles]
#bullet-item[Développement d'API REST (FastAPI) exposant les indicateurs aux outils internes de planification]

// --- Projets ---
#section-header("Projets")

#entry-header("Prévision de fréquentation – Vélos en libre-service", "École Supérieure du Numérique", "2025 – 2026")
#bullet-item[Modélisation de la demande horaire sur 400 stations à partir de données ouvertes et météo, livrée à une collectivité]
#bullet-item[Mise en place du pipeline de collecte et d'entraînement des modèles (Python, XGBoost, PostgreSQL, Airflow)]

#entry-header("DocFinder – Recherche sémantique dans des notices", "Projet personnel open source", "2026")
#bullet-item[Conception d'une application web (TypeScript, FastAPI) : recherche en langage naturel dans des notices techniques]
#bullet-item[Déploiement conteneurisé sur Azure avec Terraform, tests automatisés et CI/CD GitLab]

#entry-header("QualiVision – Détection de défauts par vision", "École Supérieure du Numérique", "2024 – 2025")
#bullet-item[Entraînement d'un modèle de classification OpenCV (92% de précision sur 2 500 images) et interface de contrôle]

// --- Formation ---
#section-header("Formation")

#entry-header("Diplôme d'ingénieur (niveau Master, CTI) – Data & Cloud", "École Supérieure du Numérique", "2022 – 2027")
#bullet-item[Semestre d'échange – Université de Lisbonne, Portugal (09.2024 – 01.2025)]

// --- Langues & Centres d'intérêt ---
#section-header("Langues & Centres d'Intérêt")
#block(below: space-line, {
  text(weight: "bold", fill: dark, "Français : ")
  text(fill: dark, "Langue maternelle")
  text(fill: light-gray, "  |  ")
  text(weight: "bold", fill: dark, "Anglais : ")
  text(fill: dark, "C1, TOEIC 905/990 (semestre d'échange au Portugal)")
})
#block(below: space-line, {
  text(weight: "bold", fill: dark, "Intérêts : ")
  text(fill: dark, "Course à pied, Photographie, Contributions à des projets open source en data")
})

#v(8pt)
#text(size: 9pt, fill: gray, style: "italic", "Références disponibles sur demande")
