# FindMyJob, document de contexte projet

Ce document rassemble toutes les décisions de cadrage du projet FindMyJob. Il sert à reprendre une conversation avec une IA sans avoir à tout réexpliquer. Il décrit le quoi, le pourquoi, et l'état d'avancement.

## 1. Vision

FindMyJob est un logiciel desktop open source qui aide les étudiants à suivre leurs candidatures (stage, alternance, CDI). L'objectif n'est pas un simple gestionnaire CRUD, mais un outil augmenté par l'IA qui supprime les tâches pénibles de la recherche (saisie des offres, relances, adaptation du CV).

Public cible : étudiants en recherche de stage, marché français en priorité. Le projet est né d'un besoin personnel (l'auteur est étudiant en ingénierie informatique) et passé en open source pour aider les gens dans la même situation.

Principe directeur de conception : penser le périmètre et le schéma le plus complètement possible en amont pour éviter les migrations. Avancer gate par gate, valider chaque fondation avant de construire dessus.

## 2. Stack technique

| Couche | Technologie |
|--------|-------------|
| Application desktop | Tauri 2 (Windows / macOS / Linux) |
| Frontend | Vue 3 + TypeScript + Tailwind CSS (via Vite) |
| Backend | Rust |
| Base de données | SQLite locale, via sqlx (async, runtime tokio) |
| IA | API Messages d'Anthropic (Claude, modèle `claude-sonnet-5`), appels HTTP directs via `reqwest` |

## 3. Périmètre

### Dans le MVP

Cœur (suivi) :
- CRUD candidatures
- Pipeline de statut : à postuler, postulé, relancé, entretien, offre, refus
- Vues kanban et liste
- Rappels de relance

Features IA :
1. Tailoring CV / lettre de motivation : génération automatique d'un CV et d'une lettre adaptés à l'offre, à partir du profil complet et du texte de l'annonce (pipeline interne : analyse d'offre → matching profil → rédaction du contenu → assemblage Typst → compilation PDF).

### Hors MVP (évolutions)

- Extraction d'offre : coller le texte d'une annonce, l'IA remplit la fiche (entreprise, intitulé, type, lieu, stack, date limite, rémunération). Cas d'usage texte non structuré vers JSON structuré.
- Relances intelligentes : détection des candidatures en attente et génération du mail de relance adapté au statut.
- Matching profil vers offre en tant que feature autonome : pour une offre donnée, classer les projets de l'utilisateur par pertinence (aujourd'hui, le matching n'existe qu'en étape interne du pipeline de tailoring, via appel LLM).
- Recherche / scraping d'offres en ligne
- Mode IA local (d'abord détection d'un Ollama déjà installé, puis sidecar llama.cpp pour un vrai zéro-setup)
- Synchronisation multi-appareils
- Mobile
- Intégrations mail / calendrier

## 4. Décisions d'architecture IA

- **État actuel : appels directs à l'API Messages d'Anthropic** (`https://api.anthropic.com/v1/messages`, modèle `claude-sonnet-5`), en HTTP brut via `reqwest` (`src-tauri/src/llm/client.rs`). Pas de base URL configurable pour l'instant, pas de SDK officiel Rust pour l'API Anthropic. La logique prompt vers JSON vit directement dans le backend Rust (HTTP + serde), pas de Python ni LangChain ici (le packaging Python dans Tauri est trop lourd). Les structured outputs (`output_config.format` + JSON Schema) sont utilisés pour garantir des réponses JSON conformes (analyse d'offre, matching, analyse de style).
- **Hors MVP — couche provider-agnostic** : idée initiale d'une couche unique compatible OpenAI avec base URL configurable (Ollama Cloud puis mode local llama.cpp en une URL de plus, sans réécriture). Pas implémentée aujourd'hui ; reste un objectif si le provider doit devenir interchangeable.
  - Pistes Ollama Cloud envisagées à l'époque : free tier suffisant pour un usage par à-coups (quotas en temps GPU, limites de session toutes les 5h, limites hebdo tous les 7 jours, modèles de niveau 1/2), mais pas de SLA (panne connue d'avril 2026) — d'où l'intérêt d'un mode local en vrai fallback. Chaque utilisateur utiliserait son propre compte/clé pour scaler sans clé centralisée.
  - Ne pas bundler de runtime IA dans l'installeur en V1 : approche cloud-first, installeur léger, sidecar llama.cpp éventuel plus tard pour un mode local zéro-setup.
- **Embeddings : non implémentés.** La table `embeddings` existe dans le schéma (migration `0001`) mais n'est ni écrite ni lue par le code. Le matching profil↔offre se fait aujourd'hui **par appel LLM** (structured output, `src-tauri/src/llm/matching.rs`), pas par similarité vectorielle. Piste repoussée hors MVP : modèle bge-m3, cosinus en brute-force dans le Rust (pas de pgvector/extension, volumétrie trop faible pour le justifier), vecteurs stockés en BLOB dans SQLite, réutilisés à la fois pour le matching et le tailoring.

## 5. Sécurité des secrets

**État actuel** : la clé `ANTHROPIC_API_KEY` vit dans un fichier `.env` à la racine du projet, gitignoré, chargé au démarrage via `dotenvy::dotenv()` (`src-tauri/src/lib.rs`). `.env.example` documente la variable sans valeur, pour la doc. Aucune table ni colonne dédiée dans le schéma.

**Cible** : quand la page Settings globale sera construite, la clé API ne sera **jamais** stockée en base ni dans la table settings. Elle vivra dans le **trousseau de l'OS** (Keychain macOS, Credential Manager Windows, Secret Service Linux), via la crate Rust `keyring` (ou un plugin Tauri équivalent). Le backend la lira du trousseau au moment d'appeler l'API.

## 6. Base de données

### Emplacement

La base ne doit jamais être à côté de l'exécutable. Logique conditionnelle :
- Build debug : `./dev-data/findmyjob.db` (relatif au projet, pour inspection facile en dev). `dev-data/` est gitignore.
- Build release : `app.path().app_data_dir()` de Tauri (chemin système par OS, dérivé de l'identifier de tauri.conf.json).

### Création au démarrage (self-initializing)

L'installeur ne crée pas la base. C'est le code Rust qui le fait au premier lancement, de façon idempotente :
1. Résoudre le chemin selon debug / release.
2. Créer le dossier de données et les sous-dossiers `documents/references/` et `documents/generated/` s'ils n'existent pas.
3. Si le .db n'existe pas, le créer et appliquer le schéma (CREATE TABLE IF NOT EXISTS).
4. S'il existe, se connecter sans rien recréer.
5. Stocker le pool de connexion dans le state managé de Tauri.

Conventions : PRAGMA `foreign_keys = ON` et mode WAL. Enums DBML implémentés en colonnes TEXT + CHECK. `created_at` / `updated_at` sur chaque table, `updated_at` maintenu par trigger AFTER UPDATE.

### Arborescence des données

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

La source de vérité du schéma est les **migrations sqlx** dans `db/migrations/` (`0001` à `0012`), appliquées au boot par `sqlx::migrate!("../db/migrations")` (`src-tauri/src/db/init.rs`). `db/schema.dbml` existe encore mais décrit une version antérieure du schéma, désynchronisée — ne pas s'y fier. Tables réelles (état après migration `0012`) :

`users`, `lang`, `formations`, `projets`, `experiences`, `skills`, `sessions`, `candidatures`, `status_history`, `contacts`, `interviews`, `documents`, `emails`, `embeddings`, `settings`.

Points de modélisation importants :
- **Entreprise dénormalisée** : les tables `companies` et `company_locations` ont été abandonnées dans le schéma final. La candidature porte directement `entreprise` (texte) et `localisation` (texte). Plus simple pour le MVP ; on pourra normaliser plus tard si le besoin de regrouper plusieurs candidatures par entreprise apparaît.
- **users** : porte aussi `permis` (boolean) et `description` (texte libre), en plus de l'identité et des liens (LinkedIn, GitHub, portfolio).
- **lang** : langues parlées par l'utilisateur (`nom`, `level`), liée à `users`. Sert au profil et au tailoring.
- **formations** : diplômes/écoles du profil (`titre`, `ecole`, dates), liée à `users` (migration `0003`).
- **sessions** : session de recherche de candidatures (`nom`, `contexte`, `statut`, `zone`), liée à `users` (migration `0008`, enrichie en `0010`). Regroupe un ensemble de candidatures et possède ses propres CV/LM de référence.
- **documents** : refondue en `0009`/`0011`. Liée à `sessions` (`session_id`), pas aux candidatures ni aux users. Colonnes : `doc_type` (`cv`/`lettre_motivation`), `file_path`, `pdf_path`, `style_analysis` (cache de l'analyse de style LLM, calculé à la sauvegarde de la référence), `UNIQUE(session_id, doc_type)` — un seul document de référence par type et par session. Plus d'auto-référence ni de notion "maître/version taillée" : ce rôle est maintenant porté par `candidatures.cv_path`/`lettre_motivation_path`.
- **embeddings** : table polymorphe présente dans le schéma (`entity_type`, `entity_id`, `model`, `vector` en BLOB, index sur `(entity_type, entity_id)`) mais **non utilisée par le code actuel** — le matching profil↔offre passe par un appel LLM (voir section 4), pas par similarité vectorielle.
- **status_history** : log des changements de statut (ancien, nouveau, date) pour les stats temps moyen entre statuts.
- **emails** : tous les mails envoyés depuis l'app. La table de liaison `email_attachments` a été supprimée (migrations `0009`/`0011`), plus de table de documents partagée à joindre.
- **skills** : liste à plat, une ligne par skill avec un champ `domaine` texte ("Langages", "Data & ML"...). Pas de table de liaison projet-skill pour le MVP.
- **candidatures** : porte `entreprise`, `localisation`, `remuneration`, `url_offre`, `description_offre` (texte brut de l'annonce, sert à l'extraction IA), `type` (incluant `interim` depuis `0005`), `status`, `date_candidature`. Porte `next_relance` et `last_relance` en colonnes (pas de table reminders, le détail des relances vit dans `emails`). Porte aussi `source` (canal de candidature, voir section 8), `session_id` (migration `0008`), et `cv_path`/`lettre_motivation_path` (migration `0009`, chemins directs vers les documents finalisés sur disque). Note : pas de champ `date_limite` dans le schéma final.

## 7. Paramètres (table settings, clé-valeur)

Clés prévues : `ia_model`, `ia_mode`, `relance_delay_days`, `theme`. La clé API n'est PAS ici (trousseau OS).

Découpage de la page Settings :
- **IA** : clé API (champ masqué, stockée au trousseau), modèle (dropdown, `ia_model`), mode IA (cloud / local, `ia_mode`, en MVP cloud seulement, local grisé "bientôt"), bouton optionnel "Tester la connexion".
- **Relances** : délai avant relance suggérée (`relance_delay_days`), alimente `next_relance`.
- **Apparence** : thème clair / sombre / système (`theme`).
- **Données / Stockage** : afficher le chemin de la base et du dossier documents (lecture seule) + bouton "Ouvrir le dossier". Sauvegarde/export et réinitialisation possibles plus tard.
- **Profil** : infos de la table users (nom, prénom, email, téléphone, ville, LinkedIn, GitHub, portfolio). Décision ouverte : section dans Settings ou page dédiée. Ces infos servent au tailoring IA et aux mails.

## 8. Canal de candidature (champ candidatures.source)

Dropdown par canal (pas un annuaire de sites, pour éviter la maintenance). Valeurs envisagées :
Mail, Plateforme en ligne, Site carrière de l'entreprise, Cooptation / Réseau, École (job board EFREI), Forum / Salon (optionnel), Autre.

Version courte possible : Mail, Plateforme en ligne, Site entreprise, Cooptation, Autre.

"Autre" toujours en dernier, idéalement avec un champ texte libre conditionnel. Stocker une valeur normalisée (code) en base, pas le label affiché, pour des stats propres.

## 9. Dépôt GitHub

- Privé d'abord, public à la sortie de la V1.
- Public : lecture / clone / fork ouverts à tous, aucun push direct par des tiers. Les contributions passent par fork + pull request, mergées à la discrétion de l'auteur.
- Option à activer si besoin : branch protection sur main.
- Avant de passer public : vérifier qu'aucun secret n'a été commité dans l'historique. `.gitignore` propre, secrets hors repo, `.env.example` vide pour la doc.
- Description du repo : "Logiciel open source pour suivre ses candidatures de stage, d'alternance et de CDI, avec assistance IA."
- Topics : tauri, vue, rust, sqlite, job-search, ai, open-source.

### Convention de commits

Le projet utilise les **Conventional Commits** pour automatiser le versioning : format `<type>(<scope optionnel>): <description>`.
- `feat:` → version mineure, `fix:` → patch, `feat!:`/`fix!:`/`BREAKING CHANGE` → version majeure.
- Autres types (pas de release) : `docs:`, `style:`, `refactor:`, `test:`, `chore:`.

### Flux de branches

- `feature/*` → PR vers `dev` (CI : lint + clippy + tests).
- `dev` → PR vers `staging` (pré-production, tests manuels).
- `staging` → PR vers `main` (déclenche la release automatique).

## 10. État d'avancement

Fait :
- Fondation Tauri 2 + Vue 3 + TS + Tailwind, périmètre et stack figés.
- README.md rédigé et tenu à jour.
- Couche base de données : initialisation idempotente au boot (emplacement debug `./dev-data` vs release `app_data_dir`), migrations sqlx (`db/migrations/`, 12 migrations à date), commande `db_status` de vérification.
- Profil utilisateur complet : onboarding, CRUD expériences, formations, langues, projets, skills (table `users` + tables liées).
- Sessions de candidature : CRUD, page de paramètres de session (contexte, zone, statut), CV/lettre de motivation de référence par session (import `.typ`, compilation Typst → PDF, analyse de style via Claude, historique de sauvegarde).
- Candidatures : CRUD, vue pipeline/kanban, page de détail (édition des champs, aperçu des documents, historique de statut), drag & drop de statut, upload/sélection de CV et lettre de motivation par candidature.
- Assistance IA — tailoring CV/lettre de motivation : pipeline complet (analyse d'offre → matching profil → rédaction du contenu → assemblage Typst → compilation PDF avec retry de correction), génère un CV et une lettre adaptés à l'offre pour une candidature.
- CI/CD : lint + clippy + tests, Conventional Commits, flux de branches `feature/* → dev → staging → main`.

En cours / prochaine étape immédiate :
- Page Settings **globale** (hors paramètres de session) : clé API (aujourd'hui en `.env`, à terme au trousseau OS), modèle, thème, délai de relance.

Ensuite :
- Features IA hors MVP : extraction d'offre (remplissage auto de la fiche), relances intelligentes, matching profil↔offre en tant que feature autonome (classement des projets par pertinence).

## 11. Conventions de travail

- Avancer gate par gate, mesurer avant de décider.
- Prompts Claude Code chirurgicaux et scopés ("ne crée rien de plus que X"), pour garder le contrôle de l'architecture.
- Pas de refactor opportuniste.
- Préférer échouer vite plutôt qu'une dégradation silencieuse.
- Style de rédaction des livrables : pas de tirets cadratins.

## 12. Architecture front / back (frontière Rust / Vue)

Principe directeur : **thin client**. Vue est une couche de rendu et d'interaction, Rust est le cerveau. Aucune vérité métier n'est décidée côté Vue. On s'appuie au maximum sur Rust ; utiliser la réactivité de Vue n'est pas un contournement de cette règle (voir ligne de partage).

### Ligne de partage : état du monde vs état de la vue

- **État du monde (Rust)** : tout ce qui est *vrai* sur les données (candidatures, profil, règles de relance, validité des transitions de statut, embeddings, stats). C'est le modèle métier, il vit en Rust et nulle part ailleurs.
- **État de la vue (Vue)** : ce qui est à l'écran à l'instant T (onglet actif, modale ouverte, contenu d'un formulaire non encore soumis, filtre sélectionné, état visuel d'un drag-and-drop). Reste en Vue. La réactivité Vue (`ref`, `computed`, `watch`) gère l'état de la vue : ce n'est pas de la logique métier et c'est autorisé.

### Règle de pont

**Tout ce que Rust calcule, il l'envoie déjà calculé dans le DTO. Vue affiche, ne recalcule jamais une valeur métier.**

- Les commandes Tauri renvoient des DTO prêts à afficher, avec les champs dérivés déjà dedans (ex : `is_relance_due`, `jours_depuis_candidature`), pas des lignes SQL brutes.
- Si une règle dépend d'un setting (ex : `relance_delay_days`), le calcul vit en Rust pour rester à un seul endroit.

### Cas limites pré-tranchés

- **Filtrer / trier une liste déjà chargée** : autorisé en Vue, mais uniquement sur des champs déjà fournis par Rust (y compris les flags booléens dérivés). Évaluer soi-même un prédicat métier côté Vue est interdit (ex : "en retard de relance" doit arriver en booléen depuis Rust, Vue filtre dessus).
- **Compteurs / agrégats** (badges kanban) : Rust les renvoie dans le DTO du board (count par colonne). Vue n'agrège pas.
- **Formatage de dates / i18n** : seul relâchement autorisé. Le formatage locale pur (`Intl`) peut rester côté Vue, car c'est de la présentation, pas du métier. Option zéro ambiguïté : Rust renvoie l'ISO brut + un label pré-formaté.

### Contrat de commande (fail fast)

- Chaque commande renvoie `Result<Dto, AppError>`, avec `AppError` enum sérialisable.
- Vue reçoit une erreur typée et l'affiche. Pas de `try/catch` qui avale, pas de fallback silencieux sur valeur vide. Cohérent avec le principe "échouer vite plutôt qu'une dégradation silencieuse".

### Exemple de référence : drag-and-drop de statut (kanban)

1. Le déplacement *visuel* de la carte est géré par Vue (état de la vue).
2. Au drop, Vue appelle `update_status(candidature_id, new_status)`.
3. Rust valide la transition (règle métier), écrit en base, log dans `status_history`, renvoie le DTO à jour.
4. Si Rust refuse (`Err`), Vue annule le déplacement visuel et affiche l'erreur.

Vue propose, Rust dispose : Vue ne connaît pas les transitions légales.

### Synchronisation du contrat (types)

- Source de vérité du contrat IPC = les structs Rust. Les types TS sont **générés** depuis elles, jamais écrits à la main en parallèle (même réflexe que DBML pour le schéma).
- **Décision ouverte** (à trancher avant le premier prompt de liaison) :
    - `tauri-specta` v2 : génère types ET wrappers de commandes typés (`commands.getCandidatures()`), une signature Rust qui change casse la compile du front. Toujours en RC (~`2.0.0-rc.24`), à épingler strictement avec `=` dans `Cargo.toml`. Projet activement maintenu.
    - `ts-rs` : génère seulement les types TS, plus stable, mais les `invoke` restent manuels.