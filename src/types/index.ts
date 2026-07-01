export type Screen = 'tracking' | 'detail' | 'new' | 'profile' | 'settings'

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
export type PopupKey = 'mail' | 'addRecruiter' | 'offer' | 'addEvent' | null
export type EventType = 'mail_relance' | 'mail_reponse' | 'entretien'

export interface BoardCard {
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

export interface Session {
  id: number
  nom: string
  candidature_count: number
  has_accepted: boolean
  created_at: string
}

export interface Recruiter {
  id: number
  nom: string
  poste: string
  email: string
}

export interface AppEvent {
  id: number
  type: EventType
}

export interface NewContact {
  id: number
}

export interface DragState {
  fromColIndex: number
  cardIndex: number
}

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

export interface CvView {
  id: number
  label: string | null
  file_path: string
  pdf_path: string | null
  created_at: string
}

export interface ProfileDto {
  user: UserView
  experiences: ExperienceView[]
  formations: FormationView[]
  projets: ProjetView[]
  skills: SkillView[]
  langues: LangView[]
  cv_reference: CvView | null
}

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
  cv_id: number | null
  session_id: number | null
}

export interface DragPopupState {
  card: BoardCard
  fromColTitle: string
  toColTitle: string
  fromColIndex: number
  toColIndex: number
  cardIndex: number
}
