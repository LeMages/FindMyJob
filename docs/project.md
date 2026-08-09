# FindMyJob, document de contexte projet

Ce document rassemble toutes les décisions de cadrage du projet FindMyJob. Il sert à reprendre une conversation avec une IA sans avoir à tout réexpliquer. Il décrit le quoi, le pourquoi, et l'état d'avancement réel du code.

Dernière mise à jour : 9 août 2026, contre l'état du code sur `fix/generation_document` (dernier commit `43c6a6c`).

## 1. Vision

FindMyJob est un logiciel desktop open source qui aide les étudiants à suivre leurs candidatures (stage, alternance, CDI). L'objectif n'est pas un simple gestionnaire CRUD, mais un outil augmenté par l'IA qui supprime les tâches pénibles de la recherche (saisie des offres, relances, adaptation du CV).

Public cible : étudiants en recherche de stage, marché français en priorité. Le projet est né d'un besoin personnel (l'auteur est étudiant en ingénierie informatique) et passé en open source pour aider les gens dans la même situation.

Principe directeur de conception : penser le périmètre et le schéma le plus complètement possible en amont pour éviter les migrations. Avancer gate par gate, valider chaque fondation avant de construire dessus.

## 2. Stack technique

| Couche | Technologie |
|--------|-------------|
| Application desktop | Tauri 2 (Windows / macOS / Linux) |
| Frontend | Vue 3.5 + TypeScript 5 + Vite 6, Tailwind CSS 4 via `@tailwindcss/vite` (pas de fichier `tailwind.config`) |
| Backend | Rust, édition 2021 |
| Base de données | SQLite locale, via sqlx 0.8 (`sqlite`, `runtime-tokio-rustls`, `migrate`) |
| Compilation de documents | `typst-as-lib` 0.16 (`typst-kit-fonts`, `typst-kit-embed-fonts`) + `typst-pdf` 0.15 |
| IA | API Messages d'Anthropic (Claude, modèle `claude-sonnet-5`), appels HTTP directs via `reqwest` 0.12 (rustls) |

Autres dépendances notables : `tauri-plugin-dialog` (sélecteur de fichiers natif), `dotenvy` (chargement du `.env`), `base64` (échange de PDF en base64 sur l'IPC), `open` (ouverture de dossiers/URL dans l'explorateur système).

Pas de vue-router : la navigation est un routeur maison par `state.screen` (`src/stores/app.ts` + `src/App.vue`). Pas de Pinia : le store est un simple objet `reactive` exporté (`src/stores/app.ts`).

## 3. Architecture du code

### Backend (`src-tauri/src/`)

```
commands/    couche IPC — un fichier par domaine, 15 fichiers, #[tauri::command] uniquement
db/          accès SQLite — un fichier par table + common.rs (utilitaires), profile.rs
             (agrégat profil complet), timeline_events.rs (règles de timeline)
models/      DTO sérialisables, miroir des lignes SQL / réponses IPC
documents/   stockage disque + compilation Typst (storage.rs, typst.rs)
llm/         intégration API Claude (client, pipeline, prompts par étape)
error.rs     AppError, type d'erreur commun sérialisé vers le frontend
```

`commands/` est fin : chaque commande extrait l'utilisateur courant, délègue à `db::` ou `llm::`, et retourne un DTO ou une `AppError`. La logique métier (transitions de statut, contraintes) vit dans `db/`, jamais dans `commands/`.

### Frontend (`src/`)

```
views/<nom_vue>/XxxView.vue      une vue = un dossier, composants propres dans components/
views/<nom_vue>/components/      composants spécifiques à cette vue
components/                      composants partagés entre plusieurs vues
composables/                     logique réutilisable côté vue (formatage, listes éditables)
stores/app.ts                    état applicatif global (reactive + fonctions exportées)
types/index.ts                   DTOs miroir des structs Rust, écrits à la main (voir section 4)
```

### Commandes IPC enregistrées

55 commandes sont enregistrées dans `generate_handler!` (`src-tauri/src/lib.rs:35-91`), regroupées par domaine :

| Domaine | Commandes |
|---|---|
| Système | `db_status` |
| Utilisateur / onboarding | `user_exists`, `create_user`, `get_user`, `update_user_info`, `open_url` |
| Profil | `get_profile`, `update_description` |
| Expériences / formations / langues / projets / skills | `add_*`, `delete_*`, `update_*` (5 tables × 3 verbes) |
| Documents de référence (session) | `get_cv_reference`, `get_lettre_motivation_reference`, `save_cv`, `save_lettre_motivation`, `compile_cv`, `compile_lettre_motivation`, `get_cv_reference_dir`, `get_lettre_motivation_reference_dir` |
| Documents joints (candidature) | `upload_candidature_cv`, `upload_candidature_lettre_motivation`, `select_candidature_cv`, `select_candidature_lettre_motivation`, `get_document_info`, `get_document_pdf`, `open_document` |
| Candidatures | `create_candidature`, `get_candidatures`, `get_candidature`, `update_candidature`, `abandon_candidature`, `postuler_sans_message`, `postuler_par_mail`, `postuler_par_message` |
| Timeline / suivi | `get_status_history`, `get_available_events`, `add_timeline_event`, `get_pending_interviews`, `get_contacts` |
| Sessions | `create_session`, `get_sessions`, `update_session`, `delete_session` |

## 4. Frontière front / back (thin client)

Principe directeur : **thin client**. Vue est une couche de rendu et d'interaction, Rust est le cerveau. Aucune vérité métier n'est décidée côté Vue. On s'appuie au maximum sur Rust ; utiliser la réactivité de Vue n'est pas un contournement de cette règle (voir ligne de partage).

### Ligne de partage : état du monde vs état de la vue

- **État du monde (Rust)** : tout ce qui est *vrai* sur les données (candidatures, profil, règles de relance, validité des transitions de statut, stats). C'est le modèle métier, il vit en Rust et nulle part ailleurs.
- **État de la vue (Vue)** : ce qui est à l'écran à l'instant T (onglet actif, modale ouverte, contenu d'un formulaire non encore soumis, filtre sélectionné). Reste en Vue. La réactivité Vue (`ref`, `computed`, `watch`) gère l'état de la vue : ce n'est pas de la logique métier et c'est autorisé.

### Règle de pont

**Tout ce que Rust calcule, il l'envoie déjà calculé dans le DTO. Vue affiche, ne recalcule jamais une valeur métier.**

- Les commandes Tauri renvoient des DTO prêts à afficher, avec les champs dérivés déjà dedans, pas des lignes SQL brutes.
- Si une règle dépend d'un setting, le calcul vit en Rust pour rester à un seul endroit.

Cette règle n'est aujourd'hui **pas totalement respectée** : voir section 11 (dette technique) pour les calculs métier repassés côté Vue dans `TrackingView.vue`.

### Contrat de commande (fail fast)

- Chaque commande renvoie `Result<Dto, AppError>`, `AppError` étant un enum sérialisé en `{ "kind": "Validation" | "Conflict" | "NotFound" | "Config" | "Llm" | "Database", "message": "..." }` (`src-tauri/src/error.rs`).
- Vue reçoit une erreur typée et l'affiche via `extractErrorMessage` (`src/composables/useErrorMessage.ts`). Pas de `try/catch` qui avale, pas de fallback silencieux sur valeur vide.

### Exemple de référence : ajout d'un événement de timeline

L'exemple historique était le drag & drop du kanban ; le suivi n'étant plus un kanban (voir section 6), l'exemple de référence est désormais l'ajout d'un événement de timeline :

1. Le frontend appelle `get_available_events(candidature_id)`, qui renvoie la liste des événements légaux pour le statut courant (`db::timeline_events::available_events`, seule source de vérité).
2. L'utilisateur choisit un événement dans `AddEventPopup.vue`, remplit les champs requis selon son type.
3. Le frontend appelle `add_timeline_event(candidature_id, event)`. Rust **revalide** que l'événement est autorisé pour le statut courant (ne fait jamais confiance à ce que le frontend envoie), applique ses effets (création/mise à jour d'un entretien, changement de statut) et écrit `status_history`, le tout dans une seule transaction (`db/timeline_events.rs:110-224`).
4. Si Rust refuse (`AppError::Conflict`), le frontend affiche l'erreur sans avoir modifié son état local.

Vue propose, Rust dispose : Vue ne connaît pas les transitions légales, elle les redemande à chaque fois via `get_available_events`.

### Synchronisation du contrat (types) — décision toujours ouverte

Source de vérité voulue du contrat IPC = les structs Rust. En pratique, `src/types/index.ts` est **toujours écrit à la main**, en miroir manuel des DTOs Rust, sans génération automatique. La décision reste ouverte :
- `tauri-specta` v2 : génère types ET wrappers de commandes typés, une signature Rust qui change casse la compile du front. Toujours en RC, à épingler strictement avec `=` dans `Cargo.toml`.
- `ts-rs` : génère seulement les types TS, plus stable, mais les `invoke` restent manuels.

## 5. Modèle de données

### Emplacement

La base ne doit jamais être à côté de l'exécutable. Logique conditionnelle (`src-tauri/src/db/init.rs:39-84`) :
- Build debug : `./dev-data/findmyjob.db` (relatif au projet, pour inspection facile en dev). `dev-data/` est gitignore.
- Build release : `app.path().app_data_dir()` de Tauri.

### Initialisation au démarrage (`init_db`, `db/init.rs:99-135`)

L'initialisation ouvre **deux pools successifs** sur le même fichier :
1. Un pool de migration avec `foreign_keys(false)` : plusieurs migrations recréent une table parente pour changer un `CHECK` ou une contrainte `NOT NULL` (seule façon de faire en SQLite), ce qui échouerait avec `FOREIGN KEY constraint failed` si les clés étrangères étaient actives pendant le `DROP TABLE`. `sqlx::migrate!("../db/migrations")` s'exécute sur ce pool, puis il est fermé.
2. Le pool applicatif, avec `foreign_keys(true)`, utilisé pour toute la durée de vie de l'app.

Les deux pools sont en mode WAL (`SqliteJournalMode::Wal`). Les dossiers de documents (`documents/sessions/<session_id>/...`) sont créés à la volée par `documents::storage`, pas à l'initialisation (les sessions n'existent pas encore à ce stade).

### Arborescence des données sur disque

```
<app_data_dir ou ./dev-data>/
  findmyjob.db
  documents/
    sessions/
      <session_id>/
        cv/
          reference.typ, reference.pdf
          backup/<timestamp>/cv.typ, cv.pdf
          artefact/<candidature_id>.typ, <candidature_id>.pdf, pending_<ts>_<filename>
        lettre_motivation/   (même structure que cv/)
```

### Schéma

La source de vérité du schéma est les **migrations sqlx** dans `db/migrations/` (**15 migrations**, `0001` à `0015`), appliquées au boot par `sqlx::migrate!("../db/migrations")`. `db/schema.dbml` et `db/schema.png` existent encore mais décrivent une version antérieure du schéma, désynchronisée — ne pas s'y fier.

Tables réelles : `users`, `lang`, `formations`, `projets`, `experiences`, `skills`, `sessions`, `candidatures`, `status_history`, `contacts`, `interviews`, `documents`, `emails`, `embeddings`, `settings`.

Points de modélisation importants :
- **Entreprise dénormalisée** : les tables `companies` et `company_locations` ont été abandonnées dans le schéma final. La candidature porte directement `entreprise` (texte) et `localisation` (texte).
- **users** : porte `permis` (boolean), `description` (texte libre), identité et liens (LinkedIn, GitHub, portfolio).
- **lang** : langues parlées par l'utilisateur (`nom`, `level`), liée à `users`.
- **formations** : diplômes/écoles du profil (`titre`, `ecole`, dates), liée à `users` (migration `0003`).
- **sessions** : session de recherche de candidatures (`nom`, `contexte`, `statut`, `zone`), liée à `users` (migration `0008`, enrichie en `0010`). Regroupe un ensemble de candidatures et possède ses propres CV/LM de référence.
- **documents** : refondue en `0009`/`0011`. Liée à `sessions` (`session_id`), pas aux candidatures ni aux users. Colonnes : `doc_type` (`cv`/`lettre_motivation`), `file_path`, `pdf_path`, `style_analysis` (cache de l'analyse de style LLM, calculé à la sauvegarde de la référence), `UNIQUE(session_id, doc_type)`. Le rôle « document rattaché à une candidature » est porté par `candidatures.cv_path`/`lettre_motivation_path`, pas par cette table.
- **candidatures** : porte `entreprise`, `localisation`, `remuneration`, `url_offre`, `description_offre` (texte brut de l'annonce), `type` (incluant `interim` depuis `0005`), `status` (10 valeurs, voir section 6), `source`, `session_id`, `cv_path`/`lettre_motivation_path`, `date_candidature`, `next_relance`, `last_relance`, `notes`. Recréée deux fois (`0014`, `0015`) pour étendre le `CHECK(status)`.
- **status_history** : devenue **timeline unifiée** en migration `0014` — chaque ligne est un événement typé (`event_type` NOT NULL), qu'il change le statut ou non. `nouveau_status` est désormais **nullable** (un événement comme « appel » ou « note » n'en écrit aucun). Porte aussi `content` (message associé, migration `0013`) et `interview_id` (lien optionnel vers un entretien, migration `0014`).
- **interviews** : porte `contact_id` (migration `0014`, lien optionnel vers le contact rencontré).
- **skills** : liste à plat, une ligne par skill avec un champ `domaine` texte. Pas de table de liaison projet-skill.
- **contacts** : interlocuteurs associés à une candidature (RH, manager…).
- **emails** : tous les mails envoyés depuis l'app. La table de liaison `email_attachments` a été supprimée (migrations `0009`/`0011`).
- **embeddings** et **settings** : présentes dans le schéma mais **mortes** — ni écrites ni lues par le code actuel (voir section 11).

## 6. Cycle de vie d'une candidature

### Statuts (10 valeurs, `candidatures.status`)

`a_postuler`, `postule`, `relance`, `entretien`, `offre`, `refus`, `abandon`, `ghosting`, `accepte`, `offre_refusee`.

`offre` est un statut **intermédiaire** depuis la migration `0015` : recevoir une offre n'est pas terminal, elle peut encore être acceptée (`accepte`) ou déclinée (`offre_refusee`). Les statuts terminaux négatifs (`abandon`, `refus`, `ghosting`, `offre_refusee`) envoient la candidature en corbeille côté frontend (`TRASHED_STATUSES`, `src/composables/candidature_display.ts:20`) ; `accepte` est terminal mais reste affiché car c'est une issue positive.

### Types d'événements de timeline (15 valeurs, `models/event_type.rs`)

`creation`, `candidature_envoyee`, `relance`, `entretien_planifie`, `entretien_passe`, `offre_recue`, `refus_recu`, `ghosting`, `desistement`, `accepter_offre`, `decliner_offre`, `message_envoye`, `message_recu`, `appel`, `note`.

Chacun a un effet optionnel sur le statut (`EventType::changes_status`) : par exemple `entretien_planifie` → `entretien`, `accepter_offre` → `accepte`, tandis que `message_envoye`, `appel` ou `note` ne changent jamais le statut. `creation` et `candidature_envoyee` ne sont jamais proposés par `get_available_events` : ils sont gérés par les flux existants (création de la candidature, flux « Postuler »).

### Événements proposables selon le statut courant

`db::timeline_events::available_events` (`src-tauri/src/db/timeline_events.rs:35-79`) est la seule source de vérité. Une candidature dans un statut non terminal (`STATUTS_NON_TERMINAUX` = `a_postuler`, `postule`, `relance`, `entretien`, `offre`) peut toujours recevoir `message_envoye`, `message_recu`, `appel`, `note`, `desistement` ; les autres événements (`relance`, `entretien_planifie`, `entretien_passe`, `offre_recue`, `refus_recu`, `ghosting`, `accepter_offre`, `decliner_offre`) ne sont proposés que pour un sous-ensemble précis de statuts.

### Flux « Postuler »

Trois commandes couvrent le passage de `a_postuler` à `postule` (`commands/candidatures.rs:174-255`), toutes en trois temps : récupérer le détail, exporter les documents, marquer postulée.

- `postuler_sans_message` : exporte CV + lettre vers `~/Downloads/<poste>_<entreprise>_<id>/` (noms de fichiers assainis, `sanitize_filename`).
- `postuler_par_mail` : fait rédiger un mail par l'IA (`llm::generate_application_email`), l'exporte en plus (`Mail_de_candidature.txt`).
- `postuler_par_message` : fait rédiger un message court par l'IA (`llm::generate_application_message`), l'exporte en plus (`Message_candidature.txt`).

Les trois appellent ensuite `candidatures::mark_postule`, qui refuse (`AppError::Conflict`) si la candidature n'est plus au statut `a_postuler`, et enregistre l'événement `candidature_envoyee` avec le contenu du message le cas échéant.

### Canal de candidature (`candidatures.source`)

`SOURCE_OPTIONS` (`src/composables/candidature_display.ts:47-55`) propose 7 valeurs : Plateforme en ligne, Site carrière de l'entreprise, Mail, Cooptation / Réseau, Forum / Salon emploi, École, Autre. Contrairement à la décision d'origine (stocker un code normalisé), c'est aujourd'hui le **label complet** qui est stocké en base — pas de valeur normalisée séparée du libellé affiché.

## 7. Documents et compilation Typst

Chaque session de candidature a son propre CV et sa propre lettre de motivation de référence, stockés sous `documents/sessions/<session_id>/{cv,lettre_motivation}/` (voir arborescence section 5). Un seul document de référence par type et par session (`UNIQUE(session_id, doc_type)`).

- **Sauvegarde d'une référence** (`documents::storage::save_reference`) : analyse de style via Claude en premier (bloquant, avant toute écriture disque — un échec ne crée rien), puis sauvegarde de l'ancienne référence dans `backup/<timestamp>/`, écriture du nouveau `.typ`, compilation en `.pdf`, puis trace best-effort en DB (le filesystem reste la source de vérité ; un échec d'écriture DB est loggué mais ne fait pas échouer la sauvegarde).
- **Compilation** : `typst-as-lib` en `spawn_blocking` (opération CPU-bound), résolution des imports/images relatifs par rapport au dossier du document.
- **Progression** : chaque étape de `save_reference` émet un événement Tauri `document-save-progress` (5 étapes), écouté par `DocumentReferenceEditor.vue` pour afficher une barre de progression filtrée sur `session_id` + `doc_type`.
- **Documents joints à une candidature** : mis en scène (`upload_candidature_*` / `select_candidature_*`) sous `artefact/pending_<timestamp>_<filename>`, puis finalisés (`finalize_candidature_document`) sous `artefact/<candidature_id>.pdf` une fois la candidature créée. Les documents générés par le pipeline IA suivent le même schéma de nommage (`save_generated_document`), avec le `.typ` conservé à côté du `.pdf` pour la traçabilité/debug.

## 8. Couche IA

### Client HTTP (`src-tauri/src/llm/client.rs`)

Appels directs à l'API Messages d'Anthropic (`https://api.anthropic.com/v1/messages`, `anthropic-version: 2023-06-01`), modèle `claude-sonnet-5` en constante, en HTTP brut via `reqwest`. Pas de SDK officiel Rust pour l'API Anthropic. La clé `ANTHROPIC_API_KEY` est lue à la volée depuis l'environnement à chaque appel.

- `thinking: { type: "disabled" }` sur chaque requête : le pipeline est une génération structurée en un seul tour, le thinking ne sert à rien ici et partage le même budget de tokens que la sortie visible — l'avoir activé par défaut tronquait les réponses avant même le contenu utile.
- `output_config.format` (structured outputs / JSON Schema) activé dès qu'un schéma est fourni, pour garantir une réponse JSON strictement conforme (analyse d'offre, matching, analyse de style, contenu CV/LM).
- `MAX_TOKENS = 8192` par défaut (réponses JSON courtes) ; un plafond explicite plus élevé est utilisé pour les étapes produisant du Typst complet (`ASSEMBLE_MAX_TOKENS = 8192` dans `llm/assemble.rs`, via `call_claude_with_tokens`).
- Deux cas d'échec explicites, plutôt qu'une erreur de parsing cryptique en aval : `stop_reason == "refusal"` (classifieur de sécurité) et `stop_reason == "max_tokens"` (réponse tronquée) renvoient tous deux une `AppError::Llm` claire.

### Pipeline de tailoring CV / lettre de motivation (`llm/pipeline.rs::generate_documents`)

Enchaîne 6 étapes pour produire un CV et une lettre adaptés à l'offre, avant toute persistance (rien n'est écrit tant que le pipeline n'a pas réussi de bout en bout — `commands/candidatures.rs:30-91`) :

0. **Analyse de style** — hors pipeline à proprement parler, calculée une fois à la sauvegarde du document de référence (`llm::analyze_style`, mise en cache dans `documents.style_analysis`).
1. **Analyse d'offre** (`llm::offer::analyze_offer`) — extrait titre, compétences requises/préférées, niveau d'expérience, mots-clés, langue de l'offre.
2. **Matching profil** (`llm::matching::match_profile`) — décide quelles expériences, projets et compétences mettre en avant, à partir de l'analyse d'offre et du profil complet.
3. **Rédaction du contenu** (`llm::content::generate_cv_content` / `generate_lm_content`) — séquentiel actuellement, voir dette technique (section 11).
4. **Assemblage Typst** (`llm::assemble::assemble_typst`) — CV et lettre en parallèle (`tokio::try_join!`), injecte les vraies coordonnées du candidat (`build_sender_block`) pour éviter les placeholders.
5. **Compilation PDF avec retry** — un seul retry par document : en cas d'échec de compilation, `assemble::fix_typst` tente une correction à partir des erreurs du compilateur et du document Typst de référence, puis recompile une fois.

### Rédaction de messages de candidature (`llm/candidature_message.rs`)

Deux fonctions (`generate_application_email`, `generate_application_message`), texte brut prêt à copier-coller, un seul appel Claude chacune, sans structured output. Règles communes : ne reformuler que ce qui existe dans le profil, pas de placeholders, pas de markdown, signature minimale (prénom + nom uniquement).

### Décisions hors MVP — non implémentées

- **Couche provider-agnostic** : idée initiale d'une couche unique compatible OpenAI avec base URL configurable (Ollama Cloud puis mode local llama.cpp en une URL de plus, sans réécriture). Pas implémentée aujourd'hui.
  - Pistes Ollama Cloud envisagées à l'époque : free tier suffisant pour un usage par à-coups (quotas en temps GPU, limites de session toutes les 5h, limites hebdo tous les 7 jours, modèles de niveau 1/2), mais pas de SLA (panne connue d'avril 2026) — d'où l'intérêt d'un mode local en vrai fallback. Chaque utilisateur utiliserait son propre compte/clé pour scaler sans clé centralisée.
  - Ne pas bundler de runtime IA dans l'installeur en V1 : approche cloud-first, installeur léger, sidecar llama.cpp éventuel plus tard pour un mode local zéro-setup.
- **Embeddings** : table `embeddings` présente dans le schéma mais ni écrite ni lue. Le matching profil↔offre se fait aujourd'hui **par appel LLM**, pas par similarité vectorielle. Piste repoussée : modèle bge-m3, cosinus en brute-force dans le Rust, vecteurs en BLOB dans SQLite.

## 9. Sécurité des secrets

**État actuel** : la clé `ANTHROPIC_API_KEY` vit dans un fichier `.env` à la racine du projet, gitignoré, chargé au démarrage via `dotenvy::dotenv().ok()` (`src-tauri/src/lib.rs:24`, silencieux si absent). `.env.example` documente la variable sans valeur.

**Cible** : quand la page Settings sera réellement branchée, la clé API ne sera **jamais** stockée en base ni dans la table `settings`. Elle vivra dans le **trousseau de l'OS** (Keychain macOS, Credential Manager Windows, Secret Service Linux), via la crate Rust `keyring` (ou un plugin Tauri équivalent).

## 10. Fonctionnalités

### Implémenté et branché de bout en bout

- Onboarding et profil complet : expériences, formations, langues, projets, skills (CRUD complet, table `users` + tables liées).
- Sessions de candidature : CRUD, paramètres de session (contexte, zone, statut), CV/lettre de motivation de référence par session (import `.typ`, compilation, analyse de style via Claude, historique de sauvegarde, aperçu PDF).
- Candidatures : création avec génération IA obligatoire (CV + lettre adaptés, pipeline complet, échec bloquant la création), liste en tableau par session, détail éditable tant que « à postuler » (règle `editable` dans `DetailView.vue`), timeline d'événements typés, gestion d'entretiens, corbeille pour les statuts terminaux négatifs.
- Flux « Postuler » : sans message, par mail (rédigé par l'IA), par message court (rédigé par l'IA) — export systématique des documents vers le dossier de téléchargements.
- Aperçu PDF des documents (référence et joints), via `get_document_pdf` / `compile_cv` / `compile_lettre_motivation` encodés en base64.
- CI/CD : lint + clippy + tests (voir section 12 pour le détail réel), Conventional Commits, flux de branches `feature/* → dev → staging → main`.

### Partiel / en maquette

- **Page Settings** (`SettingsView.vue`) : interface complète (IA, relances, apparence, signature, données/stockage) mais **aucune commande Tauri branchée** — tout est en `ref` local avec des valeurs en dur (clé API factice, chemin `~/Library/Application Support/...` macOS codé en dur, modèle affiché « llama3.2 », libellé « Clé API Ollama Cloud »).
- **Contacts recruteur** : lecture fonctionnelle (`get_contacts`), mais l'ajout depuis la page détail (`addRecruiter` dans `stores/app.ts`) pousse un contact factice **purement local**, jamais persisté — aucune commande `create_contact` n'existe côté backend.
- **Relances** : colonnes `next_relance`, `last_relance`, `date_candidature` présentes dans le schéma et lues dans tous les DTO, mais **jamais écrites** par le code actuel.

### Prévu / hors MVP

- Extraction d'offre : coller le texte d'une annonce, l'IA remplit la fiche.
- Relances intelligentes : détection des candidatures en attente et génération du mail de relance adapté au statut.
- Matching profil vers offre en tant que feature autonome (classement des projets par pertinence), distinct de l'étape interne du pipeline de tailoring.
- Recherche / scraping d'offres en ligne.
- Mode IA local (détection d'un Ollama déjà installé, puis sidecar llama.cpp).
- Synchronisation multi-appareils, mobile, intégrations mail/calendrier.

## 11. Dette technique et écarts connus

- **Diagnostic temporaire en cours dans le pipeline IA** : `llm/pipeline.rs:156-162` porte un bloc commenté `[DIAG-TEMP]` — la rédaction du contenu CV et lettre de motivation a été repassée en **séquentiel** au lieu de `tokio::try_join!`, pour tester une hypothèse de race de concurrence. À revert une fois le diagnostic conclu. La branche `fix/generation_document` porte aussi des commits de logging temporaire (`debug(assemble): log CV assembly input/output for truncation diagnosis`) liés au même sujet.
- **Code mort du drag & drop kanban** : le suivi des candidatures est aujourd'hui un tableau (`TrackingView.vue`), plus un kanban. Restent inutilisés : `startDrag`, `showDragPopup`, `confirmDrag`, `cancelDrag`, `state.dragState`, `state.dragPopup` (`src/stores/app.ts:374-396`), les types `DragState`/`DragPopupState` (`src/types/index.ts`), le composant `DragPopup.vue`, ainsi que `KANBAN_COLUMNS` et `columnTitleForStatus` (`composables/candidature_display.ts`) sans plus aucun appelant.
- **Règle thin-client partiellement contournée** : `TrackingView.vue` calcule côté Vue des valeurs qui devraient être des flags dérivés fournis par Rust — `inactivityDays`/`inactivityColor` (prédicat métier sur le nombre de jours d'inactivité), `sessionIsAccepted`, et le filtrage corbeille par `TRASHED_STATUSES`. Contraire à la règle de pont de la section 4.
- **Tables mortes** : `settings` et `embeddings` existent dans le schéma mais ne sont ni écrites ni lues par le code applicatif.
- **Relances jamais écrites** : `mark_postule` ne pose même pas `date_candidature` au passage à `postule` ; `next_relance`/`last_relance` ne sont mis à jour nulle part.
- **Formulaire de nouvelle candidature incomplet** : les champs de contact recruteur dans `NewApplicationView.vue` n'ont aucun `v-model`, donc ne sont jamais envoyés à la création.
- **Zéro test dans le dépôt** alors que `.github/workflows/ci.yml` lance `cargo test` sur chaque PR vers `dev` — le job passe systématiquement à vide — et que le README annonce « CI : lint + clippy + tests ».
- **Documentation désynchronisée en dehors de ce fichier** : `db/schema.dbml` et `db/schema.png` décrivent un schéma antérieur ; le README décrit encore la couche IA comme « compatible OpenAI → Ollama Cloud, modèle configurable » et le pipeline de statut sans les statuts terminaux (`ghosting`, `accepte`, `offre_refusee`) ni la timeline d'événements.

## 12. Dépôt, CI/CD et conventions de travail

### Dépôt GitHub

- Privé d'abord, public à la sortie de la V1.
- Public : lecture / clone / fork ouverts à tous, aucun push direct par des tiers. Les contributions passent par fork + pull request, mergées à la discrétion de l'auteur.
- Option à activer si besoin : branch protection sur main.
- Avant de passer public : vérifier qu'aucun secret n'a été commité dans l'historique. `.gitignore` propre, secrets hors repo, `.env.example` vide pour la doc.
- Description du repo : « Logiciel open source pour suivre ses candidatures de stage, d'alternance et de CDI, avec assistance IA. »
- Topics : tauri, vue, rust, sqlite, job-search, ai, open-source.

### Convention de commits

Conventional Commits pour automatiser le versioning : format `<type>(<scope optionnel>): <description>`.
- `feat:` → version mineure, `fix:` → patch, `feat!:`/`fix!:`/`BREAKING CHANGE` → version majeure.
- Autres types (pas de release) : `docs:`, `style:`, `refactor:`, `test:`, `chore:`.

Détail complet dans `docs/commit.md`. Conventions de pull request dans `docs/pull_request.md`.

### Flux de branches et CI (workflows réels)

- `feature/*` → PR vers `dev` — `.github/workflows/ci.yml` : job `frontend` (lint ESLint + `vue-tsc` via `npm run build`), job `rust` (`cargo clippy -- -D warnings` + `cargo test`, sur `ubuntu-22.04`).
- `dev` → PR vers `staging`, `staging` → PR vers `main` — `.github/workflows/gatekeeper.yml` bloque toute autre branche source (seule `staging` peut être mergée dans `main`, seule `dev` peut être mergée dans `staging`).
- Push sur `main` — `.github/workflows/release.yml` : `semantic-release` (config `.releaserc.json`, plugins commit-analyzer / release-notes-generator / exec / github) détermine la version, puis build Tauri matriciel (macOS ARM64 + x64, Ubuntu 22.04, Windows) et publication de release GitHub si une version a été calculée.

### Conventions de travail

- Avancer gate par gate, mesurer avant de décider.
- Prompts Claude Code chirurgicaux et scopés (« ne crée rien de plus que X »), pour garder le contrôle de l'architecture.
- Pas de refactor opportuniste.
- Préférer échouer vite plutôt qu'une dégradation silencieuse.
- Style de rédaction des livrables : pas de tirets cadratins.
