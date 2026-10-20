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

// [CONSIGNES]
// - Le CV tient sur une page.
// - Le titre et le résumé reprennent le vocabulaire de l'offre, sans rien inventer.

// --- Header ---
#align(center, {
  text(size: 18pt, weight: "bold", fill: dark, "CAMILLE EXEMPLE")
  v(2pt)
  // [VARIABLE] Intitulé proche du poste visé, dans le vocabulaire de l'offre.
  text(size: 10.5pt, fill: accent, "Étudiant Ingénieur Data Engineering & Machine Learning")
  v(2pt)
  text(size: 9.5pt, weight: "bold", fill: dark, "Stage de fin d'études de 6 mois, disponible dès mars 2027")
  v(2pt)
  {
    text(size: 9pt, fill: gray, "Thyez, 74300")
    sep
    text(size: 9pt, fill: gray, "camille@exemple.invalid")
    sep
    text(size: 9pt, fill: gray, "+33 6 00 00 00 00")
    sep
    text(size: 9pt, fill: gray, "Permis de conduire B")
  }
  v(1pt)
  {
    text(size: 9pt, fill: gray, "linkedin.com/in/camille-exemple")
    sep
    text(size: 9pt, fill: gray, "github.com/camille-exemple")
    sep
    text(size: 9pt, fill: gray, "camille-exemple.invalid")
  }
})

// --- Profil ---
#section-header("Profil")
#block({
  set par(leading: 0.45em, justify: false)
  // [FIXE] Formation et spécialité.
  text(size: 9.5pt, fill: gray)[Étudiant ingénieur en dernière année à l'Efrei Paris (niveau Master), spécialisé en Data Engineering et Machine Learning.
  // [ADAPTABLE] Expérience la plus pertinente pour l'offre, chiffres du profil.
  6 mois d'expérience en entreprise sur le forecasting de séries temporelles à grande échelle et des systèmes RAG en production, de la donnée brute jusqu'à l'API.]
})

// --- Compétences Techniques ---
#section-header("Compétences Techniques")
#skill-line("Langages", "Python, SQL, Rust, JavaScript (Vue.js, Node.js)")
#skill-line("Data & ML", "Pandas, NumPy, Polars, Scikit-learn, PyTorch, Darts, LangChain, RAG, LLM, OpenCV, YOLO")
#skill-line("Data Engineering", "ETL, PostgreSQL, pgvector, MySQL, SQLite, Spark, Kafka, SAP Business Objects")
#skill-line("Cloud & DevOps", "AWS (EC2, S3), Docker, Git, GitHub Actions, CI/CD, Linux/Bash")
#skill-line("Développement", "Flask, Axum, Tauri, API REST")

// --- Expérience Professionnelle ---
#section-header("Expérience Professionnelle")
#entry-header("Stage Ingénieur Data & IA", "Need4Viz, Paris", "11.2025 – 04.2026")
#bullet-item[Déploiement d'un pipeline de forecasting sur 2 000+ séries temporelles SAP : R² de 0.70, Directional Accuracy de 80% (Transformer, Darts)]
#bullet-item[Intégration d'un RAG dans l'outil de dataviz : 80% de réponses correctes sur le corpus client]
#bullet-item[Conception des API REST et de l'interface web (Flask) intégrant les modèles en production]

// --- Projets ---
#section-header("Projets")

#entry-header("RAG Juridique – Question-réponse sur textes de loi", "Efrei Paris", "2025 – 2026")
#bullet-item[Mise en place d'un RAG interrogeant le CESEDA en langage naturel, livré à un client externe]
#bullet-item[Construction du pipeline d'ingestion et de vectorisation des textes (Python, LangChain, PostgreSQL/pgvector)]

#entry-header("FindMyJob – Application de suivi de candidatures", "Projet personnel open source", "2026")
#bullet-item[Création d'une app desktop (Rust, Tauri, SQLite) : extraction d'offres par LLM, génération de CV et lettres]
#bullet-item[Migration de l'architecture Axum/PostgreSQL/Docker vers une app autonome, avec CI/CD GitHub Actions]

#entry-header("Wild Dump Prevention – Détection par Computer Vision", "Efrei Paris", "2024 – 2025")
#bullet-item[Entraînement d'un modèle YOLO/OpenCV (90% de précision sur 3 000 images) et dashboard web Flask]

// --- Formation ---
#section-header("Formation")

#entry-header("Diplôme d'ingénieur (niveau Master, CTI) – Data Engineering", "Efrei Paris", "2022 – 2027")
#bullet-item[Échange universitaire – Université Concordia, Montréal, Canada (08.2024 – 12.2024)]

// --- Langues & Centres d'intérêt ---
#section-header("Langues & Centres d'Intérêt")
#block(below: space-line, {
  text(weight: "bold", fill: dark, "Français : ")
  text(fill: dark, "Langue maternelle")
  text(fill: light-gray, "  |  ")
  text(weight: "bold", fill: dark, "Anglais : ")
  text(fill: dark, "B2, TOEIC 840/990 (échange universitaire au Canada)")
})
#block(below: space-line, {
  text(weight: "bold", fill: dark, "Intérêts : ")
  text(fill: dark, "Handball, Formule 1, Projets personnels en Machine Learning et Intelligence Artificielle")
})

#v(8pt)
#text(size: 9pt, fill: gray, style: "italic", "Références disponibles sur demande")
