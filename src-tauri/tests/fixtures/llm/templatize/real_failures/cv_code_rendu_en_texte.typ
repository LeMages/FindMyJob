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
  text(size: 18pt, weight: "bold", fill: dark, upper(data.identity.prenom + " " + data.identity.nom))
  v(2pt)
  if data.identity.title != "" {
    text(size: 10.5pt, fill: accent, data.identity.title)
    v(2pt)
  }
  if data.labels.extra.len() > 0 and data.labels.extra.at(0, default: "") != "" {
    text(size: 9.5pt, weight: "bold", fill: dark, data.labels.extra.at(0, default: ""))
    v(2pt)
  }
  let location_parts = (data.identity.ville, data.identity.zip_code).filter(v => v != "")
  if location_parts.len() > 0 or data.identity.email != "" or data.identity.telephone != "" or data.identity.permis {
    {
      if location_parts.len() > 0 {
        text(size: 9pt, fill: gray, location_parts.join(", "))
        sep
      }
      if data.identity.email != "" {
        text(size: 9pt, fill: gray, data.identity.email)
        sep
      }
      if data.identity.telephone != "" {
        text(size: 9pt, fill: gray, data.identity.telephone)
        if data.identity.permis {
          sep
        }
      }
      if data.identity.permis {
        text(size: 9pt, fill: gray, data.labels.permis)
      }
    }
    v(1pt)
  }
  if data.identity.linkedin != "" or data.identity.github != "" or data.identity.portfolio != "" {
    {
      let links = (data.identity.linkedin, data.identity.github, data.identity.portfolio).filter(v => v != "")
      text(size: 9pt, fill: gray, links.join([#sep]))
    }
  }
})

// --- Profil ---
if data.labels.section_profile != "" and data.summary != "" {
  section-header(data.labels.section_profile)
  block({
    set par(leading: 0.45em, justify: false)
    text(size: 9.5pt, fill: gray)[#data.summary]
  })
}

// --- Compétences Techniques ---
if data.labels.section_skills != "" and data.skills_groups.len() > 0 {
  section-header(data.labels.section_skills)
  for group in data.skills_groups {
    if group.category != "" and group.skills.len() > 0 {
      skill-line(group.category, group.skills.join(", "))
    }
  }
}

// --- Expérience Professionnelle ---
if data.labels.section_experiences != "" and data.experiences.len() > 0 {
  section-header(data.labels.section_experiences)
  for exp in data.experiences {
    if exp.poste != "" {
      entry-header(exp.poste, exp.entreprise, exp.period)
      for bullet in exp.bullets {
        if bullet != "" {
          bullet-item(bullet)
        }
      }
    }
  }
}

// --- Projets ---
if data.labels.section_projects != "" and data.projects.len() > 0 {
  section-header(data.labels.section_projects)
  for proj in data.projects {
    if proj.titre != "" {
      entry-header(proj.titre, proj.pour_qui, proj.period)
      for bullet in proj.bullets {
        if bullet != "" {
          bullet-item(bullet)
        }
      }
    }
  }
}

// --- Formation ---
if data.labels.section_formations != "" and data.formations.len() > 0 {
  section-header(data.labels.section_formations)
  for edu in data.formations {
    if edu.titre != "" {
      entry-header(edu.titre, edu.ecole, edu.period)
      for detail in edu.details {
        if detail != "" {
          bullet-item(detail)
        }
      }
    }
  }
}

// --- Langues & Centres d'intérêt ---
if data.labels.section_langues != "" {
  section-header(data.labels.section_langues)
  if data.langues.len() > 0 {
    block(below: space-line, {
      for (i, langue) in data.langues.enumerate() {
        if langue.nom != "" {
          if i > 0 {
            text(fill: light-gray, "  |  ")
          }
          text(weight: "bold", fill: dark, langue.nom)
          if langue.level != "" {
            text(fill: dark, " : " + langue.level)
          }
        }
      }
    })
  }
  if data.interets.len() > 0 {
    block(below: space-line, {
      text(weight: "bold", fill: dark, data.labels.interets_label + " : ")
      text(fill: dark, data.interets.join(", "))
    })
  }
}

#v(8pt)
if data.labels.extra.len() > 1 and data.labels.extra.at(1, default: "") != "" {
  text(size: 9pt, fill: gray, style: "italic", data.labels.extra.at(1, default: ""))
}