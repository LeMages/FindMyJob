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

// === Document ===

// --- Expéditeur (gauche) ---
#block({
  set par(justify: false, leading: 0.35em)
  text(size: 14pt, weight: "bold", fill: dark, data.identity.prenom + " " + data.identity.nom)
  linebreak()
  if data.identity.title != "" {
    text(size: 10pt, fill: accent, data.identity.title)
    linebreak()
  }
  let locParts = (data.identity.ville, data.identity.zip_code).filter(v => v != "")
  if locParts.len() > 0 {
    text(size: 10pt, fill: gray, locParts.join(", "))
    linebreak()
  }
  if data.identity.telephone != "" {
    text(size: 10pt, fill: gray, data.identity.telephone)
    linebreak()
  }
  if data.identity.email != "" {
    text(size: 10pt, fill: gray, data.identity.email)
  }
})

#v(10pt)

// --- Destinataire (droite, fenêtre d'enveloppe suisse) ---
#align(right, block(width: 7cm, {
  set align(left)
  set par(justify: false, leading: 0.35em)
  if data.recipient.company != "" {
    text(weight: "bold", data.recipient.company)
    linebreak()
  }
  for line in data.recipient.extra_lines {
    if line != "" {
      line
      linebreak()
    }
  }
  if data.recipient.location != "" {
    data.recipient.location
  }
}))

#v(24pt)

// --- Lieu et date ---
#{
  let locParts = (data.identity.ville).filter(v => v != "")
  let loc = if locParts.len() > 0 { locParts.join(", ") + ", " } else { "" }
  let prefix = if data.labels.date_prefix != "" { data.labels.date_prefix + " " } else { "" }
  loc + prefix + data.date
}

#v(8pt)

// --- Objet ---
#text(weight: "bold")[#{if data.labels.extra.at(0, default: "") != "" { data.labels.extra.at(0, default: "") + " " } else { "" }}#data.subject]

#v(8pt)

// --- Corps ---
#data.labels.salutation

#for p in data.paragraphs {
  p
  parbreak()
}

#data.labels.closing

#v(16pt)

#data.identity.prenom #data.identity.nom