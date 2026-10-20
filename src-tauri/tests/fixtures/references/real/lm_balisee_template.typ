#let data = json("data.json")
// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 2cm, bottom: 2cm, left: 2.2cm, right: 2.2cm),
)

#set text(font: "Arimo", size: 10.5pt, fill: rgb("2B2B2B"), lang: data.lang)
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

// Valeurs non vides jointes par un séparateur (aucun séparateur orphelin).
#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

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
  let lines = ()
  let name = joined((data.identity.prenom, data.identity.nom), " ")
  if name != "" { lines.push(text(size: 14pt, weight: "bold", fill: dark, name)) }
  if data.identity.title != "" { lines.push(text(size: 10pt, fill: accent, data.identity.title)) }
  let place = joined((data.identity.ville, data.identity.zip_code), ", ")
  for value in (place, data.identity.telephone, data.identity.email).filter(v => v != "") {
    lines.push(text(size: 10pt, fill: gray, value))
  }
  lines.join(linebreak())
})

#v(10pt)

// --- Destinataire (droite, fenêtre d'enveloppe suisse) ---
// [VARIABLE] Nom de l'entreprise et adresse tirés de l'offre. Si le service
// ou l'adresse sont inconnus, garder seulement le nom de l'entreprise et la ville.
#align(right, block(width: 7cm, {
  set align(left)
  set par(justify: false, leading: 0.35em)
  let lines = ()
  if data.recipient.company != "" { lines.push(text(weight: "bold", data.recipient.company)) }
  for line in data.recipient.extra_lines.filter(v => v != "") { lines.push(line) }
  if data.recipient.location != "" { lines.push(data.recipient.location) }
  lines.join(linebreak())
}))

#v(24pt)

// --- Lieu et date --- [VARIABLE] Date du jour de la génération, lieu inchangé.
#if data.date != "" {
  joined((data.identity.ville, joined((data.labels.date_prefix, data.date), " ")), ", ")
}

#v(8pt)

// --- Objet ---
// [VARIABLE] Intitulé exact du poste tiré de l'offre. Garder la fin
// « Stage de fin d'études de 6 mois dès mars 2027 ».
#if data.subject != "" {
  text(weight: "bold", joined((data.labels.subject_prefix, data.subject), " "))
}

#v(8pt)

// --- Corps ---
// [FIXE]
#if data.labels.salutation != "" {
  data.labels.salutation
  parbreak()
}

#for p in data.paragraphs {
  p
  parbreak()
}

// [FIXE]
#if data.labels.closing != "" {
  data.labels.closing
  parbreak()
}

#v(16pt)

#joined((data.identity.prenom, data.identity.nom), " ")

#v(12pt)
#text(size: 9pt, fill: gray)[#data.texts.t1]
