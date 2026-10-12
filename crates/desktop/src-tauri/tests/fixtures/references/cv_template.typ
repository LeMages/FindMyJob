#let data = json("data.json")
// Template de CV de test : toutes les valeurs viennent de data.json ; une
// valeur vide fait disparaître sa ligne ou son séparateur.

#set page(paper: "a4", margin: 1.5cm)
#set text(size: 10pt, lang: data.lang)

#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

#align(center, {
  text(size: 16pt, weight: "bold", upper(data.identity.prenom + " " + data.identity.nom))
  linebreak()
  let parts = (data.identity.adresse, data.identity.email, data.identity.telephone, data.identity.linkedin)
  if data.identity.permis { parts.push(data.labels.permis) }
  text(joined(parts, "  •  "))
  linebreak()
  text(style: "italic", data.title)
})

#if data.summary != "" {
  heading(data.labels.section_profile)
  text(data.summary)
}

#if data.skills_groups.len() > 0 {
  heading(data.labels.section_skills)
  for g in data.skills_groups {
    block(strong(g.category) + " " + g.skills.join(", "))
  }
}

#if data.experiences.len() > 0 {
  heading(data.labels.section_experiences)
  for e in data.experiences {
    block(joined((e.poste, e.entreprise, e.lieu, e.period), " | "))
    list(..e.bullets)
  }
}

#if data.projects.len() > 0 {
  heading(data.labels.section_projects)
  for p in data.projects {
    block(joined((p.titre, p.pour_qui, p.period), " | "))
    list(..p.bullets)
  }
}

#if data.formations.len() > 0 {
  heading(data.labels.section_formations)
  for f in data.formations {
    block(joined((f.titre, f.ecole, f.period), " | "))
    for d in f.details { block(d) }
  }
}

#if data.langues.len() > 0 {
  heading(data.labels.section_langues)
  for l in data.langues {
    block(joined((l.nom, l.level), " : "))
  }
}

#if data.interets.len() > 0 and data.labels.interets_label != "" {
  text(data.labels.interets_label + " " + data.interets.join(", "))
}
