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

### `references/real/` : références réelles anonymisées et balisées

| Fichier | Source | Réduction |
|---|---|---|
| `cv_balise.typ` | CV de référence réel d'un utilisateur (2026-10-13) | **Anonymisé** : nom, e-mail, téléphone, LinkedIn, GitHub et site remplacés par des valeurs fictives de même forme (`Camille Exemple`, `camille@exemple.invalid`, `+33 6 00 00 00 00`…). Mise en page, parcours, ville et code postal inchangés. Balisé au format de `docs/reference-markup.md` : `[CONSIGNES]`, `[VARIABLE]` sur le titre, `[FIXE]` puis `[ADAPTABLE]` dans le résumé |
| `lm_balisee.typ` | Lettre de référence réelle du même utilisateur | Même anonymisation. Balisée au même format (consignes générales, zones dans l'objet et les paragraphes) ; lieu et date sur une ligne, nom en plusieurs morceaux, texte hors schéma ajouté (« P.J. : Curriculum vitae ») |

`*_example_data.json` sont les données d'exemple correctes (sortie de l'étape A, sans `texts`), `*_template.typ` le template correct (étape B), écrits à la main : les textes en dur sont lus dans `data.texts` (`t1`, `t2`…). `lm_balisee_expected_zones.json` est le découpage attendu des zones de la lettre. Les tests de `templatize_validation.rs` les passent dans `check_example` et `check_template` : si l'un d'eux est refusé, la validation est devenue trop stricte. La police `Arimo` des références n'est pas embarquée : sans elle, Typst prend une police de repli, la référence passe à 2 pages et le template doit rester en dessous.

## `llm/` : réponses figées du modèle

Une réponse par étape, renvoyée par `tests/integration/shared/fake_llm.rs` (l'étape est reconnue d'après le schéma de sortie demandé ; `Reply::Sequence` enchaîne plusieurs fichiers pour une relance réussie) :

| Fichier | Étape | Particularité |
|---|---|---|
| `offer_analysis.json` | Analyse de l'offre | Langue `fr` (pas de traduction des libellés ni du profil) |
| `offer_analysis_en.json` | Analyse de l'offre | Langue `en` : déclenche la traduction des libellés et du profil |
| `matching.json` | Matching | Projet 1, celui de `test_state::seed_profile` |
| `matching_id_inconnu.json` | Matching | Projet 99, absent du profil, pour déclencher les relances puis l'échec |
| `cv_content.json` | Rédaction du CV | Titre, résumé et compétences seulement ; un tiret cadratin volontaire, que `strip_dashes` doit retirer |
| `cv_content_vide.json` | Rédaction du CV | Champs de prose vides, pour déclencher les relances puis l'échec |
| `profile_translation.json` | Traduction du profil | Expérience 1, projet 1 et formation 1 de `test_state::seed_profile`, une ligne chacun |
| `profile_translation_lignes.json` | Traduction du profil | Deux lignes pour une description d'une ligne, pour déclencher les relances puis l'échec |
| `lm_content.json` | Rédaction de la lettre | Un tiret cadratin volontaire |
| `lm_content_double_tiret.json` | Rédaction de la lettre | « --- » dans l'objet et « -- » dans un paragraphe (tirets ASCII), que la normalisation doit remplacer |
| `cv_labels.json`, `lm_labels.json` | Traduction des libellés | Utilisées seulement si la langue de l'offre diffère de celle de la référence |
| `scraped_offer.json` | Extraction d'offre (étape LLM) | Balises HTML, emojis, contrat en majuscules et `mailto:`, que `finalize_fields` doit nettoyer |

Les réponses de templatisation par défaut sont construites à partir de `references/` : données d'exemple (étape A) et template entouré d'un bloc ```typst``` (étape B). `Reply::Files` renvoie des fichiers de `fixtures/` tels quels pour rejouer d'autres sorties :

| Fichier | Particularité |
|---|---|
| `llm/templatize/real_failures/*.typ` | Templates refusés **réels**, repris de `dev-data/…/templatize_debug` (anonymisés comme les références) ; le `.txt` voisin garde les problèmes relevés à l'époque. `cv_hash_en_mode_code` : `#` en mode code ; `cv_code_rendu_en_texte` : code écrit en markup ; `lm_valeur_ajoutee` : « 74300, » ajouté à la ligne de date ; `lm_methode_inexistante` : `.filter` sur une chaîne |
| `llm/templatize/lm_diese_en_mode_code.typ`, `lm_correction_ciblee.txt` | Un `#` en trop puis sa correction sous forme de remplacement ciblé |
| `llm/templatize/cv_separateur_orphelin.typ` | Template dont le seul défaut est un séparateur orphelin (non bloquant) |
| `llm/templatize/lm_exemple_ville_inventee.json` | Données d'exemple avec une ville absente de l'original |
