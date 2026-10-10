# RECETTE — application desktop autonome

Liste de contrôle tirée de l'inventaire fonctionnel (`DIAGNOSTIC_MIGRATION_DESKTOP.md`, section 1). Chaque action doit se comporter comme avec l'ancienne version (agent + PostgreSQL). À dérouler avec `npm run tauri dev` (depuis `crates/desktop/`) ou l'installeur, **sans Docker ni agent lancés**.

Prérequis : `.env` avec `ANTHROPIC_API_KEY`, à la racine du dépôt (dev) ou dans `%APPDATA%\com.findmyjob.app\.env` (app installée). Le profil a été repris ; aucune session, référence ni candidature n'existe au départ.

Légende : 🤖 = appel LLM (consomme des crédits API).

## 1.0 Démarrage

- [ ] L'app s'ouvre directement sur l'application principale (profil repris, pas d'onboarding).
- [ ] Le nom et les initiales affichés sont ceux du profil.
- [ ] La liste des sessions se charge (vide au premier lancement).
- [ ] Base illisible (ex. renommer temporairement `findmyjob.db` en dossier) : écran « Impossible de démarrer » avec un message commençant par « Impossible d'initialiser la base locale ». Remettre le fichier ensuite.
- [ ] `%LOCALAPPDATA%\com.findmyjob.app\logs\findmyjob.log` contient la ligne `.env chargé`.

## 1.1 Onboarding (sur une base vide uniquement)

- [ ] (Optionnel : déplacer `findmyjob.db*` ailleurs pour repartir de zéro, puis les remettre.)
- [ ] Navigation entre étapes ; permis oui / non / **aucun** (doit être accepté, enregistré NULL).
- [ ] Terminer crée l'utilisateur et ouvre l'application.

## 1.2 Barre latérale et sessions

- [ ] Sélectionner une session.
- [ ] Créer une session (barre latérale, et depuis le Suivi).
- [ ] Supprimer une session : elle disparaît, **et** son dossier `%APPDATA%\com.findmyjob.app\documents\sessions\<id>\` est supprimé (nouveau comportement).
- [ ] Aller au profil, aux paramètres.

## 1.3 Suivi

- [ ] Liste des candidatures de la session ; colonne « Candidaté » = date de création de la fiche.
- [ ] Ouvrir une candidature.
- [ ] Corbeille : liste en lecture seule des candidatures abandonnées / refusées / ghosting / offre refusée.
- [ ] Bouton « Nouvelle candidature ».
- [ ] Planning : entretiens de la session, vues semaine / mois / aujourd'hui, détail d'un entretien.
- [ ] Paramètres de session : renommer, changer le statut.
- [ ] Paramètres de session : chargement du CV et de la LM de référence (vides au départ).
- [ ] 🤖 Importer un `.typ` de CV puis de LM (compilation + templatisation, jusqu'à 3 essais). Vérifier la création de `documents\sessions\<id>\cv\versions\<n>\{reference.typ, reference.pdf, template.typ, example_data.json}`.
- [ ] Import d'un `.typ` invalide : message d'erreur, aucune version créée, aucun dossier orphelin sous `versions\`.
- [ ] Aperçu de la référence courante (recompilée).
- [ ] Historique des versions (après deux imports) ; aperçu d'une version ; restaurer une version (la date de la référence change, la version courante est marquée).

## 1.4 Nouvelle candidature

- [ ] Sans références prêtes : création bloquée avec le message habituel.
- [ ] 🤖 Analyser un lien (offre publique).
- [ ] Lien refusé ou page JavaScript : choix proposé ; ouvrir / rouvrir la page dans la fenêtre de capture.
- [ ] 🤖 Capturer la page : extraction, fenêtre fermée.
- [ ] 🤖 Analyser un texte collé ; texte vide refusé.
- [ ] Revenir au lien / quitter l'écran : la fenêtre de capture se ferme.
- [ ] Saisie manuelle.
- [ ] Avertissement de doublon en changeant de lien (lien déjà utilisé, avec ou sans `www.`, query, slash final).
- [ ] 🤖 Créer la candidature : la modale de progression démarre à « Étape 0 / 5 — Préparation… » puis avance jusqu'à la fin, **sans** erreur « Aucune génération en cours » au premier polling.
- [ ] Fichiers créés : `documents\sessions\<id>\{cv,lettre_motivation}\artefact\<cid>.pdf` et `.data.json`, `debug\<cid>\*.json`.
- [ ] Génération en échec (ex. clé API invalide dans `.env`) : message d'erreur, candidature supprimée, **aucun fichier partiel** restant.
- [ ] Fermer l'app pendant une génération, la relancer : la candidature sans documents a disparu, ses fichiers partiels aussi (ligne « génération(s) interrompue(s) nettoyée(s) » dans le log).
- [ ] Coquille vide inchangée : bloc « Contacts recruteur » (ajouter / supprimer, champs non envoyés).

## 1.5 Détail d'une candidature

- [ ] Détail, contacts (vides), historique.
- [ ] État des PDF joints (présence, taille).
- [ ] Éditer poste, entreprise, lieu, type, rémunération, source, lien, notes (statut « À postuler » seulement) ; champ obligatoire vidé → message.
- [ ] Voir le contenu de l'offre.
- [ ] Aperçu CV / LM.
- [ ] Télécharger CV / LM (dossier Téléchargements).
- [ ] Ajouter un événement : liste des événements possibles selon le statut ; entretiens en attente ; enregistrer (note, message, appel, relance, entretien planifié avec date, entretien passé, offre, refus, ghosting, désistement, accepter / décliner l'offre).
- [ ] Lire le message d'une entrée d'historique.
- [ ] Supprimer (statut « À postuler ») : candidature et fichiers supprimés.
- [ ] 🤖 Postuler par mail ; 🤖 par message : texte rédigé, statut « Postulé », export `.txt` + PDF.
- [ ] Postuler sans message.
- [ ] Coquille vide inchangée : « + Ajouter » un recruteur (contact fictif local, perdu au rechargement).
- [ ] Code mort inchangé : popup « Rédiger le mail de candidature » (inaccessible).

## 1.6 Profil

- [ ] Profil complet repris : identité, 2 expériences, 2 formations, 2 projets, 19 compétences, 2 langues.
- [ ] Éditer les infos de contact ; la ville reste affichée sans être éditable.
- [ ] Éditer la description.
- [ ] Ouvrir LinkedIn / GitHub / portfolio / URL de projet.
- [ ] Expériences, formations, projets : ajouter / modifier / supprimer.
- [ ] Compétences : ajouter / modifier / supprimer ; **doublon** (même domaine et même nom) → message « La compétence « X » existe déjà dans le domaine « Y ». » (nouveau comportement : refusé au lieu d'être accepté).
- [ ] Ordre des compétences : par domaine puis nom, sans tenir compte de la casse (`Data & ML` avant `Data Engineering`, `pgvector` avant `Polars`).
- [ ] Langues : ajouter / modifier / supprimer ; doublon → « La langue « X » est déjà dans votre profil. ».

## 1.7 Paramètres

- [ ] Lire / choisir le modèle Claude.
- [ ] 🤖 Tester la connexion (succès, latence) ; sans clé : message « Aucune clé API Anthropic configurée. ».
- [ ] Thème clair / sombre / système, conservé au redémarrage.
- [ ] Polices : lister, importer un dossier (copie sous `%APPDATA%\com.findmyjob.app\polices\<famille>\`), supprimer.
- [ ] Ouvrir fonts.google.com.
