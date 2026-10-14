// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 2cm, bottom: 2cm, left: 2.2cm, right: 2.2cm),
)

#set text(font: "Arimo", size: 10.5pt, fill: rgb("2B2B2B"), lang: "fr")
// Note: Liberation Sans is metrically identical to Arial.
// Espacement mesuré sur les vrais contours des glyphes
#set text(top-edge: "ascender", bottom-edge: "descender")
#set par(leading: 0.55em, justify: true)
#set text(hyphenate: false)

// === Colors ===
#let accent = rgb("2E5A88")
#let dark = rgb("2B2B2B")
#let gray = rgb("555555")

// === Spacing ===
#let space-para = 12pt   // entre deux paragraphes

#show par: set block(spacing: space-para)

// === Document ===

// ============================================================
// CONSIGNES POUR LA GÉNÉRATION (lues par le LLM, invisibles dans le PDF)
// - Chaque zone de texte est précédée d'une balise :
//     [FIXE]       : recopier mot pour mot, ne rien changer.
//     [VARIABLE]   : remplacer par les informations tirées de l'offre.
//     [ADAPTABLE]  : réorganiser à partir du profil selon l'offre, sans rien inventer.
// - Toutes les valeurs « Exemple » (Exemple SA, Rue de l'Exemple…) sont fictives
//   et doivent être remplacées.
// - Les faits (expériences, chiffres, technologies, dates) viennent UNIQUEMENT
//   du profil. Ne jamais inventer un fait, un chiffre ou une compétence.
// - Garder la même longueur, le même nombre de paragraphes et la même structure.
//   La lettre doit tenir sur une page.
// - Le contact recruteur n'est jamais nommé dans la lettre ; la formule
//   « Madame, Monsieur, » reste inchangée.
// - Ne pas recopier ces commentaires dans la lettre générée.
// ============================================================

// --- Expéditeur (gauche) --- [FIXE]
#block({
  set par(justify: false, leading: 0.35em)
  text(size: 14pt, weight: "bold", fill: dark, "Camille Exemple")
  linebreak()
  text(size: 10pt, fill: accent, "Étudiant Ingénieur Data Engineering & Machine Learning")
  linebreak()
  text(size: 10pt, fill: gray, "Thyez, 74300")
  linebreak()
  text(size: 10pt, fill: gray, "+33 6 00 00 00 00")
  linebreak()
  text(size: 10pt, fill: gray, "camille@exemple.invalid")
})

#v(10pt)

// --- Destinataire (droite, fenêtre d'enveloppe suisse) ---
// [VARIABLE] Nom de l'entreprise et adresse tirés de l'offre. Si le service
// ou l'adresse sont inconnus, garder seulement le nom de l'entreprise et la ville.
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

// --- Lieu et date --- [VARIABLE] Date du jour de la génération, lieu inchangé.
Thyez, le 13 octobre 2026

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
Actuellement étudiant en dernière année du cycle ingénieur à l'Efrei Paris, spécialisé en Data Engineering et Machine Learning, je suis à la recherche d'un stage de fin d'études de 6 mois à partir de mars 2027. Ce stage, obligatoire pour l'obtention de mon diplôme, doit se dérouler à l'étranger et fera l'objet d'une convention établie par mon école. 
// [VARIABLE] Intitulé exact du poste et nom de l'entreprise, tirés de l'offre.
C'est dans ce cadre que je vous adresse ma candidature pour le poste de Stagiaire Data Engineer chez Exemple SA.

// [ADAPTABLE] Choisir et ordonner, parmi les expériences et projets du profil,
// les réalisations les plus pertinentes pour l'offre (ex. : mettre le RAG en avant
// pour une offre orientée IA, le pipeline de données pour une offre Data Engineering).
// Garder les chiffres exacts du profil. Le projet personnel de forecasting n'est
// jamais présenté comme un projet entrepreneurial et n'est pas nommé.
Lors de mon stage de 6 mois chez Need4Viz, j'ai conçu et déployé un pipeline de forecasting sur plus de 2 000 séries temporelles issues d'univers SAP, avec des modèles Transformer atteignant une Directional Accuracy de 80%. J'y ai également développé un système RAG intégré à l'outil de visualisation de l'entreprise, ainsi que les API REST permettant de mettre ces modèles en production. En parallèle, je poursuis ce travail sur le forecasting dans un projet personnel, pour lequel j'ai conçu des prévisions par intervalles sur des données d'entreprise et l'architecture qui les accompagne (Rust, PostgreSQL, Docker).

// [VARIABLE] Enjeu de l'entreprise et 2 missions reprises de l'offre.
// Garder la structure : « Intéressé par… je souhaite rejoindre… afin de…
// Les missions proposées, de … à …, correspondent pleinement… ».
Intéressé par la fiabilité et la valorisation des données, je souhaite rejoindre l'équipe Data d'Exemple SA afin de contribuer à la construction de sa plateforme de données. Les missions proposées, de l'industrialisation des pipelines d'ingestion à la mise en production de modèles prédictifs, correspondent pleinement à mes compétences et à mes aspirations professionnelles.
// [FIXE]
Mon semestre d'échange à l'Université Concordia de Montréal et mon niveau d'anglais (TOEIC 840) me permettent par ailleurs d'évoluer sans difficulté dans un environnement international.

// [FIXE] Statut de frontalier, disponibilité et demande d'entretien.
Résidant à Thyez, à une quarantaine de minutes de Genève, je peux travailler en tant que frontalier et serai pleinement disponible dès mars 2027. Autonome, rigoureux et curieux, je serais heureux de mettre mes compétences au service de vos projets et d'en discuter avec vous lors d'un entretien, en présentiel ou à distance.

// [FIXE]
Avec mes meilleures salutations,

#v(16pt)

Camille Exemple
