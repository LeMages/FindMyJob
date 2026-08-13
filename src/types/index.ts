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
  permis: boolean | null
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
  permis: boolean | null
  description: string | null
  linkedin: string | null
  github: string | null
  portfolio: string | null
  created_at: string
  updated_at: string
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

export interface BoardColumn {
  title: string
  cards: BoardCard[]
}

export type SessionStatut = 'active' | 'pause' | 'termine'

export interface Session {
  id: number
  nom: string
  contexte: string | null
  statut: SessionStatut
  zone: string | null
  candidature_count: number
  has_accepted: boolean
  created_at: string
}

export interface UpdateSessionInput {
  nom: string
  contexte: string | null
  statut: SessionStatut
  zone: string | null
}

export interface NewContact {
  id: number
}

export interface DragState {
  fromColIndex: number
  cardIndex: number
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
  date_debut: string | null
  date_fin: string | null
}

export interface CreateFormationInput {
  titre: string
  ecole: string | null
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
  date_candidature: string | null
  next_relance: string | null
  derniere_maj_statut: string
  cv_path: string | null
  lettre_motivation_path: string | null
  created_at: string
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
  date_candidature: string | null
  next_relance: string | null
  cv_path: string | null
  lettre_motivation_path: string | null
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

export interface DragPopupState {
  card: BoardCard
  fromColTitle: string
  toColTitle: string
  fromColIndex: number
  toColIndex: number
  cardIndex: number
}
