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
| IA | Couche compatible OpenAI, base URL configurable, pointée sur Ollama Cloud (free tier) |

## 3. Périmètre

### Dans le MVP

Cœur (suivi) :
- CRUD candidatures
- Pipeline de statut : à postuler, postulé, relancé, entretien, offre, refus
- Vues kanban et liste
- Rappels de relance

Features IA :
1. Extraction d'offre : coller le texte d'une annonce, l'IA remplit la fiche (entreprise, intitulé, type, lieu, stack, date limite, rémunération). Cas d'usage texte non structuré vers JSON structuré.
2. Relances intelligentes : détection des candidatures en attente et génération du mail de relance adapté au statut.
3. Tailoring CV / lettre de motivation : à partir du profil et de l'offre, mettre en avant les bons éléments.
4. Matching profil vers offre : pour une offre donnée, classer les projets de l'utilisateur par pertinence. **Développé en dernier** dans la séquence (dépend de l'infra embeddings).

### Hors MVP (évolutions)

- Recherche / scraping d'offres en ligne
- Mode IA local (d'abord détection d'un Ollama déjà installé, puis sidecar llama.cpp pour un vrai zéro-setup)
- Synchronisation multi-appareils
- Mobile
- Intégrations mail / calendrier

## 4. Décisions d'architecture IA

- **Couche unique compatible OpenAI**, avec base URL configurable. En V1 elle ne pointe que sur Ollama Cloud. Le jour où on ajoute le local (llama.cpp), c'est juste une URL de plus, aucune réécriture. La logique prompt vers JSON vit directement dans le backend Rust (HTTP + serde), pas de Python ni LangChain ici (le packaging Python dans Tauri est trop lourd).
- **Ollama Cloud free tier suffisant** pour ce cas : usage par à-coups et faible volume. Quotas mesurés en temps GPU, limites de session réinitialisées toutes les 5h, limites hebdo tous les 7 jours. Rester sur les modèles de niveau 1 et 2. Les structured outputs avec validation JSON Schema sont disponibles, parfait pour la feature d'extraction.
- **Pas de SLA sur Ollama Cloud** (service as is, panne connue d'avril 2026). C'est pourquoi le mode local reste le vrai fallback à terme.
- **Chaque utilisateur utilise son propre compte / clé Ollama**, donc son propre quota gratuit. Ça scale naturellement pour de l'open source sans clé centralisée.
- **Embeddings** : modèle bge-m3. Sur quelques centaines d'éléments en local, pas de pgvector ni d'extension : cosinus en brute-force dans le Rust, vecteurs stockés en BLOB dans SQLite. Le matching alimente aussi le tailoring (mêmes vecteurs).
- **Ne pas bundler Ollama dans l'installeur en V1.** Approche cloud-first : installeur léger, marche tout de suite avec une clé. Le sidecar llama.cpp viendra plus tard pour le mode local zéro-setup.

## 5. Sécurité des secrets

La clé API Ollama Cloud n'est **jamais** stockée en base, ni dans un .env, ni dans la table settings. Elle vit dans le **trousseau de l'OS** (Keychain macOS, Credential Manager Windows, Secret Service Linux), via la crate Rust `keyring` (ou un plugin Tauri équivalent). Le backend la lit du trousseau au moment d'appeler l'API. Aucune table ni colonne dédiée dans le schéma.

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
    references/   (CV et LM de référence)
    generated/    (versions taillées par candidature)
```

### Schéma

La source de vérité du schéma est en DBML dans `db/schema.dbml` (rendu visuel via dbdiagram.io). Tables retenues (schéma final) :

`users`, `lang`, `projets`, `experiences`, `skills`, `candidatures`, `status_history`, `contacts`, `interviews`, `documents`, `emails`, `email_attachments`, `embeddings`, `settings`.

Points de modélisation importants :
- **Entreprise dénormalisée** : les tables `companies` et `company_locations` ont été abandonnées dans le schéma final. La candidature porte directement `entreprise` (texte) et `localisation` (texte). Plus simple pour le MVP ; on pourra normaliser plus tard si le besoin de regrouper plusieurs candidatures par entreprise apparaît.
- **users** : porte aussi `permis` (boolean) et `description` (texte libre), en plus de l'identité et des liens (LinkedIn, GitHub, portfolio).
- **lang** : langues parlées par l'utilisateur (`nom`, `level`), liée à `users`. Sert au profil et au tailoring.
- **documents** : table unique avec auto-référence. `source_document_id` pointe vers une autre ligne (NULL = document de référence). `is_reference` (boolean) pour filtrer explicitement. `candidature_id` nullable (NULL = document maître, sinon version taillée). Les fichiers sont sur le disque, la base ne stocke que `file_path`.
- **embeddings** : table polymorphe. Colonnes `entity_type`, `entity_id` (FK logique, pas de contrainte stricte), `model`, `vector` (BLOB), index sur `(entity_type, entity_id)`. Permet d'embedder n'importe quel type sans migration.
- **status_history** : log des changements de statut (ancien, nouveau, date) pour les stats temps moyen entre statuts.
- **emails + email_attachments** : tous les mails envoyés depuis l'app, avec table de liaison vers les documents joints (un mail peut partir avec CV et LM).
- **skills** : liste à plat, une ligne par skill avec un champ `domaine` texte ("Langages", "Data & ML"...). Pas de table de liaison projet-skill pour le MVP.
- **candidatures** : porte `entreprise`, `localisation`, `remuneration`, `url_offre`, `description_offre` (texte brut de l'annonce, sert à l'extraction IA), `type`, `status`, `date_candidature`. Porte `next_relance` et `last_relance` en colonnes (pas de table reminders, le détail des relances vit dans `emails`). Porte aussi `source` (canal de candidature, voir section 8). Note : pas de champ `date_limite` dans le schéma final.

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

## 10. État d'avancement

Fait :
- Fondation Tauri 2 + Vue 3 + TS + Tailwind initialisée, hello world via commande Tauri `greet` (pont Rust vers Vue validé).
- Périmètre et stack figés.
- Schéma de base finalisé en DBML (`schema.dbml`), version définitive (entreprise dénormalisée, table `lang` ajoutée, `documents` avec auto-référence).
- README.md rédigé.
- Wireframe HTML statique implémenté (maquette de l'interface).
- Page Settings designée (Claude Design).

En cours / prochaine étape immédiate :
- Couche base de données : prompt d'init rédigé (création idempotente au boot, emplacement debug vs release, application du schéma, commande `db_status` de vérification). À implémenter et valider (la base se crée et l'UI liste les tables).

Ensuite :
- Premier vrai morceau métier : CRUD candidatures.
- Puis les features IA, en finissant par le matching (feature 4).

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