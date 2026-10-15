#let data = json("data.json")
// Template de lettre de test : toutes les valeurs viennent de data.json ;
// une valeur vide fait disparaître sa ligne ou son séparateur.

#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt, lang: data.lang)

#let joined(values, sep) = values.filter(v => v != "").join(sep, default: "")

#block(strong(joined((data.identity.prenom, data.identity.nom), " ")))
#for line in (joined((data.identity.ville, data.identity.zip_code), ", "), data.identity.email, data.identity.telephone).filter(v => v != "") {
  block(line)
}

#align(right, {
  for line in (data.recipient.company, data.recipient.location).filter(v => v != "") {
    block(line)
  }
})

#if data.date != "" {
  align(right, text(joined((data.identity.ville, joined((data.labels.date_prefix, data.date), " ")), ", ")))
}

#if data.subject != "" {
  block(strong(joined((data.labels.subject_prefix, data.subject), " ")))
}

#if data.labels.salutation != "" { block(data.labels.salutation) }

#for p in data.paragraphs {
  par(justify: true, p)
  v(0.5em)
}

#if data.labels.closing != "" { block(data.labels.closing) }

#align(right, text(joined((data.identity.prenom, data.identity.nom), " ")))
