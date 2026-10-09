// Template de CV de test : lit toutes ses valeurs depuis data.json, masque
// les lignes et sections vides, sans séparateur orphelin.
#let data = json("data.json")

#set page(paper: "a4", margin: 1.5cm)
#set text(size: 10pt, lang: data.lang)

#align(center, {
  text(size: 16pt, weight: "bold", upper(data.identity.prenom + " " + data.identity.nom))
  linebreak()
  let parts = (data.identity.ville, data.identity.email, data.identity.telephone, data.identity.linkedin)
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
    list(..e.bullets)
  }
}

#if data.projects.len() > 0 {
  heading(data.labels.section_projects)
  for p in data.projects {
    block(strong(p.titre) + " | " + p.period)
    list(..p.bullets)
  }
}

#if data.interets.len() > 0 {
  text(data.labels.interets_label + " : " + data.interets.join(", "))
}
