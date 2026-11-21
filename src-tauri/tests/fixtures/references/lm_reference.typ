// Référence de lettre de test, conforme au contrat FindMyJob : valeurs
// d'exemple, données injectées par sys.inputs.data, textes fixes FR/EN.

#let example = (
  lang: "fr",
  identity: (
    prenom: "Camille",
    nom: "Exemple",
    title: "",
    email: "camille@exemple.invalid",
    telephone: "00 00 00 00 00",
    ville: "Exempleville",
    zip_code: "00000",
    linkedin: "",
    github: "",
    portfolio: "",
    permis: false,
  ),
  recipient: (company: "Exemple Corp", location: "Exempleville", extra_lines: ()),
  date: "1 janvier 2026",
  subject: "Candidature exemple pour un stage",
  paragraphs: (
    "Exemple de premier paragraphe qui présente la candidate et explique pourquoi l'entreprise l'intéresse particulièrement.",
    "Exemple de second paragraphe qui relie une expérience passée aux missions décrites dans l'offre de stage.",
    "Exemple de dernier paragraphe qui propose un entretien pour échanger sur le poste.",
  ),
)

#let data = if "data" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }

#metadata((
  contract: "findmyjob/lettre",
  version: 1,
  languages: ("fr", "en"),
  example: example,
  guidance: "",
  slots: (
    (field: "subject", mode: "variable", instruction: ""),
  ),
)) <findmyjob>

#let L = (
  fr: (
    subject_prefix: "Objet :",
    salutation: "Madame, Monsieur,",
    closing: "Je vous prie d'agréer, Madame, Monsieur, mes salutations distinguées.",
    date_prefix: "le",
  ),
  en: (
    subject_prefix: "Subject:",
    salutation: "Dear Sir or Madam,",
    closing: "Yours faithfully,",
    date_prefix: "",
  ),
).at(data.lang)

#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt, lang: data.lang)

#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

#block(strong(joined((data.identity.prenom, data.identity.nom), " ")))
#for line in (joined((data.identity.ville, data.identity.zip_code), ", "), data.identity.email, data.identity.telephone).filter(v => v != "") {
  block(line)
}

#align(right, {
  for line in (data.recipient.company, ..data.recipient.extra_lines, data.recipient.location).filter(v => v != "") {
    block(line)
  }
})

#if data.date != "" {
  align(right, text(joined((data.identity.ville, joined((L.date_prefix, data.date), " ")), ", ")))
}

#if data.subject != "" {
  block(strong(joined((L.subject_prefix, data.subject), " ")))
}

#block(L.salutation)

#for p in data.paragraphs {
  par(justify: true, p)
  v(0.5em)
}

#block(L.closing)

#align(right, text(joined((data.identity.prenom, data.identity.nom), " ")))
