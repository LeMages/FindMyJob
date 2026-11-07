/**
 * Textes de l'interface en anglais. Même forme exacte que `fr.ts` (langue de
 * référence) : une clé manquante ou en trop est une erreur `vue-tsc`.
 */

// Importation des modules internes
import type { fr } from '@/locales/fr'

export const en = {
  // ======================================================================
  // Éléments communs
  // ======================================================================
  common: {
    loading: 'Loading…',
    save: 'Save',
    saving: 'Saving…',
    delete: 'Delete',
    cancel: 'Cancel',
    irreversible: 'This cannot be undone.',
    yes: 'Yes',
    no: 'No',
    previous: '← Back',
    next: 'Next →',
    close: 'Close',
    edit: 'Edit',
    validate: 'Confirm',
    add: '+ Add',
    not_specified: 'Not specified',
    select_placeholder: '-- Select --',
    back: '← Back',
    unknown_company: 'Unknown company',
    date_missing: 'No date set',
    date: {
      at: 'at',
      present: 'Present',

    },
    months: {
      short: {
        jan: 'Jan',
        feb: 'Feb',
        mar: 'Mar',
        apr: 'Apr',
        may: 'May',
        jun: 'Jun',
        jul: 'Jul',
        aug: 'Aug',
        sep: 'Sep',
        oct: 'Oct',
        nov: 'Nov',
        dec: 'Dec',
      },
    },
    weekdays: {
      short: {
        mon: 'Mon',
        tue: 'Tue',
        wed: 'Wed',
        thu: 'Thu',
        fri: 'Fri',
        sat: 'Sat',
        sun: 'Sun',
      },
    },
    file_size: {
      b: 'B',
      kb: 'KB',
      mb: 'MB',
    },
  },

  // ======================================================================
  // Erreurs
  // ======================================================================
  error: {
    kind: {
      Database: 'Database error.',
      Validation: 'Invalid input.',
      Conflict: 'Conflicts with existing data.',
      NotFound: 'Not found.',
      Config: 'Configuration error.',
      Llm: 'AI error.',
      Scraping: 'Could not extract the job offer.',
      OutdatedReference: 'Outdated reference document: please re-import it.',
    },
    fallback: {
      startup: 'Could not start the app.',
      unexpected: 'An unexpected error occurred.',
      create_session: 'Could not create the session.',
      update_session: 'Could not update the session.',
      load_session_documents: 'Could not load the session documents.',
      load_versions: 'Could not load the version history.',
      restore_version: 'Could not restore this version.',
      load_version_preview: 'Could not load the preview of this version.',
      save_cv: 'Error while saving the CV.',
      save_cover_letter: 'Error while saving the cover letter.',
      compile_cv: 'Error while compiling the CV.',
      compile_cover_letter: 'Error while compiling the cover letter.',
      load_preview: 'Could not load the document preview.',
      download_document: 'Could not download the document.',
      download_application_documents: 'Could not download the application documents.',
      load_fonts: 'Could not load fonts.',
      import_font: 'Could not import this font folder.',
      delete_font: 'Could not delete this font.',
      generation_tracking: 'Error while tracking document generation.',
      save_theme: 'Could not save the theme.',
      load_application: 'Could not load the application.',
      load_applications: 'Could not load applications.',
      create_application: 'Error while creating the application.',
      update_application: 'Could not update the application.',
      delete_application: 'Could not delete the application.',
      add_event: 'Could not add the event.',
      load_available_events: 'Could not load available events.',
      load_pending_interviews: 'Could not load pending interviews.',
      load_interviews: 'Could not load interviews.',
      load_contacts: 'Could not load the address book.',
      link_contact: 'Could not link the contact.',
      create_contact: 'Could not create the contact.',
      update_contact: 'Could not update the contact.',
      unlink_contact: 'Could not remove the contact from the application.',
      delete_contact: 'Could not delete the contact.',
      apply_without_message: 'Could not apply without a message.',
      apply_by_email: 'Could not apply by email.',
      apply_by_message: 'Could not apply by message.',
      analyze_offer: 'Job offer analysis failed.',
      check_duplicates: 'Could not check for duplicates.',
      open_offer_page: 'Could not open the job offer page.',
      capture_failed: 'Page capture failed.',
      extract_from_text: 'Could not extract information from this text.',
      close_capture_window: 'Could not close the page window.',
      open_link: 'Could not open this link.',
      save_profile: 'Error while saving.',
      load_profile: 'Could not load the profile.',
      read_api_key: 'Could not read the API key status.',
      save_api_key: 'Could not save the API key.',
      delete_api_key: 'Could not delete the API key.',
      connection_test: 'Connection test failed.',
    },
  },

  // ======================================================================
  // Toasts
  // ======================================================================
  toast: {
    document_downloaded: 'Document downloaded: {path}',
    documents_downloaded: 'Documents downloaded to {dir}',
  },

  // ======================================================================
  // Candidatures : statuts, contrats, sources
  // ======================================================================
  status: {
    a_postuler: 'To apply',
    postule: 'Application sent',
    relance: 'In discussion',
    entretien: 'Interview',
    entretien_passe: 'Awaiting answer',
    offre: 'Offer',
    refus: 'Rejected',
    abandon: 'Withdrawn',
    ghosting: 'Ghosted',
    accepte: 'Accepted',
    offre_refusee: 'Offer declined',
  },
  contract: {
    stage: 'Internship',
    alternance: 'Apprenticeship',
    interim: 'Temp work',
    cdd: 'Fixed-term contract',
    cdi: 'Permanent contract',
  },
  source: {
    online_platform: 'Online platform (Indeed, LinkedIn, Welcome to the Jungle...)',
    company_site: 'Company careers site (form on the website)',
    email: 'Email (direct application)',
    referral: 'Referral / Network (recommendation, personal contact)',
    job_fair: 'Job fair (in person)',
    school: 'School (job board, offers from teachers / clubs)',
    other: 'Other',
  },

  // ======================================================================
  // Timeline de suivi
  // ======================================================================
  event: {
    history: {
      creation: 'Application created',
      candidature_envoyee: 'Application sent',
      relance: 'Follow-up sent',
      entretien_planifie: 'Interview scheduled',
      entretien_passe: 'Interview done',
      offre_recue: 'Offer received',
      refus_recu: 'Rejection received',
      ghosting: 'Ghosting',
      desistement: 'Withdrawal',
      accepter_offre: 'Offer accepted',
      decliner_offre: 'Offer declined',
      message_envoye: 'Message sent',
      message_recu: 'Message received',
      appel: 'Call',
      note: 'Note',
    },
    action: {
      message_envoye: 'Message sent',
      message_recu: 'Message received',
      appel: 'Call',
      note: 'Note',
      relance: 'Follow-up',
      entretien_planifie: 'Interview scheduled',
      entretien_passe: 'Interview done',
      offre_recue: 'Offer received',
      refus_recu: 'Rejection received',
      ghosting: 'Ghosting',
      accepter_offre: 'Accept the offer',
      decliner_offre: 'Decline the offer',
      desistement: 'Withdraw',
    },
    category: {
      communication: 'Communication',
      interview: 'Interview',
      follow_up: 'Follow-up',
      outcome: 'Outcome',
      other: 'Other',
    },
  },
  interview: {
    type_default: 'Interview',
    type: {
      rh: 'HR',
      technique: 'Technical',
      manager: 'Manager',
      final: 'Final',
      autre: 'Other',
    },
    issue: {
      positif: 'Positive',
      negatif: 'Negative',
      pending: 'Pending',
    },
    detail: {
      modality: 'Format',
      modality_missing: 'Not specified',
      location: 'Location',
      contacts: 'Interviewers',
      notes: 'Notes',
    },
    modalite: {
      visio: 'Video call',
      presentiel: 'In person',
      telephone: 'Phone',
    },
  },

  // ======================================================================
  // Sessions, contacts, profil
  // ======================================================================
  session: {
    params: {
      section: 'GENERAL INFORMATION',
      title: 'Session title',
      status: 'Session status',
    },
    create: {
      title: 'New session',
      intro: 'A session groups a set of applications (e.g. “MSc internship – 2025”).',
      name: 'Session name',
      placeholder: 'e.g. MSc internship – 2025',
      submit: 'Create the session',
    },
    delete: {
      title: 'Delete the session',
      body: '{name} and all its applications will be permanently deleted.',
    },
    status: {
      active: 'Active',
      pause: 'Paused',
      termine: 'Finished',
    },
  },
  contacts: {
    entry_title: 'CONTACT {n}',
    empty_new: 'No contact for this application.',
    add: '+ Add a contact',
    errors: {
      pick_from_book: 'Pick a contact from the address book.',
    },
  },
  profile: {
    title: 'My profile',
    subtitle: 'This information pre-fills your applications.',
    driving_licence: 'Driving licence',
    saving: 'Saving…',
    pending: {
      edited: 'Edited',
      new: 'Not saved',
    },
    bullet: {
      add: 'Add a bullet',
      label: 'Bullet',
    },
    in_progress: 'Ongoing',
    start_year: 'Start year',
    end_year: 'End year',
    description: 'Description',
    info: {
      section: 'INFORMATION',
      first_name: 'First name',
      last_name: 'Last name',
      email: 'Email address',
      phone: 'Phone',
      city: 'City',
      zip_code: 'Postcode',
      interests: 'Interests (one per line)',
      profile_language: 'Language the profile is written in',
      lang_fr: 'French',
      lang_en: 'English',
    },
    placeholders: {
      first_name: 'First name',
      last_name: 'Last name',
      email: "firstname.lastname{'@'}email.com",
      phone: '+44 7700 900000',
      city: 'London',
      zip_code: 'SW1A 1AA',
      interests: 'Football',
      portfolio: 'https://mysite.com',
      about: 'Describe your background and goals…',
    },
    links: {
      section: 'LINKS',
      invalid_linkedin: 'The LinkedIn link is not a valid URL.',
      invalid_github: 'The GitHub link is not a valid URL.',
      invalid_portfolio: 'The portfolio link is not a valid URL.',
    },
    about: {
      section: 'ABOUT',
    },
    experiences: {
      section: 'EXPERIENCE',
      empty: 'No experience added',
      add_title: 'Add an experience',
      edit_title: 'Edit an experience',
      job_title: 'Job title *',
      job_placeholder: 'Full-Stack Developer',
      location_placeholder: 'London, United Kingdom',
      start: 'Start',
      end: 'End',
      description_placeholder: 'Describe your tasks, the technologies you used…',
    },
    formations: {
      section: 'EDUCATION',
      empty: 'No education added',
      add_title: 'Add an education entry',
      edit_title: 'Edit an education entry',
      title: 'Title / Degree *',
      title_placeholder: 'MSc Computer Science',
      school: 'School / Institution',
      school_placeholder: 'University of Manchester',
      details: 'Details (one per line)',
      details_placeholder: 'Specialisation, key courses, exchange programme...',
    },
    projects: {
      section: 'PROJECTS',
      empty: 'No projects added',
      add_title: 'Add a project',
      edit_title: 'Edit a project',
      title: 'Title *',
      title_placeholder: 'My great project',
      for_whom: 'For whom',
      for_whom_placeholder: 'Personal, School, Client...',
      stack: 'Tech stack',
      link: 'Link',
      description_placeholder: 'This project is about...',
    },
    skills: {
      section: 'SKILLS',
      empty: 'No skills added',
      add_title: 'Add a skill',
      edit_title: 'Edit a skill',
      name: 'Skill name *',
      domain: 'Category *',
      domain_placeholder: 'Languages, Frameworks, Tools…',
      create: 'Create {name}',
      quoted: '“{name}”',
      no_domain: 'No existing category',
    },
    languages: {
      section: 'LANGUAGES',
      empty: 'No languages added',
      add_title: 'Add a language',
      edit_title: 'Edit a language',
      name: 'Language *',
      name_placeholder: 'English, Spanish…',
      level_label: 'Level',
      level: {
        native: 'Native',
        bilingual: 'Bilingual',
        c2: 'Fluent (C2)',
        c1: 'Advanced (C1)',
        b2: 'Upper intermediate (B2)',
        b1: 'Intermediate (B1)',
        a2: 'Elementary (A2)',
        a1: 'Beginner (A1)',
      },
    },
  },

  // ======================================================================
  // Import d'offre
  // ======================================================================
  offer_import: {
    failure: {
      invalid_url: 'This link is not valid: only public http or https web addresses are accepted.',
      offer_unavailable: 'This job offer cannot be found or has expired: the page no longer exists on the site.',
      access_denied: 'This site blocks automated reading of its pages.',
      timeout: 'The site did not respond in time.',
      network: 'The site could not be reached.',
      not_html: 'This link does not lead to a web page (PDF file or other document).',
      empty_page: 'The retrieved page is empty.',
      too_large: 'The page is too large to analyse (over 5 MB).',
      insufficient_content:
        'The retrieved page does not contain the job offer text (content loaded with JavaScript, anti-bot check or cookie banner).',
    },
  },

  // ======================================================================
  // Détail d'une candidature
  // ======================================================================
  detail: {
    back_to_tracking: '← Back to tracking',
    job: {
      section: 'JOB DETAILS',
      view_offer: 'View offer →',
      title: 'Job title',
      title_placeholder: 'e.g. Data Scientist internship',
      company: 'Company',
      company_placeholder: 'Company name',
      location: 'Location',
      location_placeholder: 'City',
      contract: 'Contract type',
      salary: 'Pay',
      salary_placeholder: 'e.g. €1,300/month',
      platform: 'Offer platform',
      link: 'Offer link',
    },
    notes: {
      section: 'NOTES',
      placeholder: 'Free notes, interview feedback, contacts...',
    },
    documents: {
      section: 'DOCUMENTS',
      cv: 'CV',
      cover_letter: 'Cover letter',
      attached: 'Attached · {size}',
      missing: 'Not available',
      cv_generated_with: 'Generated with reference v{version}',
      lm_generated_with: 'Generated with reference v{version}',
      preview: 'Preview',
      download: 'Download',
      cv_preview_title: 'CV preview',
      lm_preview_title: 'Cover letter preview',
      warnings_title: 'Generation warnings',
    },
    contacts: {
      section: 'RECRUITER CONTACTS',
      unlink: 'Remove from application',
      delete: 'Delete permanently',
      empty: 'No contacts yet.',
    },
    footer: {
      accepted: '✓ Application accepted — congratulations!',
      closed: 'Application closed — {status}.',
      apply: 'Apply',
    },
    apply: {
      title: 'Apply',
      question: 'How do you want to apply to this offer?',
      mail_title: 'Apply by email',
      mail_writing: 'AI is writing the email…',
      mail_description: 'Write and send an application email.',
      message_title: 'Apply by message',
      message_writing: 'AI is writing the message…',
      message_description: "Send a message through the offer's platform.",
      none_title: 'Apply without a message',
      none_description: 'Mark the application as sent, without a message.',
    },
    mail: {
      title: 'Write the application email',
      subject_label: 'Subject',
      subject: 'Application — {role} — Alex Martin',
      body_label: 'Email body',
      body_placeholder: 'Dear Sir or Madam, ...',
      generate: '✦ Generate with AI',
    },
    delete_contact: {
      title: 'Delete contact permanently',
      body: '{name} will be removed from the address book, from this application and from its interviews.',
      also_linked:
        'They are also linked to {n} other application, from which they will also be removed. | They are also linked to {n} other applications, from which they will also be removed.',
      confirm: 'Delete permanently',
    },
    offer: {
      title: 'Offer content',
      empty: 'No offer content provided.',
    },
    sent_content: {
      title: 'Sent content',
    },
    delete_application: {
      title: 'Delete application',
      body: 'This application will be permanently removed from your tracking. This cannot be undone.',
    },
    timeline: {
      section: 'STATUS & TRACKING',
      view_content: 'View content',
      empty: 'No tracking yet.',
      add_event: '+ Add an event',
    },
    add_event: {
      title: 'Add an event',
      empty: 'No event available for this status.',
      interview_type: 'Interview type',
      date_time: 'Date and time',
      location: 'Location (optional)',
      location_placeholder: 'Video call, company offices...',
      modality: 'Format (optional)',
      modality_unspecified: 'Not specified',
      contacts: 'Interviewers (optional)',
      no_contacts: 'No contact linked to this application.',
      interview: 'Interview',
      pick_interview: 'Choose the past interview',
      issue: 'Outcome',
      content: {
        relance: 'Message content',
        message: 'Content',
        appel: 'Call notes',
        optional_notes: 'Notes (optional)',
        decline_reason: 'Reason for declining (optional)',
        note: 'Note',
        default: 'Notes',
      },
    },
  },

  // ======================================================================
  // Nouvelle candidature
  // ======================================================================
  new_application: {
    title: 'New application',
    no_session: 'No job search session is selected: create or choose a session in tracking before adding an application.',
    go_to_tracking: 'Go to tracking',
    go_to_settings: 'Go to settings',
    back_to_offer_link: '← Back to the offer link',
    back_to_link: '← Back to the link',
    analyzing: 'Analysing…',
    manual: 'Manual entry',
    submit: 'Create the application →',
    form: {
      location_placeholder: 'City, country',
      offer_placeholder: 'Paste the full text of the job offer here...',
      notes_placeholder: 'Free notes, context, things to prepare...',
    },
    link: {
      section: 'OFFER LINK',
      intro: 'Paste the offer link to fill in the form automatically, or go straight to manual entry.',
      unavailable_hint: 'Check the link or choose another offer.',
      placeholder: 'https://www.example.com/jobs/...',
      analyze: 'Analyse the offer →',
    },
    capture: {
      section: 'OFFER PAGE',
      intro:
        'The offer page is open in a FindMyJob window. Accept cookies or sign in if the site asks, wait until the offer text is displayed, then capture the page. Your session on this site is kept for next time.',
      capture: 'Capture this page',
      reopen: 'Reopen the window',
      paste: 'Paste the text',
    },
    paste: {
      section: 'OFFER TEXT',
      intro: 'Select all the text of the offer page and paste it below.',
      label: 'Offer text',
      analyze: 'Analyse the text →',
    },
    choice: {
      section: 'AUTOMATIC ANALYSIS NOT POSSIBLE',
      prompt: 'Choose how to continue:',
      capture: 'Open the page in FindMyJob',
      paste: 'Paste the offer text',
    },
    duplicates: {
      title: 'You already have an application for this offer: | You already have {n} applications for this offer:',
      item: '({status}, session “{session}”)',
      hint: 'You can still create this one.',
    },
  },

  // ======================================================================
  // Génération des documents
  // ======================================================================
  generation: {
    title: 'Generating documents',
    starting: 'Starting…',
    step_counter: 'Step {current} / {total}',
    failed: 'Document generation failed.',
    step: {
      '0': 'Preparing…',
      '1': 'Analysing the offer…',
      '2': 'Matching profile and offer…',
      '3': 'Writing the CV and cover letter…',
      '4': 'Building and compiling the CV…',
      '5': 'Building and compiling the cover letter…',
    },
  },

  // ======================================================================
  // Suivi et planning
  // ======================================================================
  tracking: {
    create_first_session: '+ Create my first session',
    summary: '{n} application · last updated {date} | {n} applications · last updated {date}',
    trash: 'Trash',
    documents_blocking_hint: '{message} Add them in the Settings tab.',
    new_application: '+ New application',
    columns: {
      company: 'COMPANY',
      role: 'POSITION',
      status: 'STATUS',
      applied: 'APPLIED',
      inactivity: 'INACTIVITY',
    },
    inactivity_days: '{n} d',
    empty: 'No applications',
    tabs: {
      applications: 'Applications',
      planning: 'Planning',
      settings: 'Settings',
    },
    trash_popup: {
      intro: 'Withdrawn applications for this session.',
      abandoned_on: 'Withdrawn on {date}',
      empty: 'No applications in the trash.',
    },
  },
  planning: {
    count: '{n} interview · {range} | {n} interviews · {range}',
    week: 'Week',
    month: 'Month',
    previous: 'Previous period',
    today: 'Today',
    next: 'Next period',
    empty: 'No interviews',
  },

  // ======================================================================
  // Documents de référence
  // ======================================================================
  documents: {
    reference: {
      empty_cv: 'No CV provided',
      empty_lm: 'No cover letter provided',
      saved_cv: 'Saved on {date}',
      saved_lm: 'Saved on {date}',
      preview_cv: 'View the compiled CV',
      preview_lm: 'View the compiled cover letter',
      add_cv: 'Add a CV',
      replace_cv: 'Replace the CV',
      add_lm: 'Add a cover letter',
      replace_lm: 'Replace the cover letter',
      iframe_cv: 'CV PDF preview',
      iframe_lm: 'Cover letter PDF preview',
      with_version: '{title} — version {version}',
      only_typ: 'Only .typ files are accepted.',
      versions: 'Versions',
      compiling: 'Compiling…',
    },
    versions: {
      title_cv: 'Versions – CV',
      title_lm: 'Versions – Cover letter',
      empty: 'No saved versions',
      current: 'current',
      preview: 'View version {version}',
      restore_hint: 'Use this version for the next generations',
      restoring: 'Restoring…',
      restore: 'Restore',
    },
    preview: {
      downloading: 'Downloading…',
      pdf_title: 'PDF preview',
    },
    missing: {
      message: 'Reference document(s) missing or not analysed: {list}.',
      cv: 'the CV',
      lm: 'the cover letter',
      both: 'the CV and the cover letter',
    },
    markup: {
      summary: 'Instruction markup',
      intro: 'Instructions are written as {comment} comments in the .typ file; they do not appear in the PDF.',
      instruction: 'instruction',
      general: '{tag} followed by {comment} lines: general instructions, up to the first line that is not a comment.',
      fixed: '{tag}: the following text is copied word for word (faithfully translated if the document is generated in another language).',
      variable: '{tag}: the following text is rewritten based on the offer.',
      adaptable: '{tag}: the following text is rewritten based on your profile, without inventing anything.',
      zones:
        'A zone runs until the next tag or the next blank line. Tags apply to written text: subject and paragraphs of the letter, title and summary of the CV. Written text without a tag is treated as {tag}.',
      must_open: 'The tag must open the comment: {example} is rejected.',
      example_heading: 'Body',
    },
  },

  // ======================================================================
  // Contacts, champs éditables, barre latérale, démarrage
  // ======================================================================
  contact_form: {
    existing: 'Existing contact',
    new: 'New contact',
    from_book: 'Contact from the address book',
    pick: 'Choose a contact…',
    name: 'Name',
    name_placeholder: 'First and last name',
    role: 'Role',
    role_placeholder: 'e.g. Recruiter',
    email: 'Email',
    email_placeholder: "contact{'@'}company.com",
    phone: 'Phone',
    phone_placeholder: '07700 900000',
    notes: 'Notes',
    notes_placeholder: 'Met at the job fair, prefers to be contacted by email...',
    edit_title: 'Edit the contact',
    add_title: 'Add a contact',
    shared_warning:
      'This contact is linked to {n} application: the change applies everywhere. | This contact is linked to {n} applications: the change applies everywhere.',
  },
  editable_field: {
    locked: 'Editable only while the status is “To apply”',
    click_to_edit: 'Click to edit',
    double_click_to_edit: 'Double-click to edit',
  },
  sidebar: {
    sessions: 'MY SESSIONS',
    new_session: '+ New session',
    view_profile: 'View profile',
    settings: 'Settings',
  },
  startup: {
    failed: 'Could not start',
  },

  // ======================================================================
  // Paramètres
  // ======================================================================
  settings: {
    title: 'Settings',
    subtitle: 'FindMyJob app configuration.',
    ai: {
      section: 'ARTIFICIAL INTELLIGENCE',
      model: 'Model',
      models: {
        sonnet: 'Claude Sonnet 4.6 (recommended)',
        haiku: 'Claude Haiku 4.5 (fast, cheaper)',
      },
      test: {
        title: 'Test the connection',
        description: 'Checks that the key and the selected model respond correctly.',
        success: 'Connection successful ({latency}ms)',
        running: 'Testing…',
        button: 'Test →',
      },
    },
    api_key: {
      label: 'Anthropic API key',
      status_unavailable: 'Key status unavailable',
      none: 'No key configured',
      configured: 'Key configured (…{last4})',
      source_keyring: 'Stored in the system keyring.',
      source_env: 'Read from the .env file (DEV mode): edit ANTHROPIC_API_KEY to change it.',
      placeholder_replace: 'New key (replaces the current one)',
      delete_title: 'Delete the API key',
      delete_body: 'The key {key} will be removed from the system keyring. AI features will be unavailable until a new key is entered.',
    },
    fonts: {
      section: 'FONTS',
      imported: 'Font imported. Write in font: {families}.',
      name_hint: 'Name to write in font:',
      shadowed_warning:
        'This family is already provided by the system or by Typst: the imported version will only be used for the variants the other one lacks.',
      file_count: '{n} file | {n} files',
      empty: 'No imported fonts',
      help: 'Download a font from {link}, unzip it, then select its folder here (subfolders included). Accepted formats: .ttf, .otf, .ttc, .otc. Write the name shown above in {font}.',
      importing: 'Importing…',
      add: 'Add a font',
      delete_title: 'Delete the font',
      delete_body: 'will be permanently deleted. | will be permanently deleted.',
    },
    appearance: {
      section: 'APPEARANCE',
      theme_label: 'Theme',
      theme: {
        clair: 'Light',
        sombre: 'Dark',
        systeme: 'System',
      },
    },
    language: {
      section: 'LANGUAGE',
      label: 'Interface language',
      save_error: 'Could not save the interface language.',
    },
  },

  // ======================================================================
  // Onboarding
  // ======================================================================
  onboarding: {
    welcome: 'Welcome to FindMyJob',
    intro: "Let's set up your profile to get started.",
    progress: 'Step {current} of {total} — {name}',
    steps: {
      language: 'Language',
      identity: 'Identity',
      contact: 'Contact',
      location: 'Location',
      about: 'About',
      links: 'Links',
    },
    language: {
      label: 'Interface language',
      hint: 'You can change it anytime in Settings. CVs and cover letters are always written in the language of the job offer.',
    },
    fields: {
      first_name: 'First name',
      last_name: 'Last name',
      email: 'Email',
      phone: 'Phone',
      city: 'City',
      zip_code: 'Postcode',
      driving_licence: 'Driving licence',
      about: 'About you',
    },
    placeholders: {
      first_name: 'John',
      last_name: 'Smith',
      email: "john.smith{'@'}email.com",
      phone: '+44 7700 900123',
      city: 'London',
      zip_code: 'SW1A 1AA',
      about: 'MSc Data Science student looking for an internship…',
      linkedin: 'linkedin.com/in/john-smith',
      github: 'github.com/johnsmith',
      portfolio: 'johnsmith.dev',
    },
    creating: 'Creating…',
    finish: 'Finish',
  },
} satisfies typeof fr
