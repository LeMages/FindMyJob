/**
 * Types partagés du frontend : navigation, DTOs miroir des commandes Tauri
 * (`src-tauri/src/models/`), et état d'UI (kanban, popups).
 */

// ── Navigation ────────────────────────────────────────────────────────────────

export type Screen = 'tracking' | 'detail' | 'new' | 'profile' | 'settings'

// ── Utilisateur / profil ─────────────────────────────────────────────────────

export interface UpdateUserInfoInput {
  prenom: string | null
  nom: string | null
  email: string | null
  telephone: string | null
  adresse: string | null
  code_postal: string | null
  permis: boolean | null
  linkedin: string | null
  github: string | null
  portfolio: string | null
  /** Centres d'intérêt, une ligne = un item. */
  interets: string | null
}

export interface CreateUserInput {
  nom: string
  prenom: string
  email: string
  telephone: string | null
  ville: string | null
  permis: boolean | null
  description: string | null
  linkedin: string | null
  github: string | null
  portfolio: string | null
}

export interface UserView {
  id: number
  nom: string | null
  prenom: string | null
  email: string | null
  telephone: string | null
  ville: string | null
  adresse: string | null
  code_postal: string | null
  permis: boolean | null
  description: string | null
  linkedin: string | null
  github: string | null
  portfolio: string | null
  /** Centres d'intérêt, une ligne = un item. */
  interets: string | null
  created_at: string
}
// ── Suivi de candidatures (kanban) ───────────────────────────────────────────

export type PopupKey = 'mail' | 'addRecruiter' | 'offer' | 'statusHistoryContent' | 'confirmDelete' | 'apply' | null

export interface BoardCard {
  id: number
  company: string
  role: string
  location: string
  date: string
  meta: string
}

export type SessionStatut = 'active' | 'pause' | 'termine'

export interface Session {
  id: number
  nom: string
  statut: SessionStatut
  candidature_count: number
  has_accepted: boolean
  created_at: string
}

export interface UpdateSessionInput {
  nom: string
  statut: SessionStatut
}

export interface NewContact {
  id: number
}

// ── Profil : expériences, compétences, langues, formations, projets ──────────

export interface ExperienceView {
  id: number
  poste: string
  entreprise: string | null
  lieu: string | null
  description: string | null
  date_debut: string | null
  date_fin: string | null
  type_contrat: string | null
}

export interface CreateExperienceInput {
  poste: string
  entreprise: string | null
  lieu: string | null
  description: string | null
  date_debut: string | null
  date_fin: string | null
  type_contrat: string | null
}

export interface SkillView {
  id: number
  domaine: string
  nom: string
}

export interface CreateSkillInput {
  nom: string
  domaine: string
}

export interface LangView {
  id: number
  nom: string | null
  level: string | null
}

export interface CreateLangInput {
  nom: string
  level: string | null
}

export interface FormationView {
  id: number
  titre: string
  ecole: string | null
  /** Détails (spécialité, cours, échange...), une ligne = un détail. */
  description: string | null
  date_debut: string | null
  date_fin: string | null
}

export interface CreateFormationInput {
  titre: string
  ecole: string | null
  /** Détails (spécialité, cours, échange...), une ligne = un détail. */
  description: string | null
  date_debut: string | null
  date_fin: string | null
}

export interface ProjetView {
  id: number
  titre: string
  description: string | null
  stack: string | null
  url: string | null
  pour_qui: string | null
  date_debut: string | null
  date_fin: string | null
}

export interface CreateProjetInput {
  titre: string
  description: string | null
  stack: string | null
  url: string | null
  pour_qui: string | null
  date_debut: string | null
  date_fin: string | null
}

// ── Documents (CV / lettre de motivation) ────────────────────────────────────

export interface DocumentReferenceView {
  file_path: string
  pdf_path: string
  updated_at: string
}

/** Version d'un document de référence d'une session (voir `DocumentVersionView` côté Rust). */
export interface DocumentVersionView {
  id: number
  session_id: number
  doc_type: 'cv' | 'lettre_motivation'
  version: number
  original_filename: string
  typst_path: string
  pdf_path: string
  created_at: string
  /** Vrai si c'est la version utilisée pour les prochaines générations. */
  is_current: boolean
}

export interface DocumentsStatus {
  cv_ready: boolean
  lm_ready: boolean
  all_ready: boolean
}

export interface ProfileDto {
  user: UserView
  experiences: ExperienceView[]
  formations: FormationView[]
  projets: ProjetView[]
  skills: SkillView[]
  langues: LangView[]
}

// ── Candidatures ──────────────────────────────────────────────────────────────

export interface CandidatureView {
  id: number
  session_id: number | null
  poste: string
  entreprise: string | null
  localisation: string | null
  status: string
  source: string | null
  type_contrat: string | null
  derniere_maj_statut: string
  cv_path: string | null
  lettre_motivation_path: string | null
  created_at: string
}

/** Progression de la génération des documents (CV + lettre de motivation) d'une candidature, consultée par polling. */
export interface GenerationProgress {
  candidature_id: number
  current_step: number
  total_steps: number
  step_label: string
  status: 'in_progress' | 'completed' | 'failed'
  error: string | null
}

export interface CreateCandidatureInput {
  poste: string
  entreprise: string | null
  localisation: string | null
  type_contrat: string | null
  source: string | null
  remuneration: string | null
  url_offre: string | null
  description_offre: string | null
  notes: string | null
  cv_path: string | null
  lettre_motivation_path: string | null
  session_id: number | null
}

/** Champs extraits d'une offre scrapée (voir `scrape_offer`), pour pré-remplir le formulaire. */
export interface ScrapedOfferFields {
  poste: string | null
  entreprise: string | null
  localisation: string | null
  type_contrat: string | null
  remuneration: string | null
  description_offre: string | null
}

/** Candidature existante pointant vers la même offre (miroir de `DuplicateCandidatureView`). */
export interface DuplicateCandidatureView {
  id: number
  session_id: number
  session_nom: string
  poste: string
  entreprise: string | null
  status: string
}

/** Raison d'échec typée de la récupération/extraction d'une offre (miroir de `ScrapingFailure`). */
export type ScrapingFailure =
  | 'invalid_url'
  | 'offer_unavailable'
  | 'access_denied'
  | 'timeout'
  | 'network'
  | 'not_html'
  | 'empty_page'
  | 'too_large'
  | 'insufficient_content'

/** Étage de la cascade d'extraction qui a produit les champs (miroir de `ExtractionStage`). */
export type ExtractionStage = 'json_ld' | 'llm'

/** Résultat d'une extraction d'offre (miroir de `OfferExtraction`). */
export interface OfferExtraction {
  fields: ScrapedOfferFields
  stage: ExtractionStage
}

export interface UpdateCandidatureInput {
  poste: string
  entreprise: string | null
  localisation: string | null
  type_contrat: string | null
  source: string | null
  remuneration: string | null
  url_offre: string | null
  notes: string | null
}

export interface CandidatureDetail {
  id: number
  session_id: number | null
  poste: string
  entreprise: string | null
  localisation: string | null
  status: string
  source: string | null
  type_contrat: string | null
  remuneration: string | null
  url_offre: string | null
  description_offre: string | null
  notes: string | null
  cv_path: string | null
  lettre_motivation_path: string | null
  /** Numéro de la version du CV de référence utilisée pour la génération. */
  cv_reference_version: number | null
  /** Numéro de la version de la lettre de référence utilisée pour la génération. */
  lm_reference_version: number | null
  created_at: string
}

export interface ContactView {
  id: number
  candidature_id: number
  nom: string | null
  role: string | null
  email: string | null
  telephone: string | null
  notes: string | null
  created_at: string
}

export interface StatusHistoryView {
  id: number
  candidature_id: number
  event_type: string
  ancien_status: string | null
  /** `null` si l'événement ne change pas le statut (message, appel, note...). */
  nouveau_status: string | null
  content: string | null
  interview_id: number | null
  changed_at: string
}

/** Événement proposable pour le statut courant d'une candidature (voir `get_available_events`). */
export interface AvailableEvent {
  event_type: string
  label: string
  changes_status: boolean
  target_status: string | null
}

/** Entretien rattaché à une candidature. */
export interface InterviewView {
  id: number
  candidature_id: number
  type: string | null
  date_entretien: string | null
  lieu: string | null
  issue: string
  notes: string | null
  contact_id: number | null
}

/** Données de l'entretien à créer, pour un événement `entretien_planifie`. */
export interface NewInterviewData {
  type: string
  date_entretien: string | null
  lieu: string | null
  modalite: string | null
  contact_id: number | null
}

/** Entretien enrichi de sa candidature, pour la vue planning. */
export interface PlanningInterviewView {
  id: number
  candidature_id: number
  entreprise: string | null
  poste: string
  type: string | null
  date_entretien: string | null
  lieu: string | null
  modalite: string | null
  issue: string
  notes: string | null
}

/** Entrée soumise pour ajouter un événement à la timeline d'une candidature. */
export interface NewTimelineEvent {
  event_type: string
  content: string | null
  interview_data: NewInterviewData | null
  interview_id: number | null
  interview_issue: string | null
}

export interface DocumentFileInfo {
  exists: boolean
  size: number
}

// ── Paramètres ────────────────────────────────────────────────────────────────

/** Thème d'affichage de l'application. */
export type ThemeSetting = 'clair' | 'sombre' | 'systeme'

/** Résultat d'un test de connexion à l'API Claude. */
export interface TestConnectionResult {
  success: boolean
  model: string
  message: string
  latency_ms: number
}

// ── Polices ───────────────────────────────────────────────────────────────────

/** Famille de polices importée par l'utilisateur pour la compilation Typst. */
export interface FontFamilyView {
  name: string
  file_count: number
}
