# Fixtures des tests desktop

Aucune fixture ne contient de script exécutable, de traceur ni de donnée personnelle. Les tests ne vont jamais chercher ces pages sur Internet : elles sont servies par un serveur local (`tests/integration/shared/offer_server.rs`) ou passées directement aux services.

## `offers/` : pages d'offres

| Fichier | Source (récupérée le 2026-10-12) | Réduction | Utilisée pour |
|---|---|---|---|
| `ashby_json_ld.html` | Offre ouverte Doctolib « Stage - Assistant Satisfaction (x/f/m) - janvier 2027 », `jobs.ashbyhq.com/doctolib/b25c9448-8e74-45ac-b8ab-2017a49ce3a9`, récupérée par une requête HTTP simple | Gardés : le `<title>`, le bloc JSON-LD `JobPosting` (sans `logo`, `sameAs` ni `identifier`) et la coquille JavaScript d'origine (`<noscript>`, `<div id="root">`). Retirés : tous les autres scripts et feuilles de style | Import par lien, étape JSON-LD |
| `smartrecruiters_sans_json_ld.html` | Offre ouverte Ubisoft « Media Assistant - Stage (6 mois) Janvier 2027 (F/H/NB) », `jobs.smartrecruiters.com/Ubisoft2/744000154670714`, récupérée par une requête HTTP simple. La page n'a que du microdata, pas de JSON-LD | Seul le `<main>` est gardé, avec tous les attributs retirés et les liens remplacés par `#`. Retirés aussi : scripts, styles, images, boutons, bandeau cookies et la FAQ de fin (sauf un paragraphe qui garde un emoji d'origine) | Import par lien, étape LLM |
| `indeed_challenge.html` | Page de vérification anti-bot qu'Indeed renvoie (HTTP 401) à une requête HTTP simple sur `fr.indeed.com/viewjob` | Retirés : scripts, styles, images inline en base64. Le lien de retour est remplacé par `#` | Page protégée (401) et page challenge servie en 200 |
| `wttj_capture.html` | Offre ouverte Doctolib « Stage - Product Content Manager Junior - Nantes (x/f/m) » sur Welcome to the Jungle. Le site renvoie un 202 vide à une requête HTTP simple. La page a été rendue dans un navigateur et son contenu relevé (texte visible et champs du JSON-LD `JobPosting`) | **Reconstruction** : l'extension de lecture refusait de renvoyer le HTML brut. Le fichier est un HTML minimal rebâti à partir du texte réel de la page (extraits du descriptif), avec un bloc JSON-LD qui reprend les vrais `title`, `employmentType`, `hiringOrganization` et `jobLocation` et une description réduite. URL, logos et identifiants ont été retirés | HTML capturé passé à `services::scraping::extract_from_html` |

## `references/` : références CV et lettre

Toutes les références suivent le contrat FindMyJob (`docs/reference-contract/`) : un `.typ` autonome qui déclare ses valeurs d'exemple dans `example`, lit `data` dans `sys.inputs.data` (l'exemple quand il est absent) et décrit sa forme dans une métadonnée `<findmyjob>` (contrat, langues, consignes, emplacements rédigés, format des périodes pour le CV). Les tests les importent avec `documents::save_reference`, sans aucun appel LLM.

`cv_reference.typ` et `lm_reference.typ` sont des références écrites pour les tests. Une valeur vide fait disparaître sa ligne sans séparateur orphelin, et toutes les valeurs d'exemple contiennent « Exemple », ce qui permet de vérifier que la génération ne les recopie jamais. Elles sont importées par `test_state::seed_references` et compilées par les tests unitaires de `src/docgen/tests.rs`.

Le symbole de chaque élément de description du CV vient de `data.list_marker` (« • » dans les valeurs d'exemple, celui de la session à la génération), y compris pour les formations, et l'`intro` d'une description s'affiche avant ses éléments, sans symbole. Les titres de section sont des `heading` Typst : un titre dont la section est vide est détecté à l'import.

### `references/tagged/` : références fictives au contrat

| Fichier | Source | Réduction |
|---|---|---|
| `cv_balise.typ` | Référence fictive d'Antoine Dupont, même personnage que `docs/reference-contract/cv_exemple.typ` | Identité, parcours, compétences, projets et ligne « Intérêts » fictifs (texte fixe de la référence, en français et en anglais : le profil n'a pas d'intérêts), disponibilité « mars 2027 ». Emplacements `variable` sur le titre, `adaptable` sur le résumé ; périodes au format numérique (« 09.2025 ») ; tirets « - » à la place des « – » |
| `lm_balisee.typ` | Référence fictive d'Antoine Dupont, même personnage que `docs/reference-contract/lm_exemple.typ` | Mêmes valeurs fictives que le CV, date « 13 octobre 2026 ». Textes fixes en français et en anglais autour des paragraphes rédigés (`paragraphs[0]` et `[2]` variables, `[1]` adaptable), objet = préfixe fixe + `subject` + suffixe fixe ; destinataire dans un `box` calé à droite, texte hors données (« P.J. : Curriculum vitae ») |

Les tests de `recipient_check.rs` compilent aussi `lm_balisee.typ` avec son bloc destinataire remplacé par un `block(width: 7cm)`, pour vérifier la détection du bloc de largeur fixe. Les références demandent la police `Arimo`, absente des machines et des polices embarquées de Typst : une famille introuvable étant une erreur de compilation, les tests la fournissent comme une police importée, en copiant `fonts/arimo_reference/` dans le dossier de polices propre au test (`shared/font_fixtures.rs::install_reference_font`, appelé par `test_state::app` et `recipient_check.rs`). Avec elle, le CV et la lettre tiennent chacun sur 1 page.

## `description_list_cases.json` : découpage des descriptions

Table de cas écrite à la main (aucune donnée réelle) : pour chaque description saisie (`input`), l'intro et les éléments attendus. Elle est lue à la fois par le parseur Rust (`src/docgen/description_list.rs`) et par son miroir TS (`src/composables/parseDescriptionList.test.ts`), pour que les deux appliquent exactement la même règle. Un cas ajouté ici est testé des deux côtés.

## `fonts/` : polices de fixture

Sous-ensembles générés avec fontTools (`subset`, `varLib.instancer`) à partir de polices sous licence SIL OFL 1.1 ; chaque dossier garde son `OFL.txt`. Basic Latin (U+0020–U+007E) sauf `arimo_reference/`. Copiées dans le dossier propre au test via `shared/font_fixtures.rs`.

| Dossier | Contenu | Utilisé pour |
|---|---|---|
| `arimo_reference/Arimo-{Regular,Bold,Italic,BoldItalic}.ttf` | Arimo 4 styles (TTF statiques de Google Fonts, `static/`), environ 17 à 19 Ko chacun | Police des références balisées (`references/tagged/`) : installée dans le dossier de polices de chaque app de test ; `typst_fonts.rs` vérifie que les deux références se rendent, en français et en anglais, sans avertissement ni glyphe de repli |
| `arimo/static/Arimo-Regular.ttf` | Instance statique 400 d'Arimo (police variable de Google Fonts), sous un sous-dossier comme dans une archive | Police importée utilisée sans avertissement et présente dans le PDF ; import récursif, nom de famille « Arimo » |
| `collection/Arimo.ttc` | Collection : Arimo Regular et Italic (instances 400) | Import d'une collection `.ttc` |
| `libertinus/LibertinusSerif-Regular.ttf` | Libertinus Serif Regular, homonyme d'une police embarquée par Typst | Avertissement d'homonyme ; dossier `Libertinus_Serif` vs famille `Libertinus Serif` |
| `web/Exemple-Regular.woff2` | Fichier texte factice : seule l'extension compte | Refus des formats WOFF/WOFF2 |

`arimo_reference/` couvre Latin-1 (U+0020–U+00FF), `Œ œ Ÿ` (U+0152–U+0153, U+0178), `– —` (U+2013–U+2014), `‘ ’ “ ”` (U+2018–U+2019, U+201C–U+201D), `•` (U+2022), `…` (U+2026), l'espace fine insécable U+202F et `€` (U+20AC). La liste vient d'un relevé des caractères non ASCII des fixtures rendues en Arimo (`references/tagged/`, `llm/`, profil de `test_state.rs`), complété des signes typographiques français courants ; les emojis de `llm/scraped_offer.json` ne sont jamais rendus. Si une fixture gagne un caractère hors de cette liste, `typst_fonts::references_balisees_entierement_rendues_en_arimo` échoue : regénérer le sous-ensemble depuis les TTF statiques d'origine (Google Fonts, archive Arimo, dossier `static/`), avec `OFL.txt` (Copyright 2026 The Arimo Project Authors) :

```bash
for s in Regular Bold Italic BoldItalic; do
  pyftsubset "Arimo/static/Arimo-$s.ttf"     --unicodes="U+0020-00FF,U+0152-0153,U+0178,U+2013-2014,U+2018-2019,U+201C-201D,U+2022,U+2026,U+202F,U+20AC"     --no-hinting --output-file="arimo_reference/Arimo-$s.ttf"
done
```

## `llm/` : réponses figées du modèle

Une réponse par étape, renvoyée par `tests/integration/shared/fake_llm.rs` (l'étape est reconnue d'après le schéma de sortie demandé ; `Reply::Sequence` enchaîne plusieurs fichiers pour une relance réussie) :

| Fichier | Étape | Particularité |
|---|---|---|
| `offer_analysis.json` | Analyse de l'offre | Langue `fr` (pas de traduction du profil) |
| `offer_analysis_en.json` | Analyse de l'offre | Langue `en` : variante anglaise des références et traduction du profil |
| `profile_translation_structure.json` | Traduction du profil | Expérience à intro + élément avec une suite de ligne + élément (4 lignes à plat), pour vérifier la reconstruction de la structure |
| `matching.json` | Matching | Projet 1, celui de `test_state::seed_profile` |
| `matching_id_inconnu.json` | Matching | Projet 99, absent du profil, pour déclencher les relances puis l'échec |
| `cv_content.json` | Rédaction du CV | Titre, résumé et compétences seulement ; un tiret cadratin volontaire, que `strip_dashes` doit retirer |
| `cv_content_vide.json` | Rédaction du CV | Champs de prose vides, pour déclencher les relances puis l'échec |
| `profile_translation.json` | Traduction du profil | Expérience 1, projet 1 et formation 1 de `test_state::seed_profile`, une ligne chacun |
| `profile_translation_lignes.json` | Traduction du profil | Deux lignes pour une description d'une ligne, pour déclencher les relances puis l'échec |
| `lm_content.json` | Rédaction de la lettre | Un tiret cadratin volontaire |
| `lm_content_double_tiret.json` | Rédaction de la lettre | « --- » dans l'objet et « -- » dans un paragraphe (tirets ASCII), que la normalisation doit remplacer |
| `lm_content_recopie_contexte.json` | Rédaction de la lettre | Le paragraphe 2 recopie plus de 8 mots du contexte détaillé de l'expérience 1 posé par `generation_detailed_context.rs`, pour déclencher la relance (puis l'avertissement si elle persiste) |
| `lm_content_contexte_reformule.json` | Rédaction de la lettre | Même lettre, fait du contexte reformulé et sans tiret : essai corrigé après la recopie |
| `lm_prompt_sans_contexte.txt` | Rédaction de la lettre (message envoyé) | Message utilisateur de la lettre capturé sans contexte détaillé, avec le profil de `test_state::seed_profile` : sans contexte, le prompt doit lui rester identique. À régénérer si le prompt de la lettre change volontairement (`FINDMYJOB_UPDATE_SNAPSHOTS=1 cargo test`) |
| `scraped_offer.json` | Extraction d'offre (étape LLM) | Balises HTML, emojis, contrat en majuscules et `mailto:`, que `finalize_fields` doit nettoyer |
