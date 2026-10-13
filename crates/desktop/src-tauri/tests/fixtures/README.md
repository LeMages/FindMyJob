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

`cv_template.typ` et `lm_template.typ` sont des templates écrits pour les tests. Ils lisent `data.json`, et une valeur vide fait disparaître sa ligne sans séparateur orphelin. `cv_example_data.json` et `lm_example_data.json` sont leurs données d'exemple fictives : toutes les valeurs contiennent « Exemple », ce qui permet de vérifier que la génération ne les recopie jamais.

La référence déposée par les tests (`test_state::reference_source`) est le template lui-même, avec ses données d'exemple écrites dans le document à la place de `json("data.json")`. Rendue, elle affiche exactement le texte attendu, si bien que la validation de la templatisation passe dès le premier essai.

`cv_template.typ` à la racine de `fixtures/` est un autre template, utilisé par les tests unitaires de `src/docgen/tests.rs`.

### `references/real/` : références réelles anonymisées

| Fichier | Source | Réduction |
|---|---|---|
| `cv_suisse.typ` | CV de référence réel d'un utilisateur (2026-10-13) | **Anonymisé** : nom, e-mail, téléphone, LinkedIn, GitHub et site remplacés par des valeurs fictives de même forme (`Camille Exemple`, `camille@exemple.invalid`, `+33 6 00 00 00 00`…). Mise en page, commentaires, parcours, ville et code postal inchangés |
| `lm_exemple.typ` | Lettre de référence réelle du même utilisateur, avec ses commentaires de consignes (`[FIXE]`, `[VARIABLE]`, `[ADAPTABLE]`) | Même anonymisation |

`cv_suisse_template.typ`, `lm_exemple_template.typ` et leurs `*_example_data.json` sont une templatisation correcte écrite à la main : rien n'est retiré de la référence (la ligne de disponibilité et « Références disponibles sur demande » restent en dur dans le template du CV), les lignes service et rue du destinataire sont dans `recipient.extra_lines`, et les consignes de la lettre sont dans `guidance`. Les tests de `src/documents/templatize_tests.rs` les passent dans `validate` (rendus exemple, vide, clairsemé, sentinelle, comparaison avec la référence) : si l'un d'eux est refusé, la validation est devenue trop stricte. La police `Arimo` des références n'est pas embarquée : sans elle, Typst prend une police de repli, la référence passe à 2 pages et le template doit rester en dessous.

## `llm/` : réponses figées du modèle

Une réponse par étape, renvoyée par `tests/integration/shared/fake_llm.rs` (l'étape est reconnue d'après le schéma de sortie demandé) :

| Fichier | Étape | Particularité |
|---|---|---|
| `offer_analysis.json` | Analyse de l'offre | Langue `fr` (pas de traduction des libellés) |
| `matching.json` | Matching | Expérience 1 et projet 1, ceux de `test_state::seed_profile` |
| `cv_content.json` | Rédaction du CV | Un tiret cadratin volontaire, que `strip_dashes` doit retirer |
| `cv_content_vide.json` | Rédaction du CV | Champs de prose vides, pour déclencher les relances puis l'échec |
| `lm_content.json` | Rédaction de la lettre | Un tiret cadratin volontaire |
| `cv_labels.json`, `lm_labels.json` | Traduction des libellés | Utilisées seulement si la langue de l'offre diffère de celle de la référence |
| `scraped_offer.json` | Extraction d'offre (étape LLM) | Balises HTML, emojis, contrat en majuscules et `mailto:`, que `finalize_fields` doit nettoyer |

Les réponses de templatisation sont construites à partir de `references/`.
