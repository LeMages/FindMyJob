// Template de CV de test : lit toutes ses valeurs depuis data.json, masque
// les lignes et sections vides, sans séparateur orphelin.
#let data = json("data.json")

#set page(paper: "a4", margin: 1.5cm)
#set text(size: 10pt, lang: data.lang)

#align(center, {
  text(size: 16pt, weight: "bold", upper(data.identity.prenom + " " + data.identity.nom))
  linebreak()
  let lieu = (data.identity.zip_code, data.identity.ville).filter(p => p != "").join(" ", default: "")
  let parts = (lieu, data.identity.email, data.identity.telephone, data.identity.linkedin)
    .filter(p => p != "")
  if data.identity.permis { parts.push(data.labels.permis) }
  text(parts.join("  •  "))
})

#if data.summary != "" {
  heading(data.labels.section_profile)
  text(data.summary)
}

#if data.experiences.len() > 0 {
  heading(data.labels.section_experiences)
  for e in data.experiences {
    block(strong(e.poste) + " | " + e.entreprise + " | " + e.period)
    if e.intro != "" { block(e.intro) }
    list(marker: data.list_marker, ..e.bullets)
  }
}

#if data.projects.len() > 0 {
  heading(data.labels.section_projects)
  for p in data.projects {
    block(strong(p.titre) + " | " + p.period)
    if p.intro != "" { block(p.intro) }
    list(marker: data.list_marker, ..p.bullets)
  }
}
