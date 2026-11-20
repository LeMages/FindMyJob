# Contrat FindMyJob des références CV et lettre (version 1)

FindMyJob génère votre CV et votre lettre de motivation à partir de **vos propres documents de référence**, écrits en [Typst](https://typst.app). Pour qu'une référence soit utilisable, elle doit respecter ce contrat. L'application la vérifie à l'import, de façon déterministe et sans aucun appel à l'IA : si un point n'est pas respecté, l'import est refusé et chaque problème est listé (avec la ligne ou le champ concerné).

Deux exemples complets et conformes, pour un étudiant fictif : [`cv_exemple.typ`](cv_exemple.typ) et [`lm_exemple.typ`](lm_exemple.typ). Le plus simple est de partir de l'un d'eux, ou d'utiliser le bouton « Copier le prompt pour Claude » de l'application pour faire convertir votre document actuel.

## 1. Forme du fichier

Un seul fichier `.typ`, en UTF-8, de 512 Kio au plus, qui contient dans cet ordre :

```typst
// 1. Les valeurs d'exemple, dans la forme exacte des données (section 3)
#let example = (
  lang: "fr",
  identity: (prenom: "Antoine", nom: "Dupont", /* … */),
  // …
)

// 2. Les données : celles de FindMyJob, ou l'exemple quand le fichier est compilé seul
#let data = if "data" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }

// 3. La métadonnée du contrat (section 2)
#metadata((
  contract: "findmyjob/cv",
  version: 1,
  languages: ("fr", "en"),
  example: example,
  guidance: "Consignes générales de rédaction.",
  slots: (
    (field: "summary", mode: "adaptable", instruction: "Consigne de cet emplacement."),
  ),
  period: (
    granularity: (experiences: "month", projects: "year", formations: "year"),
    separator: " - ",
    month_style: "short",
  ),
)) <findmyjob>

// 4. Les textes fixes, dans chaque langue
#let L = (
  fr: (experiences: "Expériences", /* … */),
  en: (experiences: "Experience", /* … */),
).at(data.lang)

// 5. La mise en page, libre, qui ne lit que `data` et `L`
```

Le fichier doit compiler seul (il s'affiche alors avec ses valeurs d'exemple). FindMyJob le compile ensuite avec les vraies données, passées dans `sys.inputs.data`.

**Interdit** (repéré par analyse syntaxique, avant toute compilation) :

- `#import` et `#include` (y compris les paquets `@preview/…`) ;
- `read(…)`, `eval(…)`, `plugin(…)` ;
- `image("fichier")`, `json("fichier")`, `csv(…)`, `yaml(…)`, `toml(…)`, `xml(…)`, `cbor(…)` avec un chemin de fichier ;
- `datetime.today` (la date de la lettre est fournie dans `data.date`).

**Autorisé** : une icône ou un logo collé dans le fichier, par exemple `image(bytes("<svg …>…</svg>"))`.

> **Piège Typst** : un tableau d'un seul élément s'écrit avec une virgule finale, `("Python",)`. Sans elle, `("Python")` est une simple chaîne, et l'exemple est refusé (`example_invalid`).

## 2. La métadonnée `<findmyjob>`

Exactement une `#metadata((…)) <findmyjob>` par fichier, avec :

| Champ | Contenu |
|---|---|
| `contract` | `"findmyjob/cv"` pour un CV, `"findmyjob/lettre"` pour une lettre. Doit correspondre au type importé. |
| `version` | `1`, la version actuelle du contrat. |
| `languages` | Les langues dans lesquelles le fichier sait s'afficher. Doit contenir toutes les langues de l'application : `("fr", "en")`. |
| `example` | Les valeurs d'exemple (`example`), dans la forme exacte des données (section 3). |
| `guidance` | Facultatif. Consignes générales pour la rédaction (ton, longueur, ce qu'il ne faut jamais faire). |
| `slots` | Facultatif. Les emplacements rédigés et leur consigne (section 4). |
| `period` | CV seulement : le format des périodes (section 5). Ignoré pour une lettre. |

Les valeurs d'exemple servent de modèle : longueur et style des textes rédigés, nombre de projets et de groupes de compétences, nombre de paragraphes de la lettre, nombre de pages (le document généré n'en a jamais plus). Les catégories montrées par l'exemple (expériences, projets, formations, compétences, langues, permis, coordonnées) sont signalées si votre profil ne les remplit pas.

## 3. Les données (`data`)

Toutes les valeurs sont des chaînes (`""` quand elles sont vides, jamais `none`), sauf `permis` (booléen) et les listes.

### Commun au CV et à la lettre

| Champ | Contenu |
|---|---|
| `lang` | Langue du document : `"fr"` ou `"en"` (celle de l'offre). Choisit les textes fixes : `L = (fr: …, en: …).at(data.lang)`. |
| `identity.prenom`, `identity.nom` | Prénom et nom. |
| `identity.title` | Titre professionnel sous le nom, rédigé pour l'offre. |
| `identity.email`, `identity.telephone` | Coordonnées. |
| `identity.ville`, `identity.zip_code` | Ville et code postal de la session de recherche. |
| `identity.linkedin`, `identity.github`, `identity.portfolio` | Liens, en texte. |
| `identity.permis` | `true` si vous avez le permis de conduire. |

### CV (`findmyjob/cv`)

| Champ | Contenu |
|---|---|
| `summary` | Résumé du profil, rédigé pour l'offre. |
| `skills_groups[]` | Groupes de compétences : `category` (nom du groupe) et `skills[]` (compétences). |
| `experiences[]` | `poste`, `entreprise`, `lieu`, `period`, `intro`, `bullets[]`. |
| `projects[]` | `titre`, `pour_qui`, `period`, `stack`, `url`, `intro`, `bullets[]`. |
| `formations[]` | `titre`, `ecole`, `period`, `intro`, `details[]`. |
| `langues[]` | `nom` et `level`. |
| `list_marker` | Symbole de liste choisi dans la session (`•` ou `-`). |

`intro` est le texte d'une description avant sa liste, sans symbole. Chaque élément de `bullets` ou de `details` peut contenir un retour à la ligne (`\n`) : affichez la chaîne telle quelle. `period` est déjà formatée (section 5).

### Lettre (`findmyjob/lettre`)

| Champ | Contenu |
|---|---|
| `recipient.company` | Entreprise de l'offre. |
| `recipient.extra_lines[]` | Lignes d'adresse intermédiaires (service, rue…). |
| `recipient.location` | Dernière ligne de l'adresse (ville). |
| `date` | Date du jour, déjà écrite dans la langue : « 13 octobre 2026 », « 13 October 2026 ». Le lieu et la formule (« Lyon, le ») sont à composer dans le fichier. |
| `subject` | Objet de la lettre, rédigé pour l'offre (il contient l'intitulé exact du poste). |
| `paragraphs[]` | Les paragraphes rédigés, dans l'ordre. Leur nombre est celui de l'exemple. |

Le texte fixe de la lettre (formule d'appel, phrases toujours identiques, formule de politesse, signature) est écrit dans le fichier, dans chaque langue, autour des paragraphes rédigés. Un même paragraphe peut mêler texte fixe et texte rédigé : `#L.intro #data.paragraphs.at(0)`.

## 4. Emplacements rédigés (`slots`)

Les champs rédigés par l'IA sont `identity.title` et `summary` pour un CV, `subject` et `paragraphs[i]` pour une lettre (`i` à partir de 0, inférieur au nombre de paragraphes de l'exemple). Chaque emplacement déclare :

| Champ | Contenu |
|---|---|
| `field` | Le champ concerné (`"summary"`, `"paragraphs[1]"`…). Un champ ne peut avoir qu'un emplacement. |
| `mode` | `"variable"` : le texte d'exemple est remplacé par les informations tirées de l'offre, en suivant la consigne. `"adaptable"` : le texte est rédigé à partir du profil selon l'offre, sans rien inventer. |
| `instruction` | La consigne propre à cet emplacement (facultative). |

Un champ rédigé sans emplacement est rédigé librement à partir du profil et de l'offre. Le rédacteur reçoit aussi le rendu complet de votre exemple, pour en reprendre le style et éviter de répéter les parties fixes voisines.

## 5. Périodes (CV)

FindMyJob formate `period` lui-même, à partir des dates du profil, selon `period` dans la métadonnée :

| Champ | Contenu |
|---|---|
| `granularity` | Pour `experiences`, `projects` et `formations` : `"month"` (mois et année) ou `"year"` (année seule). |
| `separator` | Texte entre les deux dates, par exemple `" - "` ou `" à "`. Un tiret « – » ou « — » est remplacé par « - ». |
| `month_style` | Facultatif, `"short"` par défaut : `"short"` (« Nov. 2025 », « Nov 2025 »), `"long"` (« novembre 2025 », « November 2025 ») ou `"numeric"` (« 11.2025 »). |
| `numeric_separator` | Facultatif, `"/"` par défaut : séparateur entre mois et année en style `"numeric"`. |

Une période en cours se termine par « présent » / « present » ; une période dont les deux bornes s'écrivent pareil n'en affiche qu'une (« 2025 »).

## 6. Conventions obligatoires

1. **Aucune donnée en dur.** Tout ce qui vient du profil, de la session, de l'offre ou de la rédaction est lu dans `data`. FindMyJob le vérifie en remplaçant chaque valeur d'exemple par un marqueur : une valeur d'identité ou du profil encore visible est refusée.
2. **Ce qui est vide disparaît.** Une ligne, une section ou un séparateur dont la donnée est vide n'est pas affiché. Le plus simple : `values.filter(v => v != "").join(" | ")`.
3. **Titres de section en `heading`.** Écrivez chaque titre de section avec `heading(…)` et mettez-le en forme avec une règle `#show heading: …`. Un titre dont la section est vide doit disparaître avec elle.
4. **Symbole de liste.** Dans un CV, `data.list_marker` précède chaque élément de `bullets` et de `details` (par exemple `list(marker: data.list_marker, ..e.bullets)`), et seulement eux : aucun symbole de liste n'est écrit en dur. Les séparateurs d'en-tête (« • » entre les coordonnées) ne sont pas des symboles de liste.
5. **Textes fixes dans chaque langue.** Titres, libellés, mentions et phrases fixes existent en français et en anglais, choisis par `data.lang`. Une catégorie que FindMyJob ne connaît pas (centres d'intérêt, références…) reste un texte fixe, dans chaque langue.
6. **Nombre de pages.** Dans chaque langue, le document rendu avec l'exemple ne doit pas dépasser le nombre de pages de l'exemple.
7. **Destinataire de la lettre.** S'il est à droite, il est aussi large que sa plus longue ligne, calé contre la marge droite, texte aligné à gauche : `#align(right, box({ set align(left); … }))`.
8. **Pas de tiret cadratin ni demi-cadratin** (« — », « – ») dans le rendu : utilisez « - ».

## 7. Polices

Une police doit être disponible au moment de la compilation : polices fournies par Typst (Libertinus Serif, New Computer Modern, DejaVu Sans Mono), polices installées sur l'ordinateur, ou polices importées dans **Paramètres > Polices** (le plus sûr : elles suivent l'application). Une famille introuvable est une erreur, jamais un remplacement silencieux.

## 8. Erreurs et avertissements

Une **erreur** bloque l'import ; un **avertissement** est affiché mais l'import réussit. Tous les problèmes d'un fichier sont listés ensemble.

| Code | Type | Explication |
|---|---|---|
| `not_typ` | Erreur | Le fichier n'est pas un fichier Typst (`.typ`). |
| `too_large` | Erreur | Le fichier dépasse 512 Kio. |
| `not_utf8` | Erreur | Le fichier n'est pas un texte UTF-8. |
| `forbidden_construct` | Erreur | Construction interdite (section 1), avec sa ligne. |
| `compile_error` | Erreur | Le fichier ne compile pas avec ses valeurs d'exemple ; le message de Typst et la ligne sont donnés. |
| `missing_font` | Erreur | Une police demandée est introuvable (section 7). |
| `metadata_missing` | Erreur | Aucune métadonnée `<findmyjob>`. |
| `metadata_duplicated` | Erreur | Plusieurs métadonnées `<findmyjob>`. |
| `metadata_invalid` | Erreur | Métadonnée illisible : champ manquant, en trop ou mal typé. |
| `wrong_contract` | Erreur | `contract` ne correspond pas au type importé (CV ou lettre). |
| `outdated_contract_version` | Erreur | `version` n'est pas la version actuelle du contrat. |
| `language_missing` | Erreur | Une langue de l'application manque dans `languages`. |
| `example_invalid` | Erreur | Une valeur d'exemple est manquante, en trop ou mal typée ; le chemin du champ est donné (`example.experiences[1].period`). |
| `slot_unknown_field` | Erreur | Emplacement sur un champ qui n'est pas rédigé ou absent de l'exemple. |
| `slot_invalid_mode` | Erreur | Mode d'emplacement autre que `variable` ou `adaptable`. |
| `slot_duplicated` | Erreur | Deux emplacements sur le même champ. |
| `language_compile_error` | Erreur | Le fichier ne compile pas dans une des langues (souvent un texte fixe manquant dans `L`). |
| `page_count_mismatch` | Erreur | Dans une des langues, le document a plus de pages qu'avec l'exemple. |
| `empty_data_compile_error` | Erreur | Le fichier ne compile pas avec des données vides (listes vides, chaînes vides). |
| `hardcoded_value` | Erreur | Une valeur d'exemple d'identité ou du profil est écrite en dur au lieu d'être lue dans `data`. |
| `list_marker_wrong` | Erreur | `data.list_marker` ne précède pas exactement chaque élément de liste, ou un symbole est écrit en dur. |
| `orphan_heading` | Erreur | Un titre de section reste affiché alors que sa section est vide. |
| `orphan_separator` | Avertissement | Un séparateur reste affiché quand un champ secondaire est vide ; le champ est donné. |
| `recipient_not_right_aligned` | Avertissement | Le bloc destinataire n'est pas calé contre la marge droite quand il est court. |
| `dash_marks` | Avertissement | Le rendu contient des tirets « — » ou « – ». |

## 9. Anciennes références

Les références importées avant ce contrat (balisage `[CONSIGNES]`, `[FIXE]`… en commentaires) ne sont plus acceptées : la génération demande de les réimporter. Convertissez-les avec le bouton « Copier le prompt pour Claude ».
