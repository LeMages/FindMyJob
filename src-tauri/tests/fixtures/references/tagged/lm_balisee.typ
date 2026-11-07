// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 2cm, bottom: 2cm, left: 2.2cm, right: 2.2cm),
)

#set text(font: "Arimo", size: 10.5pt, fill: rgb("2B2B2B"), lang: "fr")
// Note: Liberation Sans is metrically identical to Arial.
#set par(leading: 0.55em, justify: true)
#set text(hyphenate: false)

// === Colors ===
#let accent = rgb("2E5A88")
#let dark = rgb("2B2B2B")
#let gray = rgb("555555")

// === Spacing ===
#let space-para = 12pt   // entre deux paragraphes

#set par(spacing: space-para)

// === Document ===

// [CONSIGNES]
// - Toutes les valeurs « Exemple » (Exemple SA, Rue de l'Exemple…) sont fictives
//   et doivent être remplacées.
// - Les faits (expériences, chiffres, technologies, dates) viennent UNIQUEMENT
//   du profil. Ne jamais inventer un fait, un chiffre ou une compétence.
// - La lettre doit tenir sur une page.
// - Le contact recruteur n'est jamais nommé dans la lettre.

// --- Expéditeur (gauche) ---
#block({
  set par(justify: false, leading: 0.35em)
  text(size: 14pt, weight: "bold", fill: dark, "Antoine Dupont")
  linebreak()
  text(size: 10pt, fill: accent, "Étudiant Ingénieur Informatique, spécialité Data & Cloud")
  linebreak()
  text(size: 10pt, fill: gray, "Lyon, 69003")
  linebreak()
  text(size: 10pt, fill: gray, "+33 6 00 00 00 00")
  linebreak()
  text(size: 10pt, fill: gray, "antoine.dupont@exemple.fr")
})

#v(10pt)

// --- Destinataire (droite, fenêtre d'enveloppe suisse) ---
#align(right, block(width: 7cm, {
  set align(left)
  set par(justify: false, leading: 0.35em)
  text(weight: "bold", "Exemple SA")
  linebreak()
  [Service des ressources humaines]
  linebreak()
  [Rue de l'Exemple 1]
  linebreak()
  [1201 Genève]
}))

#v(24pt)

// --- Lieu et date ---
Lyon, le 13 octobre 2026

#v(8pt)

// --- Objet ---
// [VARIABLE] Intitulé exact du poste tiré de l'offre. Garder la fin
// « Stage de fin d'études de 6 mois dès mars 2027 ».
#text(weight: "bold")[Objet : Candidature pour le poste de Stagiaire Data Engineer – Stage de fin d'études de 6 mois dès mars 2027]

#v(8pt)

// --- Corps ---
// [FIXE]
Madame, Monsieur,

// [FIXE] Les deux premières phrases.
Actuellement étudiant en dernière année du cycle ingénieur à l'École Supérieure du Numérique de Lyon, spécialisé en Data et Cloud, je suis à la recherche d'un stage de fin d'études de 6 mois à partir de mars 2027. Ce stage, obligatoire pour l'obtention de mon diplôme, peut se dérouler en France ou à l'étranger et fera l'objet d'une convention établie par mon école. 
// [VARIABLE] Intitulé exact du poste et nom de l'entreprise, tirés de l'offre.
C'est dans ce cadre que je vous adresse ma candidature pour le poste de Stagiaire Data Engineer chez Exemple SA.

// [ADAPTABLE] Choisir et ordonner, parmi les expériences et projets du profil,
// les réalisations les plus pertinentes pour l'offre (ex. : mettre les tableaux de bord
// en avant pour une offre orientée BI, le pipeline de données pour une offre Data Engineering).
// Garder les chiffres exacts du profil. Le projet personnel n'est jamais présenté
// comme un projet entrepreneurial et n'est pas nommé.
Lors de mon stage de 6 mois chez Transvia Logistique, j'ai refondu le pipeline d'ingestion de plus de 300 entrepôts avec Airflow et dbt, ce qui a réduit le temps de traitement de 40% tout en maintenant un taux d'erreur inférieur à 1%. J'y ai également conçu des tableaux de bord de suivi des livraisons adoptés par cinq équipes opérationnelles, ainsi que les API REST exposant ces indicateurs aux outils internes. En parallèle, j'ai développé un projet personnel de recherche sémantique dans des notices techniques, déployé sur Azure avec Terraform et une chaîne d'intégration continue complète.

// [VARIABLE] Enjeu de l'entreprise et 2 missions reprises de l'offre.
// Garder la structure : « Intéressé par… je souhaite rejoindre… afin de…
// Les missions proposées, de … à …, correspondent pleinement… ».
Intéressé par la fiabilité et la valorisation des données, je souhaite rejoindre l'équipe Data d'Exemple SA afin de contribuer à la construction de sa plateforme de données. Les missions proposées, de l'industrialisation des pipelines d'ingestion à la mise en production de modèles prédictifs, correspondent pleinement à mes compétences et à mes aspirations professionnelles.
// [FIXE]
Mon semestre d'échange à l'Université de Lisbonne et mon niveau d'anglais (TOEIC 905) me permettent par ailleurs d'évoluer sans difficulté dans un environnement international.

// [FIXE] Mobilité, disponibilité et demande d'entretien.
Mobile et prêt à m'installer à proximité de votre site, je serai pleinement disponible dès mars 2027 pour toute la durée du stage. Autonome, rigoureux et curieux, je serais heureux de mettre mes compétences au service de vos projets et d'en discuter avec vous lors d'un entretien, en présentiel ou à distance.

// [FIXE]
Avec mes meilleures salutations,

#v(16pt)

Antoine Dupont

#v(12pt)
#text(size: 9pt, fill: gray)[P.J. : Curriculum vitae]
