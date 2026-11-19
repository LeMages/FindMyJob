/**
 * Textes de l'interface en français — langue de référence. `en.ts` doit
 * avoir exactement la même forme (vérifié par `vue-tsc` via `satisfies`).
 *
 * Syntaxe `vue-i18n` : `{nom}` est une interpolation, `|` sépare les formes
 * du pluriel, `@` doit être échappé (`{'@'}`).
 */

export const fr = {
  // ======================================================================
  // Éléments communs
  // ======================================================================
  common: {
    loading: 'Chargement…',
    save: 'Enregistrer',
    saving: 'Enregistrement…',
    delete: 'Supprimer',
    cancel: 'Annuler',
    irreversible: 'Cette action est irréversible.',
    yes: 'Oui',
    no: 'Non',
    previous: '← Précédent',
    next: 'Suivant →',
    close: 'Fermer',
    edit: 'Modifier',
    validate: 'Valider',
    add: '+ Ajouter',
    not_specified: 'Non renseigné',
    select_placeholder: '-- Sélectionner --',
    back: '← Retour',
    unknown_company: 'Entreprise inconnue',
    date_missing: 'Date non renseignée',
    date: {
      at: 'à',
      present: 'Présent',
    },
    months: {
      short: {
        jan: 'Jan',
        feb: 'Fév',
        mar: 'Mar',
        apr: 'Avr',
        may: 'Mai',
        jun: 'Juin',
        jul: 'Juil',
        aug: 'Août',
        sep: 'Sept',
        oct: 'Oct',
        nov: 'Nov',
        dec: 'Déc',
      },
    },
    weekdays: {
      short: {
        mon: 'Lun',
        tue: 'Mar',
        wed: 'Mer',
        thu: 'Jeu',
        fri: 'Ven',
        sat: 'Sam',
        sun: 'Dim',
      },
    },
    file_size: {
      b: 'o',
      kb: 'Ko',
      mb: 'Mo',
    },
  },

  // ======================================================================
  // Erreurs
  // ======================================================================
  error: {
    // Messages génériques par `AppError.kind`, affichés seulement en anglais
    // (en français, le message d'origine du backend est affiché tel quel).
    kind: {
      Database: 'Erreur de base de données.',
      Validation: 'Donnée invalide.',
      Conflict: 'Conflit avec une donnée existante.',
      NotFound: 'Élément introuvable.',
      Config: 'Erreur de configuration.',
      Llm: "Erreur de l'IA.",
      Scraping: "Échec de l'extraction de l'offre.",
      OutdatedReference: 'Document de référence obsolète : réimporte-le.',
      SessionLocked: 'Session en lecture seule.',
    },
    fallback: {
      startup: "Impossible de démarrer l'application.",
      unexpected: 'Une erreur inattendue est survenue.',
      create_session: 'Impossible de créer la session.',
      update_session: 'Impossible de mettre à jour la session.',
      load_session_documents: 'Impossible de charger les documents de la session.',
      load_versions: "Impossible de charger l'historique des versions.",
      restore_version: 'Impossible de restaurer cette version.',
      load_version_preview: "Impossible de charger l'aperçu de cette version.",
      save_cv: 'Erreur lors de la sauvegarde du CV.',
      save_cover_letter: 'Erreur lors de la sauvegarde de la lettre de motivation.',
      compile_cv: 'Erreur lors de la compilation du CV.',
      compile_cover_letter: 'Erreur lors de la compilation de la lettre de motivation.',
      load_preview: "Impossible de charger l'aperçu du document.",
      download_document: 'Impossible de télécharger le document.',
      download_application_documents: 'Impossible de télécharger les documents de la candidature.',
      load_fonts: 'Impossible de charger les polices.',
      import_font: "Impossible d'importer ce dossier de police.",
      delete_font: 'Impossible de supprimer cette police.',
      generation_tracking: 'Erreur de suivi de la génération des documents.',
      save_theme: "Impossible d'enregistrer le thème.",
      load_application: 'Impossible de charger la candidature.',
      load_applications: 'Impossible de charger les candidatures.',
      create_application: 'Erreur lors de la création de la candidature.',
      update_application: 'Impossible de mettre à jour la candidature.',
      delete_application: 'Impossible de supprimer la candidature.',
      add_event: "Impossible d'ajouter l'événement.",
      load_available_events: 'Impossible de charger les événements disponibles.',
      load_pending_interviews: 'Impossible de charger les entretiens en attente.',
      load_interviews: 'Impossible de charger les entretiens.',
      load_contacts: 'Impossible de charger le carnet de contacts.',
      link_contact: 'Impossible de lier le contact.',
      create_contact: 'Impossible de créer le contact.',
      update_contact: 'Impossible de modifier le contact.',
      unlink_contact: 'Impossible de retirer le contact de la candidature.',
      delete_contact: 'Impossible de supprimer le contact.',
      apply_without_message: 'Impossible de postuler sans message.',
      apply_by_email: 'Impossible de postuler par mail.',
      apply_by_message: 'Impossible de postuler par message.',
      analyze_offer: "L'analyse de l'offre a échoué.",
      check_duplicates: 'Vérification des doublons impossible.',
      open_offer_page: "Impossible d'ouvrir la page de l'offre.",
      capture_failed: 'La capture de la page a échoué.',
      extract_from_text: "Impossible d'extraire les informations de ce texte.",
      close_capture_window: 'Impossible de fermer la fenêtre de la page.',
      open_link: "Impossible d'ouvrir ce lien.",
      save_profile: 'Erreur lors de la sauvegarde.',
      load_profile: 'Impossible de charger le profil.',
      read_api_key: "Impossible de lire l'état de la clé API.",
      save_api_key: "Impossible d'enregistrer la clé API.",
      delete_api_key: 'Impossible de supprimer la clé API.',
      connection_test: 'Le test de connexion a échoué.',
    },
  },

  // ======================================================================
  // Toasts
  // ======================================================================
  toast: {
    document_downloaded: 'Document téléchargé : {path}',
    documents_downloaded: 'Documents téléchargés dans {dir}',
  },

  // ======================================================================
  // Candidatures : statuts, contrats, sources
  // ======================================================================
  status: {
    a_postuler: 'À postuler',
    postule: 'Candidature envoyée',
    relance: 'Discussion',
    entretien: 'Entretien',
    entretien_passe: 'En attente de réponse',
    offre: 'Offre',
    refus: 'Refus',
    abandon: 'Abandon',
    ghosting: 'Ghosté',
    accepte: 'Accepté',
    offre_refusee: 'Offre déclinée',
  },
  contract: {
    stage: 'Stage',
    alternance: 'Alternance',
    interim: 'Intérim',
    cdd: 'CDD',
    cdi: 'CDI',
  },
  source: {
    online_platform: 'Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)',
    company_site: "Site carrière de l'entreprise (formulaire sur le site)",
    email: 'Mail (candidature directe par email)',
    referral: 'Cooptation / Réseau (recommandation, contact perso)',
    job_fair: 'Forum / Salon emploi (rencontre physique)',
    school: 'École (job board, offres des profs / asso)',
    other: 'Autre',
  },

  // ======================================================================
  // Timeline de suivi
  // ======================================================================
  event: {
    // Libellés des événements passés (historique).
    history: {
      creation: 'Candidature créée',
      candidature_envoyee: 'Candidature envoyée',
      relance: 'Relance envoyée',
      entretien_planifie: 'Entretien planifié',
      entretien_passe: 'Entretien passé',
      offre_recue: 'Offre reçue',
      refus_recu: 'Refus reçu',
      ghosting: 'Ghosting',
      desistement: 'Désistement',
      accepter_offre: 'Offre acceptée',
      decliner_offre: 'Offre déclinée',
      message_envoye: 'Message envoyé',
      message_recu: 'Message reçu',
      appel: 'Appel',
      note: 'Note',
    },
    // Libellés des événements proposés à l'ajout (mêmes textes que le backend).
    action: {
      message_envoye: 'Message envoyé',
      message_recu: 'Message reçu',
      appel: 'Appel',
      note: 'Note',
      relance: 'Relance',
      entretien_planifie: 'Entretien planifié',
      entretien_passe: 'Entretien passé',
      offre_recue: 'Offre reçue',
      refus_recu: 'Refus reçu',
      ghosting: 'Ghosting',
      accepter_offre: "Accepter l'offre",
      decliner_offre: "Décliner l'offre",
      desistement: 'Se désister',
    },
    category: {
      communication: 'Communication',
      interview: 'Entretien',
      follow_up: 'Suivi',
      outcome: 'Issue',
      other: 'Autre',
    },
  },
  interview: {
    type_default: 'Entretien',
    type: {
      rh: 'RH',
      technique: 'Technique',
      manager: 'Manager',
      final: 'Final',
      autre: 'Autre',
    },
    issue: {
      positif: 'Positif',
      negatif: 'Négatif',
      pending: 'En attente',
    },
    detail: {
      modality: 'Modalité',
      modality_missing: 'Non renseignée',
      location: 'Lieu',
      contacts: 'Interlocuteurs',
      notes: 'Notes',
    },
    modalite: {
      visio: 'Visio',
      presentiel: 'Présentiel',
      telephone: 'Téléphone',
    },
  },

  // ======================================================================
  // Sessions, contacts, profil
  // ======================================================================
  session: {
    params: {
      section: 'Session',
      name: 'Nom',
      name_help: 'Affiché dans la barre latérale.',
      status: 'Statut',
      status_help: 'Une session en pause ou terminée reste consultable, en lecture seule : seul son statut reste modifiable.',
      city: 'Ville',
      zip_code: 'Code postal',
      city_placeholder: 'Lyon',
      zip_code_placeholder: '69000',
      city_help: "Ville de l'expéditeur, reprise dans le CV et la lettre.",
      city_required: 'Obligatoire pour créer une candidature.',
      documents_title: 'Modèles de documents',
      documents_description: 'Modèles Typst à partir desquels le CV et la lettre sont générés pour chaque candidature.',
      markup_summary: 'Comment baliser un modèle (consignes, zones fixes et variables)',
    },
    create: {
      title: 'Nouvelle session',
      intro: 'Une session regroupe un ensemble de candidatures (ex : « Stage M2 – 2025 »).',
      name: 'Nom de la session',
      placeholder: 'Ex. Stage M2 – 2025',
      city_optional: 'Ville (facultatif)',
      zip_code_optional: 'Code postal (facultatif)',
      submit: 'Créer la session',
    },
    delete: {
      title: 'Supprimer la session',
      body: '{name} et toutes ses candidatures seront supprimées définitivement.',
    },
    status: {
      active: 'Active',
      pause: 'En pause',
      termine: 'Terminée',
    },
    locked: {
      banner: {
        pause: 'Session en pause : lecture seule',
        termine: 'Session terminée : lecture seule',
      },
      change_status: 'Changer le statut',
      hint: 'Session en lecture seule : repasse-la en « Active » dans Paramètres pour la modifier.',
    },
  },
  contacts: {
    entry_title: 'CONTACT {n}',
    empty_new: 'Aucun contact pour cette candidature.',
    add: '+ Ajouter un contact',
    errors: {
      pick_from_book: 'Choisis un contact du carnet.',
    },
  },
  profile: {
    title: 'Mon profil',
    subtitle: 'Ces informations pré-remplissent tes candidatures.',
    driving_licence: 'Permis B',
    saving: 'Sauvegarde…',
    pending: {
      edited: 'Modifié',
      new: 'Non enregistré',
    },
    bullet: {
      add: 'Ajouter une puce',
      label: 'Puce',
    },
    in_progress: 'En cours',
    start_year: 'Année de début',
    end_year: 'Année de fin',
    description: 'Description',
    detailed_context: {
      label: 'Contexte détaillé',
      help: "Jamais affiché sur le CV. Sert à l'IA pour la lettre et le choix des projets.",
      provided: 'Contexte renseigné',
      missing: 'Contexte détaillé à compléter (utile pour la lettre et le choix des projets)',
      counter: '{count} / {max}',
    },
    identity: {
      written_in_fr: 'Profil rédigé en français',
      written_in_en: 'Profil rédigé en anglais',
      add_email: 'Ajouter un e-mail',
      add_phone: 'Ajouter un téléphone',
      add_linkedin: 'Ajouter LinkedIn',
      add_github: 'Ajouter GitHub',
      add_portfolio: 'Ajouter un portfolio',
      no_licence: 'Pas de permis',
      licence_unknown: 'Permis non renseigné',
    },
    info: {
      section: 'INFORMATIONS',
      first_name: 'Prénom',
      first_name_required: 'Le prénom est requis.',
      last_name_required: 'Le nom est requis.',
      last_name: 'Nom',
      email: 'Adresse e-mail',
      phone: 'Téléphone',
      profile_language: 'Langue dans laquelle le profil est rédigé',
      lang_fr: 'Français',
      lang_en: 'Anglais',
    },
    placeholders: {
      first_name: 'Prénom',
      last_name: 'Nom',
      email: "prenom.nom{'@'}email.com",
      phone: '+33 6 00 00 00 00',
      portfolio: 'https://monsite.com',
      about: 'Décris ton parcours, tes objectifs…',
    },
    links: {
      invalid_linkedin: "Le lien LinkedIn n'est pas une URL valide.",
      invalid_github: "Le lien GitHub n'est pas une URL valide.",
      invalid_portfolio: "Le lien portfolio n'est pas une URL valide.",
    },
    about: {
      section: 'À PROPOS',
    },
    experiences: {
      section: 'EXPÉRIENCES',
      empty: 'Aucune expérience renseignée',
      add_title: 'Ajouter une expérience',
      edit_title: 'Modifier une expérience',
      job_title: 'Intitulé du poste *',
      job_placeholder: 'Développeur Full-Stack',
      location_placeholder: 'Paris, France',
      start: 'Début',
      end: 'Fin',
      description_placeholder: 'Décris tes missions, technologies utilisées…',
    },
    formations: {
      section: 'FORMATION',
      empty: 'Aucune formation renseignée',
      add_title: 'Ajouter une formation',
      edit_title: 'Modifier une formation',
      title: 'Titre / Diplôme *',
      title_placeholder: 'Master Informatique',
      school: 'École / Établissement',
      school_placeholder: 'École Supérieure du Numérique',
      details: 'Détails (un par ligne)',
      details_placeholder: 'Spécialité, cours clés, échange universitaire...',
    },
    projects: {
      section: 'PROJETS',
      empty: 'Aucun projet renseigné',
      add_title: 'Ajouter un projet',
      edit_title: 'Modifier un projet',
      title: 'Titre *',
      title_placeholder: 'Mon super projet',
      for_whom: 'Pour qui',
      for_whom_placeholder: 'Perso, École, Client...',
      stack: 'Stack utilisé',
      link: 'Lien',
      description_placeholder: 'Ce projet consiste à...',
    },
    skills: {
      section: 'COMPÉTENCES',
      empty: 'Aucune compétence renseignée',
      add_title: 'Ajouter une compétence',
      edit_title: 'Modifier une compétence',
      name: 'Nom de la compétence *',
      domain: 'Domaine *',
      domain_placeholder: 'Langages, Frameworks, Outils…',
      create: 'Créer {name}',
      quoted: '« {name} »',
      no_domain: 'Aucun domaine existant',
    },
    languages: {
      section: 'LANGUES',
      empty: 'Aucune langue renseignée',
      add_title: 'Ajouter une langue',
      edit_title: 'Modifier une langue',
      name: 'Langue *',
      name_placeholder: 'Anglais, Espagnol…',
      level_label: 'Niveau',
      level: {
        native: 'Langue maternelle',
        bilingual: 'Bilingue',
        c2: 'Courant (C2)',
        c1: 'Avancé (C1)',
        b2: 'Intermédiaire avancé (B2)',
        b1: 'Intermédiaire (B1)',
        a2: 'Élémentaire (A2)',
        a1: 'Débutant (A1)',
      },
    },
  },

  // ======================================================================
  // Import d'offre
  // ======================================================================
  offer_import: {
    failure: {
      invalid_url: "Ce lien n'est pas valide : seules les adresses web publiques en http ou https sont acceptées.",
      offer_unavailable: "Cette offre est introuvable ou a expiré : la page n'existe plus sur le site.",
      access_denied: 'Ce site refuse la lecture automatique de ses pages.',
      timeout: "Le site n'a pas répondu à temps.",
      network: "Le site n'a pas pu être joint.",
      not_html: 'Ce lien ne mène pas à une page web (fichier PDF ou autre document).',
      empty_page: 'La page récupérée est vide.',
      too_large: 'La page est trop volumineuse pour être analysée (plus de 5 Mo).',
      insufficient_content:
        "La page récupérée ne contient pas le texte de l'offre (contenu chargé en JavaScript, vérification anti-robot ou bandeau cookies).",
    },
  },

  // ======================================================================
  // Détail d'une candidature
  // ======================================================================
  detail: {
    back_to_tracking: '← Retour au suivi',
    job: {
      section: 'DÉTAILS DU POSTE',
      view_offer: "Voir l'offre →",
      title: 'Intitulé du poste',
      title_placeholder: 'Ex. Stage Data Scientist',
      company: 'Entreprise',
      company_placeholder: "Nom de l'entreprise",
      location: 'Lieu',
      location_placeholder: 'Ville',
      contract: 'Type de contrat',
      salary: 'Gratification',
      salary_placeholder: 'Ex. 1 300 €/mois',
      platform: "Plateforme de l'offre",
      link: "Lien de l'offre",
    },
    notes: {
      section: 'NOTES',
      placeholder: "Notes libres, retours d'entretien, contacts...",
    },
    documents: {
      section: 'DOCUMENTS',
      cv: 'CV',
      cover_letter: 'Lettre de motivation',
      attached: 'Joint · {size}',
      missing: 'Non existant',
      cv_generated_with: 'Généré avec la référence v{version}',
      lm_generated_with: 'Générée avec la référence v{version}',
      preview: 'Visualiser',
      download: 'Télécharger',
      cv_preview_title: 'Aperçu du CV',
      lm_preview_title: 'Aperçu de la lettre de motivation',
      warnings_title: 'Avertissements de la génération',
    },
    contacts: {
      section: 'CONTACTS RECRUTEUR',
      unlink: 'Retirer de la candidature',
      delete: 'Supprimer définitivement',
      empty: 'Aucun contact renseigné.',
    },
    footer: {
      accepted: '✓ Candidature acceptée — félicitations !',
      closed: 'Candidature clôturée — {status}.',
      apply: 'Postuler',
    },
    apply: {
      title: 'Postuler',
      question: 'Comment veux-tu postuler à cette offre ?',
      mail_title: 'Postuler par mail',
      mail_writing: "Rédaction du mail par l'IA…",
      mail_description: 'Rédiger et envoyer un mail de candidature.',
      message_title: 'Postuler par message',
      message_writing: "Rédaction du message par l'IA…",
      message_description: "Envoyer un message via la plateforme de l'offre.",
      none_title: 'Postuler sans message',
      none_description: 'Marquer la candidature comme envoyée, sans message.',
    },
    mail: {
      title: 'Rédiger le mail de candidature',
      subject_label: 'Objet',
      subject: 'Candidature — {role} — Alex Martin',
      body_label: 'Corps du mail',
      body_placeholder: 'Madame, Monsieur, ...',
      generate: '✦ Générer par IA',
    },
    delete_contact: {
      title: 'Supprimer définitivement le contact',
      body: '{name} sera retiré du carnet, de cette candidature et de ses entretiens.',
      also_linked:
        "Il est aussi lié à {n} autre candidature, d'où il sera également retiré. | Il est aussi lié à {n} autres candidatures, d'où il sera également retiré.",
      confirm: 'Supprimer définitivement',
    },
    offer: {
      title: "Contenu de l'offre",
      empty: "Aucun contenu d'offre renseigné.",
    },
    sent_content: {
      title: 'Contenu envoyé',
    },
    delete_application: {
      title: 'Supprimer la candidature',
      body: 'Cette candidature sera définitivement retirée de votre suivi. Cette action est irréversible.',
    },
    timeline: {
      section: 'STATUT & SUIVI',
      view_content: 'Voir le contenu',
      empty: "Aucun suivi pour l'instant.",
      add_event: '+ Ajouter un événement',
    },
    add_event: {
      title: 'Ajouter un événement',
      empty: 'Aucun événement disponible pour ce statut.',
      interview_type: "Type d'entretien",
      date_time: 'Date et heure',
      location: 'Lieu (optionnel)',
      location_placeholder: "Visio, locaux de l'entreprise...",
      modality: 'Modalité (optionnel)',
      modality_unspecified: 'Non précisée',
      contacts: 'Interlocuteurs (optionnel)',
      no_contacts: 'Aucun contact lié à cette candidature.',
      interview: 'Entretien',
      pick_interview: "Choisir l'entretien passé",
      issue: 'Issue',
      content: {
        relance: 'Contenu du message',
        message: 'Contenu',
        appel: "Notes de l'appel",
        optional_notes: 'Notes (optionnel)',
        decline_reason: 'Raison du refus (optionnel)',
        note: 'Note',
        default: 'Notes',
      },
    },
  },

  // ======================================================================
  // Nouvelle candidature
  // ======================================================================
  new_application: {
    title: 'Nouvelle candidature',
    no_session:
      "Aucune session de recherche n'est sélectionnée : crée ou choisis une session dans le suivi avant d'ajouter une candidature.",
    go_to_tracking: 'Aller au suivi',
    go_to_settings: 'Aller aux paramètres',
    back_to_offer_link: "← Revenir au lien de l'offre",
    back_to_link: '← Revenir au lien',
    analyzing: 'Analyse en cours…',
    manual: 'Saisie manuelle',
    submit: 'Valider la candidature →',
    form: {
      location_placeholder: 'Ville, pays',
      offer_placeholder: "Colle ici le texte complet de l'offre d'emploi...",
      notes_placeholder: 'Notes libres, contexte, points à préparer...',
    },
    link: {
      section: "LIEN DE L'OFFRE",
      intro:
        "Colle le lien de l'offre pour pré-remplir automatiquement le formulaire, ou passe directement à la saisie manuelle.",
      unavailable_hint: 'Vérifie le lien ou choisis une autre offre.',
      placeholder: 'https://www.exemple.fr/offres/...',
      analyze: "Analyser l'offre →",
    },
    capture: {
      section: "PAGE DE L'OFFRE",
      intro:
        "La page de l'offre est ouverte dans une fenêtre FindMyJob. Accepte les cookies ou connecte-toi si le site le demande, attends que le texte de l'offre soit affiché, puis capture la page. Ta session sur ce site est conservée pour les prochaines fois.",
      capture: 'Capturer cette page',
      reopen: 'Rouvrir la fenêtre',
      paste: 'Coller le texte',
    },
    paste: {
      section: "TEXTE DE L'OFFRE",
      intro: "Sélectionne tout le texte de la page de l'offre et colle-le ci-dessous.",
      label: "Texte de l'offre",
      analyze: 'Analyser le texte →',
    },
    choice: {
      section: 'ANALYSE AUTOMATIQUE IMPOSSIBLE',
      prompt: 'Choisis comment continuer :',
      capture: 'Ouvrir la page dans FindMyJob',
      paste: "Coller le texte de l'offre",
    },
    duplicates: {
      title: 'Tu as déjà une candidature pour cette offre : | Tu as déjà {n} candidatures pour cette offre :',
      item: '({status}, session « {session} »)',
      hint: 'Tu peux quand même créer celle-ci.',
    },
  },

  // ======================================================================
  // Génération des documents
  // ======================================================================
  generation: {
    title: 'Génération des documents',
    starting: 'Démarrage…',
    step_counter: 'Étape {current} / {total}',
    failed: 'La génération des documents a échoué.',
    // Libellés des étapes, indexés par `current_step` : ils suivent l'ordre
    // des étapes de `pipeline.rs` (`on_progress`), l'étape 0 étant la
    // préparation publiée par `services/generation.rs`. À mettre à jour si
    // le pipeline change d'étapes.
    step: {
      '0': 'Préparation…',
      '1': "Analyse de l'offre…",
      '2': 'Matching profil / offre…',
      '3': 'Rédaction du CV et de la lettre…',
      '4': 'Assemblage et compilation du CV…',
      '5': 'Assemblage et compilation de la lettre…',
    },
  },

  // ======================================================================
  // Suivi et planning
  // ======================================================================
  stats: {
    load_error: 'Impossible de charger les statistiques.',
    empty_title: 'Pas encore assez de données',
    empty_body: 'Les statistiques apparaissent à partir de {n} candidatures envoyées dans la session.',
    days: '{n} j',
    kpi: {
      sent: 'Candidatures envoyées',
      sent_week: '+{n} cette semaine',
      reply_rate: 'Taux de réponse',
      interview_rate: "Taux d'entretien",
      of: '{a} sur {b}',
      first_reply: 'Première réponse',
      first_reply_sub: 'délai moyen après envoi',
    },
    funnel: {
      title: 'Entonnoir',
      hint: "Taux de passage d'une étape à la suivante",
      sent: 'Envoyées',
      replied: 'Réponse',
      interviewed: 'Entretien',
      offer: 'Offre',
    },
    outcomes: {
      title: 'Issues',
      hint: 'Où en sont les candidatures envoyées',
      note: '{n} candidature envoyée dans la session. | {n} candidatures envoyées dans la session.',
      labels: {
        in_progress: 'En cours',
        offer: 'Offre',
        refus: 'Refus',
        ghosting: 'Ghosting',
        abandon: 'Abandon',
        offre_refusee: 'Offre déclinée',
      },
    },
    weekly: {
      title: 'Activité par semaine',
      hint: '{n} dernières semaines',
      sent: 'Candidatures envoyées',
      interviews: 'Entretiens',
      tooltip: 'Semaine du {date}',
      sent_count: '{n} candidature | {n} candidatures',
      interview_count: '{n} entretien | {n} entretiens',
    },
    works: {
      title: 'Ce qui marche',
      note: 'Les lignes grisées comptent moins de {n} envois : trop peu pour en tirer une conclusion.',
      reply_rate: 'Taux de réponse',
      interviews: 'Entretiens',
      sends: '{n} envoi | {n} envois',
      too_few: 'trop peu',
      few_hint: 'Moins de {n} envois : trop peu pour conclure',
      row_hint: '{replied} réponses sur {sent} envois',
      criteria: {
        cv_template: { title: 'Template de CV', hint: 'Version de référence utilisée à la génération' },
        lm_template: { title: 'Template de lettre', hint: 'Version de référence utilisée à la génération' },
        source: { title: "Plateforme d'origine", hint: 'Plateforme indiquée pour la candidature' },
        contact: { title: 'Contact recruteur', hint: 'Au moins un contact lié à la candidature' },
      },
      rows: {
        cv_version: 'CV v{n}',
        lm_version: 'LM v{n}',
        unknown_version: 'Version inconnue',
        with_contact: 'Avec contact',
        without_contact: 'Sans contact',
      },
    },
    delays_title: 'Entretiens et délais',
    interviews: {
      title: 'Entretiens',
      hint: 'Passés, par type, et issues positives',
      positive: '{n} positif | {n} positifs',
      row_hint: '{type} : {positive} positifs sur {passed}',
      empty: 'Aucun entretien passé.',
    },
    durations: {
      title: 'Durées',
      hint: "Médiane depuis l'envoi",
      to_offer: "Jusqu'à une offre",
      to_rejection: "Jusqu'à un refus",
      to_ghosting: 'Avant de classer en ghosting',
    },
    follow_ups: {
      title: 'Relances',
      hint: 'Effet des relances envoyées',
      sent: 'Relances envoyées',
      answered: "Suivies d'une réponse",
      delay: 'Délai moyen avant relance',
    },
  },
  tracking: {
    create_first_session: '+ Créer ma première session',
    summary: {
      total: '{n} candidature | {n} candidatures',
      open: '{n} en cours',
      closed: '{n} clôturée | {n} clôturées',
    },
    documents_blocking_hint: "{message} Complète-les dans l'onglet Paramètres.",
    new_application: '+ Nouvelle candidature',
    columns: {
      company: 'ENTREPRISE',
      role: 'POSTE',
      status: 'STATUT',
      applied: 'CANDIDATÉ',
      inactivity: 'INACTIVITÉ',
      closed_on: 'CLÔTURÉE LE',
    },
    inactivity_days: '{n} j',
    empty: 'Aucune candidature',
    filters: {
      all: 'Toutes',
      todo: 'À postuler',
      active: 'En cours',
      offer: 'Offres',
      closed: 'Clôturées',
    },
    closed_outcome: {
      refus: '{n} refus | {n} refus',
      abandon: '{n} abandon | {n} abandons',
      ghosting: '{n} ghosté | {n} ghostés',
      offre_refusee: '{n} offre déclinée | {n} offres déclinées',
    },
    filter_empty: 'Aucune candidature dans ce filtre.',
    tabs: {
      applications: 'Candidatures',
      planning: 'Planning',
      stats: 'Statistiques',
      settings: 'Paramètres',
    },
  },
  planning: {
    count: '{n} entretien · {range} | {n} entretiens · {range}',
    week: 'Semaine',
    month: 'Mois',
    previous: 'Période précédente',
    today: "Aujourd'hui",
    next: 'Période suivante',
    empty: 'Aucun entretien',
  },

  // ======================================================================
  // Documents de référence
  // ======================================================================
  documents: {
    reference: {
      none_cv: 'Aucun modèle importé. Il est nécessaire pour générer le CV.',
      none_lm: 'Aucun modèle importé. Il est nécessaire pour générer la lettre.',
      ready: 'Prêt',
      missing: 'Manquant',
      imported_on: 'Importé le {date}',
      importing: 'Import en cours…',
      preview: 'Aperçu',
      replace: 'Remplacer',
      history: 'Historique',
      import: 'Importer un modèle .typ',
      iframe_cv: 'Aperçu PDF du CV',
      iframe_lm: 'Aperçu PDF de la lettre de motivation',
      with_version: '{title} — version {version}',
      only_typ: 'Seuls les fichiers .typ sont acceptés.',
      compiling: 'Compilation en cours…',
    },
    versions: {
      title_cv: 'Versions – CV',
      title_lm: 'Versions – Lettre de motivation',
      empty: 'Aucune version enregistrée',
      current: 'actuelle',
      preview: 'Visualiser la version {version}',
      restore_hint: 'Utiliser cette version pour les prochaines générations',
      restoring: 'Restauration…',
      restore: 'Restaurer',
    },
    preview: {
      downloading: 'Téléchargement…',
      pdf_title: 'Aperçu PDF',
    },
    missing: {
      message: 'Document(s) de référence manquant(s) ou non analysé(s) : {list}.',
      cv: 'le CV',
      lm: 'la lettre de motivation',
      both: 'le CV et la lettre de motivation',
    },
    markup: {
      summary: 'Balisage des consignes',
      intro: "Les consignes s'écrivent en commentaires {comment} dans le fichier .typ ; elles n'apparaissent pas dans le PDF.",
      instruction: 'consigne',
      general:
        "{tag} puis des lignes {comment} : consignes générales, jusqu'à la première ligne qui n'est pas un commentaire.",
      fixed: '{tag} : le texte qui suit est recopié mot pour mot (traduit fidèlement si le document est généré dans une autre langue).',
      variable: "{tag} : le texte qui suit est réécrit d'après l'offre.",
      adaptable: "{tag} : le texte qui suit est réécrit d'après votre profil, sans rien inventer.",
      zones:
        "Une zone va jusqu'à la balise suivante ou jusqu'à la ligne vide. Les balises s'appliquent au texte rédigé : objet et paragraphes de la lettre, titre et résumé du CV. Un texte rédigé sans balise est traité comme {tag}.",
      must_open: 'La balise doit ouvrir le commentaire : {example} est refusé.',
      example_heading: 'Corps',
    },
  },

  // ======================================================================
  // Contacts, champs éditables, barre latérale, démarrage
  // ======================================================================
  contact_form: {
    existing: 'Contact existant',
    new: 'Nouveau contact',
    from_book: 'Contact du carnet',
    pick: 'Choisir un contact…',
    name: 'Nom',
    name_placeholder: 'Prénom Nom',
    role: 'Rôle',
    role_placeholder: 'Ex. Chargé·e de recrutement',
    email: 'Email',
    email_placeholder: "contact{'@'}entreprise.com",
    phone: 'Téléphone',
    phone_placeholder: '06 00 00 00 00',
    notes: 'Notes',
    notes_placeholder: 'Rencontré au salon, préfère être contacté par mail...',
    edit_title: 'Modifier le contact',
    add_title: 'Ajouter un contact',
    shared_warning:
      "Ce contact est lié à {n} candidature : la modification s'applique partout. | Ce contact est lié à {n} candidatures : la modification s'applique partout.",
  },
  editable_field: {
    locked: 'Modifiable uniquement au statut « À postuler »',
    click_to_edit: 'Cliquer pour modifier',
    double_click_to_edit: 'Double-cliquer pour modifier',
  },
  sidebar: {
    applications: 'Candidatures',
    new_session: 'Nouvelle session',
    profile: 'Profil',
    settings: 'Paramètres',
    collapse: 'Réduire',
    expand: 'Déplier',
    save_error: "Impossible d'enregistrer l'état de la barre latérale.",
  },
  startup: {
    failed: 'Impossible de démarrer',
  },

  // ======================================================================
  // Paramètres
  // ======================================================================
  settings: {
    title: 'Paramètres',
    subtitle: "Réglages de l'IA, des documents et de l'interface.",
    ai: {
      section: 'Intelligence artificielle',
      description: 'Modèle et clé utilisés pour analyser les offres et rédiger tes documents.',
      model: 'Modèle',
      recommended: 'Recommandé',
      models: {
        sonnet: { name: 'Claude Sonnet 4.6', description: 'Meilleure qualité pour le CV et la lettre.' },
        haiku: { name: 'Claude Haiku 4.5', description: 'Plus rapide et plus économique.' },
      },
      model_load_error: 'Impossible de charger le modèle configuré.',
      model_save_error: "Impossible d'enregistrer le modèle sélectionné.",
      test: {
        title: 'Tester la connexion',
        description: 'Vérifie que la clé et le modèle sélectionné répondent correctement.',
        success: 'Connexion OK · {latency} ms',
        failed: 'Échec',
        running: 'Test en cours…',
        button: 'Tester',
      },
    },
    api_key: {
      label: 'Clé API Anthropic',
      status_unavailable: 'État de la clé indisponible',
      none: 'Aucune clé',
      configured: 'Configurée · {key}',
      source_keyring: 'Stockée dans le trousseau du système, jamais dans un fichier.',
      source_env: 'Lue depuis le fichier .env (mode DEV) : modifie ANTHROPIC_API_KEY pour la changer.',
      replace: 'Remplacer',
      delete_title: 'Supprimer la clé API',
      delete_body:
        "La clé {key} sera retirée du trousseau du système. L'IA restera indisponible jusqu'à la saisie d'une nouvelle clé.",
    },
    fonts: {
      section: 'Polices',
      description: 'Polices supplémentaires pour tes modèles de CV et de lettre.',
      imported: 'Police importée. À écrire dans font: {families}.',
      name_hint: 'Nom à écrire dans font:',
      shadowed_warning: 'Déjà fournie par le système ou Typst : seules les variantes manquantes seront utilisées.',
      file_count: '{n} fichier | {n} fichiers',
      empty: 'Aucune police importée. Les polices du système et de Typst restent disponibles.',
      help: 'Télécharge une police sur {link}, décompresse le ZIP puis sélectionne son dossier (sous-dossiers compris). Formats : .ttf, .otf, .ttc, .otc. Utilise le nom affiché dans {font}.',
      importing: 'Import…',
      add: '+ Ajouter une police',
      delete_title: 'Supprimer la police',
      delete_body: 'sera supprimée définitivement. | seront supprimées définitivement.',
    },
    appearance: {
      section: 'Interface',
      theme_label: 'Thème',
      theme_help: 'Système suit le réglage de ton ordinateur.',
      theme: {
        clair: 'Clair',
        sombre: 'Sombre',
        systeme: 'Système',
      },
    },
    language: {
      label: "Langue de l'interface",
      help: "Sans effet sur la langue du CV et de la lettre, qui suit celle de l'offre.",
      save_error: "Impossible d'enregistrer la langue de l'interface.",
    },
  },

  // ======================================================================
  // Onboarding
  // ======================================================================
  onboarding: {
    welcome: 'Bienvenue dans FindMyJob',
    intro: 'Configurons ton profil pour commencer.',
    progress: 'Étape {current} sur {total} — {name}',
    steps: {
      language: 'Langue',
      identity: 'Identité',
      contact: 'Contact',
      location: 'Localisation',
      about: 'À propos',
      links: 'Liens',
    },
    language: {
      label: "Langue de l'interface",
      hint: "Modifiable à tout moment dans les Paramètres. Les CV et lettres sont toujours rédigés dans la langue de l'offre.",
    },
    fields: {
      first_name: 'Prénom',
      last_name: 'Nom',
      email: 'E-mail',
      phone: 'Téléphone',
      driving_licence: 'Permis de conduire',
      about: 'À propos de toi',
    },
    placeholders: {
      first_name: 'Jean',
      last_name: 'Dupont',
      email: "jean.dupont{'@'}email.com",
      phone: '+33 6 12 34 56 78',
      about: "Étudiant·e en M2 Data Science à la recherche d'un stage…",
      linkedin: 'linkedin.com/in/jean-dupont',
      github: 'github.com/jeandupont',
      portfolio: 'jeandupont.dev',
    },
    creating: 'Création…',
    finish: 'Terminer',
  },
}
