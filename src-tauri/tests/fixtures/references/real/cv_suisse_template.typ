#let data = json("data.json")
// === Page setup ===
#set page(
  paper: "a4",
  margin: (top: 1.2cm, bottom: 1.0cm, left: 1.59cm, right: 1.59cm),
)

#set text(font: "Arimo", size: 9.5pt, fill: rgb("2B2B2B"), lang: data.lang)
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

// Valeurs non vides jointes par un séparateur (aucun séparateur orphelin).
#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

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

// Ligne de coordonnées : valeurs non vides séparées par `sep`.
#let contact-line(values) = values.filter(v => v != "").map(v => text(size: 9pt, fill: gray, v)).join(sep)

// === Document ===

// --- Header ---
#align(center, {
  text(size: 18pt, weight: "bold", fill: dark, upper(joined((data.identity.prenom, data.identity.nom), " ")))
  if data.identity.title != "" {
    v(2pt)
    text(size: 10.5pt, fill: accent, data.identity.title)
  }
  v(2pt)
  text(size: 9.5pt, weight: "bold", fill: dark, "Stage de fin d'études de 6 mois, disponible dès mars 2027")
  let permis = if data.identity.permis { data.labels.permis } else { "" }
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

// --- Profil ---
#if data.labels.section_profile != "" and data.summary != "" {
  section-header(data.labels.section_profile)
  block({
    set par(leading: 0.45em, justify: false)
    text(size: 9.5pt, fill: gray, data.summary)
  })
}

// --- Compétences Techniques ---
#if data.labels.section_skills != "" and data.skills_groups.len() > 0 {
  section-header(data.labels.section_skills)
  for g in data.skills_groups {
    skill-line(g.category, g.skills.join(", "))
  }
}

// --- Expérience Professionnelle ---
#if data.labels.section_experiences != "" and data.experiences.len() > 0 {
  section-header(data.labels.section_experiences)
  for e in data.experiences {
    entry-header(e.poste, joined((e.entreprise, e.lieu), ", "), e.period)
    for b in e.bullets { bullet-item(b) }
  }
}

// --- Projets ---
#if data.labels.section_projects != "" and data.projects.len() > 0 {
  section-header(data.labels.section_projects)
  for p in data.projects {
    entry-header(p.titre, p.pour_qui, p.period)
    for b in p.bullets { bullet-item(b) }
  }
}

// --- Formation ---
#if data.labels.section_formations != "" and data.formations.len() > 0 {
  section-header(data.labels.section_formations)
  for f in data.formations {
    entry-header(f.titre, f.ecole, f.period)
    for d in f.details { bullet-item(d) }
  }
}

// --- Langues & Centres d'intérêt ---
#if data.labels.section_langues != "" and (data.langues.len() > 0 or data.interets.len() > 0) {
  section-header(data.labels.section_langues)
  if data.langues.len() > 0 {
    block(below: space-line, data.langues.map(l => {
      text(weight: "bold", fill: dark, if l.level != "" { l.nom + " : " } else { l.nom })
      text(fill: dark, l.level)
    }).join(text(fill: light-gray, "  |  ")))
  }
  if data.interets.len() > 0 {
    block(below: space-line, {
      if data.labels.interets_label != "" { text(weight: "bold", fill: dark, data.labels.interets_label + " ") }
      text(fill: dark, data.interets.join(", "))
    })
  }
}

#v(8pt)
#text(size: 9pt, fill: gray, style: "italic", "Références disponibles sur demande")
