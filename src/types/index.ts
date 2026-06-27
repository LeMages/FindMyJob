export type Screen = 'tracking' | 'detail' | 'new' | 'profile' | 'settings'

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
  name: string
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

export interface DragPopupState {
  card: BoardCard
  fromColTitle: string
  toColTitle: string
  fromColIndex: number
  toColIndex: number
  cardIndex: number
}
