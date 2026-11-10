//! Règles des prompts de templatisation, partagées par ses deux étapes :
//! sens des champs du schéma canonique (étape A, données d'exemple) et
//! règles d'écriture du template Typst (étape B).

/// Libellé lisible du type de document, pour les prompts.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `"CV"` ou `"lettre de motivation"`.
pub fn doc_type_label(doc_type: &str) -> &'static str {
    match doc_type {
        "cv" => "CV",
        _ => "lettre de motivation",
    }
}

/// Règles des données d'exemple, communes aux deux types de documents.
pub const EXAMPLE_DATA_RULES: &str = "Règles de example_data :
1. Recopie chaque valeur exactement telle qu'elle est écrite dans l'original (mêmes mots, même ponctuation), sauf la casse : une valeur affichée en majuscules par l'original (par exemple le nom) est rangée sous sa forme naturelle (\"Camille\", pas \"CAMILLE\").
2. Un champ absent de l'original vaut une chaîne vide ou une liste vide. N'invente jamais une valeur : chaque valeur se lit dans l'original.
3. `lang` vaut la langue dans laquelle l'original est rédigé : \"fr\" ou \"en\".
4. `identity.permis` vaut true seulement si l'original affiche le permis de conduire.
5. Les commentaires de l'original (// …) ne sont pas du contenu : ignore-les. Une valeur ne contient jamais de saut de ligne : les lignes d'un même paragraphe sont jointes par un espace.";

/// Règle du lieu de l'expéditeur, pour les données d'exemple.
pub const ADDRESS_DATA_RULE: &str = "Lieu de l'expéditeur :
- Le lieu du candidat tient en deux champs : `identity.ville` et `identity.zip_code`, repris exactement tels qu'affichés dans l'original (chaîne vide si absent). Une rue, un numéro ou un pays de l'expéditeur n'ont pas de champ.";

/// Sens des champs du CV, pour les données d'exemple.
pub const CV_DATA_FIELDS: &str = "Champs du CV :
- identity : coordonnées du candidat affichées dans l'original. identity.title : sous-titre professionnel affiché sous le nom, chaîne vide si l'original n'en a pas.
- summary : texte de la section profil.
- skills_groups : une entrée par ligne de compétences, la catégorie puis les compétences.
- experiences, projects : une entrée par expérience ou projet, avec ses bullets. `pour_qui` est l'organisation affichée pour un projet. projects.stack et projects.url : technologies et lien affichés pour un projet, chaîne vide si l'original ne les affiche pas.
- formations : une entrée par formation ; `details` reçoit ses lignes de détail (spécialité, cours, échange).
- langues : nom et niveau de chaque langue. interets : un centre d'intérêt par élément.
- period : la période telle qu'affichée. period_style indique pour chaque section si les périodes affichent le mois (\"month\") ou seulement l'année (\"year\") ; \"month\" pour une section absente.
- labels.section_* : titres des sections, tels qu'écrits dans le code source de l'original. labels.interets_label : le préfixe de la ligne des centres d'intérêt (par exemple \"Intérêts :\"). labels.permis : le texte affiché pour le permis.
- labels.months_short : les 12 abréviations de mois, de janvier à décembre, dans la langue et le style de l'original (\"01\"…\"12\" si l'original écrit les mois en chiffres).
- labels.present : le mot qui termine une période en cours dans la langue de l'original (par exemple \"aujourd'hui\"), même s'il n'apparaît pas.";

/// Sens des champs de la lettre, pour les données d'exemple.
pub const LM_DATA_FIELDS: &str = "Champs de la lettre :
- identity : coordonnées de l'expéditeur affichées dans l'original. identity.title : sous-titre professionnel affiché sous le nom, chaîne vide si absent.
- recipient : destinataire. company : l'entreprise. location : la dernière ligne de l'adresse, celle qui porte la ville (par exemple \"1201 Genève\"). extra_lines : les autres lignes du destinataire (service, rue), une par élément, dans l'ordre ; jamais la ligne de la ville.
- date : la date affichée, sans la ville ni le mot qui la précède (par exemple \"13 octobre 2026\"). Si l'original calcule la date au lieu de l'écrire, chaîne vide.
- labels.date_prefix : le mot entre la ville et la date (par exemple \"le\"), chaîne vide s'il n'y en a pas.
- subject : le texte de l'objet, sans son préfixe. labels.subject_prefix : le préfixe (par exemple \"Objet :\").
- labels.salutation : la formule d'appel. labels.closing : la formule de politesse finale, en entier.
- paragraphs : les paragraphes du corps, entre la formule d'appel et la formule de politesse. Une entrée = un paragraphe de l'original, c'est-à-dire ce qui est séparé du texte voisin par une ligne vide, #v, parbreak(), \\ ou un saut de bloc. Des lignes séparées seulement par un commentaire forment un seul paragraphe.";

/// Règles d'écriture du template, communes aux deux types de documents.
pub const TEMPLATE_RULES: &str = "Règles du template :
1. La première ligne est `#let data = json(\"data.json\")`. Recopie ensuite à l'identique le préambule (#set, #show), les polices, les couleurs, les fonctions définies avec #let, les espacements et les commentaires. Seule exception : la langue du texte devient `lang: data.lang`.
2. Rendu avec example_data, le template affiche exactement le texte de l'original, mot pour mot, avec la même mise en page et le même nombre de pages. Rien n'est retiré.
3. Tout texte visible vient de data : un champ du schéma (identité, expériences, prose…), un libellé (data.labels.…) ou un texte de la référence (data.texts.<id>, liste fournie). Seuls la ponctuation et les séparateurs (« | », « • », « : », « , ») restent écrits en dur. Une valeur d'un champ n'est jamais écrite en dur, même au milieu d'une phrase recopiée.
4. Mode markup et mode code : dans le corps du document et dans un bloc de contenu [ … ], chaque instruction commence par # (#data.summary, #if …, #for …, #text(…)[…]). Dans un bloc de code { … } ou une expression #( … ), on est déjà en mode code : jamais de #. Du code écrit en markup sans # s'affiche comme du texte.
5. Insère les valeurs directement : #data.identity.nom en markup, text(data.summary) en code. Pour juxtaposer deux valeurs, écris-les côte à côte (`#data.identity.prenom #data.identity.nom`) ; ne construis pas une ligne affichée en concaténant des chaînes avec +, ni avec un tableau et .join().
6. Logique minimale : un `if` fait disparaître un champ vide avec sa ligne, ou avec le séparateur qui l'accompagne (le séparateur est placé dans le `if`) ; une boucle `for` rend chaque liste. Une section dont la liste ou le titre est vide disparaît entièrement, titre compris. Un élément de liste n'est jamais écrit à la main.
7. Une valeur affichée transformée (par exemple le nom en majuscules) est transformée dans le template (`upper(data.identity.nom)`).
8. N'utilise jamais `eval`, `datetime.today` ni `read`. Une date calculée par l'original devient data.date.
9. Le template compile avec Typst 0.15, y compris quand tous les champs sont vides et toutes les listes vides.";

/// Rendu propre au CV.
pub const CV_TEMPLATE_FIELDS: &str = "Rendu du CV :
- Les coordonnées s'affichent dans l'ordre de l'original ; `data.labels.permis` s'affiche à la place du permis, seulement si `data.identity.permis` est vrai.
- Le lieu se compose de `data.identity.ville` et `data.identity.zip_code` dans l'ordre et avec la ponctuation de l'original ; le séparateur n'apparaît que si les deux sont non vides.
- skills_groups : une ligne par groupe, la catégorie puis `g.skills.join(\", \")`.
- experiences, projects, formations : une boucle par section, chaque entrée avec ses bullets (ou details). Un élément affiché avec une mise en page différente des autres se choisit dans la boucle d'après ses données.
- langues : le séparateur entre le nom et le niveau ne s'affiche que si le niveau n'est pas vide. interets : joints par \", \" après `data.labels.interets_label`.";

/// Rendu propre à la lettre.
pub const LM_TEMPLATE_FIELDS: &str = "Rendu de la lettre :
- La signature affiche `data.identity.prenom` et `data.identity.nom`.
- Destinataire : `data.recipient.company`, puis une boucle sur `data.recipient.extra_lines`, puis `data.recipient.location` ; chaque ligne n'apparaît que si elle est non vide.
- Ligne de date : `data.identity.ville` suivi de la ponctuation de l'original (seulement si la ville est non vide), `data.labels.date_prefix`, puis `data.date`. Jamais le code postal.
- Objet : `data.labels.subject_prefix` puis `data.subject`.
- Corps : `data.labels.salutation`, puis une boucle sur `data.paragraphs` où chaque entrée devient un paragraphe distinct, séparé du précédent comme dans l'original (par exemple `#for p in data.paragraphs [#p #parbreak()]`), puis `data.labels.closing`. Jamais `.join()` ni `.at(i)` sur paragraphs.";
