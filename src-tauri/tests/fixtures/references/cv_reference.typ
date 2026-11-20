// Référence de CV de test, conforme au contrat FindMyJob : valeurs
// d'exemple, données injectées par sys.inputs.data, libellés FR/EN.

#let example = (
  lang: "fr",
  identity: (
    prenom: "Camille",
    nom: "Exemple",
    title: "Exemple de titre professionnel",
    email: "camille@exemple.invalid",
    telephone: "00 00 00 00 00",
    ville: "Exempleville",
    zip_code: "00000",
    linkedin: "",
    github: "",
    portfolio: "",
    permis: false,
  ),
  summary: "Exemple de résumé du profil, qui présente le parcours et les envies de la candidate en quelques phrases simples.",
  skills_groups: (
    (category: "Exemple outils", skills: ("ExempleSQL", "ExemplePython")),
  ),
  experiences: (
    (
      poste: "Exemple de poste",
      entreprise: "Exemple SARL",
      lieu: "Exempleville",
      period: "Janv. 2024 - Juin 2024",
      intro: "",
      bullets: (
        "Exemple de réalisation chiffrée sur un projet interne.",
        "Exemple de mission menée avec une équipe.",
      ),
    ),
  ),
  projects: (
    (
      titre: "Exemple de projet",
      pour_qui: "Exemple association",
      period: "2023",
      stack: "",
      url: "",
      intro: "",
      bullets: ("Exemple de livrable présenté au client.",),
    ),
  ),
  formations: (
    (
      titre: "Exemple de master",
      ecole: "Université Exemple",
      period: "2022 - 2024",
      intro: "",
      details: ("Exemple de spécialité",),
    ),
  ),
  langues: ((nom: "Exemplais", level: "courant"),),
  list_marker: "•",
)

#let data = if "data" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }

#metadata((
  contract: "findmyjob/cv",
  version: 1,
  languages: ("fr", "en"),
  example: example,
  guidance: "",
  slots: (
    (field: "identity.title", mode: "variable", instruction: ""),
    (field: "summary", mode: "adaptable", instruction: ""),
  ),
  period: (
    granularity: (experiences: "month", projects: "year", formations: "year"),
    separator: " - ",
    month_style: "short",
  ),
)) <findmyjob>

#let L = (
  fr: (
    profile: "Profil",
    skills: "Compétences",
    experiences: "Expériences",
    projects: "Projets",
    formations: "Formations",
    langues: "Langues",
    permis: "Permis B",
  ),
  en: (
    profile: "Profile",
    skills: "Skills",
    experiences: "Experience",
    projects: "Projects",
    formations: "Education",
    langues: "Languages",
    permis: "Driving licence",
  ),
).at(data.lang)

#set page(paper: "a4", margin: 1.5cm)
#set text(size: 10pt, lang: data.lang)

#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

#align(center, {
  text(size: 16pt, weight: "bold", upper(joined((data.identity.prenom, data.identity.nom), " ")))
  linebreak()
  let parts = (joined((data.identity.zip_code, data.identity.ville), " "), data.identity.email, data.identity.telephone, data.identity.linkedin)
  if data.identity.permis { parts.push(L.permis) }
  text(joined(parts, "  •  "))
  if data.identity.title != "" {
    linebreak()
    text(style: "italic", data.identity.title)
  }
})

#if data.summary != "" {
  heading(L.profile)
  text(data.summary)
}

#if data.skills_groups.len() > 0 {
  heading(L.skills)
  for g in data.skills_groups {
    block(strong(g.category) + " " + g.skills.join(", "))
  }
}

#if data.experiences.len() > 0 {
  heading(L.experiences)
  for e in data.experiences {
    block(joined((e.poste, e.entreprise, e.lieu, e.period), " | "))
    if e.intro != "" { block(e.intro) }
    list(marker: data.list_marker, ..e.bullets)
  }
}

#if data.projects.len() > 0 {
  heading(L.projects)
  for p in data.projects {
    block(joined((p.titre, p.pour_qui, p.period), " | "))
    if p.stack != "" { block(p.stack) }
    if p.url != "" { block(p.url) }
    if p.intro != "" { block(p.intro) }
    list(marker: data.list_marker, ..p.bullets)
  }
}

#if data.formations.len() > 0 {
  heading(L.formations)
  for f in data.formations {
    block(joined((f.titre, f.ecole, f.period), " | "))
    if f.intro != "" { block(f.intro) }
    list(marker: data.list_marker, ..f.details)
  }
}

#if data.langues.len() > 0 {
  heading(L.langues)
  for l in data.langues {
    block(joined((l.nom, l.level), " : "))
  }
}
