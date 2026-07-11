/**
 * Store applicatif global (état réactif partagé entre toutes les vues) :
 * navigation, sessions, candidature sélectionnée, popups. Les données de
 * candidatures/sessions viennent du backend ; le reste (recruteurs, corbeille)
 * est encore en mock en attendant leur implémentation backend.
 */

// Importation des modules Vue / framework
import { reactive, ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import type {
  Screen, PopupKey, EventType,
  BoardCard, Session,
  Recruiter, AppEvent, NewContact,
  DragState, DragPopupState,
} from '@/types'

// ---------------------------------------------------------------------------
// Sessions — chargées depuis le backend
// ---------------------------------------------------------------------------

const backendSessions = ref<Session[]>([])

// Recruteurs de démonstration — pas encore de commande Tauri dédiée, à
// remplacer quand la gestion des contacts recruteur sera implémentée côté backend.
const MOCK_RECRUITERS: Recruiter[] = [
  { id: 1, nom: 'Sophie Lecomte', poste: 'Tech Recruiter', email: 's.lecomte@criteo.com' },
]

// ---------------------------------------------------------------------------
// Reactive state — UI state always lives here; data will move to backend
// ---------------------------------------------------------------------------

const state = reactive({
  // Navigation
  screen: 'tracking' as Screen,

  // User identity (populated after profile load)
  userDisplayName: '',
  userInitials: '',

  // Sessions
  sessionId: 0,

  // Currently open application
  selectedCard: null as BoardCard | null,

  // Detail page state
  detailPopup: null as PopupKey,
  detailRecruiters: [...MOCK_RECRUITERS] as Recruiter[],
  detailEvents: [] as AppEvent[],
  nextEventId: 1,
  nextRecruiterId: 10,

  // Drag & drop
  dragState: null as DragState | null,
  dragPopup: null as DragPopupState | null,

  // Trash popup
  trashPopup: false,

  // Create session popup
  createSessionPopup: false,

  // Delete session popup
  deleteSessionPopup: null as { id: number; nom: string } | null,

  // New application form contacts
  newContacts: [{ id: 0 }] as NewContact[],
  nextContactId: 1,
})

// ---------------------------------------------------------------------------
// Computed values (these will eventually come from backend)
// ---------------------------------------------------------------------------

export const allSessions = computed<(Session & { count: number; hasAccepted: boolean })[]>(() =>
  backendSessions.value.map(s => ({
    ...s,
    count: s.candidature_count,
    hasAccepted: s.has_accepted,
  }))
)

export const currentSession = computed(() =>
  allSessions.value.find(s => s.id === state.sessionId) ?? allSessions.value[0] ?? null,
)

// Corbeille de session : pas encore de commande Tauri dédiée (candidatures
// refusées/abandonnées ne sont pas encore listées séparément côté backend).
export const currentTrashedList = computed<BoardCard[]>(() => [])

// Colonne du kanban où se trouve la candidature sélectionnée : à recalculer
// une fois le statut de candidature exposé par le backend au DetailView.
export const selectedCardColumn = computed<string>(() => '')

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/** Charge les sessions de l'utilisateur depuis le backend. */
export async function loadSessions() {
  try {
    backendSessions.value = await invoke<Session[]>('get_sessions')
    if (backendSessions.value.length > 0 && !backendSessions.value.find(s => s.id === state.sessionId)) {
      state.sessionId = backendSessions.value[0].id
    }
  } catch (e) {
    console.error('Impossible de charger les sessions', e)
  }
}

/**
 * Met à jour le nom et les initiales affichés dans la barre latérale.
 *
 * @param displayName - Nom complet à afficher.
 * @param initials - Initiales (fallback avatar).
 */
export function setUserInfo(displayName: string, initials: string) {
  state.userDisplayName = displayName
  state.userInitials = initials
}

/**
 * Change l'écran actif de l'application.
 *
 * @param screen - Écran à afficher.
 */
export function navigate(screen: Screen) {
  state.screen = screen
}

/**
 * Sélectionne une session et retourne à l'écran de suivi.
 *
 * @param id - Identifiant de la session à sélectionner.
 */
export function selectSession(id: number) {
  state.sessionId = id
  state.screen = 'tracking'
  state.selectedCard = null
}

/** Ouvre la popup de création de session. */
export function addSession() {
  state.createSessionPopup = true
}

/** Ferme la popup de création de session sans créer. */
export function cancelCreateSession() {
  state.createSessionPopup = false
}

/**
 * Crée une session via le backend, la sélectionne et ferme la popup.
 *
 * @param nom - Nom de la session à créer.
 */
export async function confirmCreateSession(nom: string) {
  const session = await invoke<Session>('create_session', { input: { nom } })
  backendSessions.value.unshift(session)
  state.sessionId = session.id
  state.createSessionPopup = false
  state.screen = 'tracking'
}

/**
 * Ouvre la popup de confirmation de suppression d'une session.
 *
 * @param id - Identifiant de la session à supprimer.
 * @param nom - Nom de la session (affiché dans la confirmation).
 */
export function openDeleteSessionPopup(id: number, nom: string) {
  state.deleteSessionPopup = { id, nom }
}

/** Ferme la popup de suppression de session sans supprimer. */
export function cancelDeleteSession() {
  state.deleteSessionPopup = null
}

/** Supprime la session ciblée par la popup de confirmation via le backend. */
export async function confirmDeleteSession() {
  const popup = state.deleteSessionPopup
  if (!popup) return
  try {
    await invoke('delete_session', { id: popup.id })
    backendSessions.value = backendSessions.value.filter(s => s.id !== popup.id)
    state.deleteSessionPopup = null
    state.sessionId = backendSessions.value.length ? backendSessions.value[0].id : 0
    state.screen = 'tracking'
  } catch (e) {
    console.error('Impossible de supprimer la session', e)
  }
}

/**
 * Ouvre le détail d'une candidature.
 *
 * @param card - Candidature à afficher.
 */
export function openCard(card: BoardCard) {
  state.selectedCard = card
  state.screen = 'detail'
  state.detailEvents = []
  state.detailRecruiters = [...MOCK_RECRUITERS]
}

/** Ajoute un contact vide au formulaire de nouvelle candidature. */
export function addContact() {
  state.newContacts.push({ id: state.nextContactId++ })
}

/**
 * Retire un contact du formulaire de nouvelle candidature (au moins un contact reste toujours présent).
 *
 * @param index - Index du contact à retirer.
 */
export function removeContact(index: number) {
  if (state.newContacts.length <= 1) return
  state.newContacts.splice(index, 1)
}

/**
 * Démarre le glisser-déposer d'une candidature dans le kanban.
 *
 * @param fromColIndex - Index de la colonne d'origine.
 * @param cardIndex - Index de la carte dans la colonne d'origine.
 */
export function startDrag(fromColIndex: number, cardIndex: number) {
  state.dragState = { fromColIndex, cardIndex }
}

/**
 * Termine le glisser-déposer sur une colonne cible.
 *
 * @param _toColIndex - Index de la colonne cible (popup de confirmation à implémenter).
 */
export function showDragPopup(_toColIndex: number) {
  state.dragState = null
}

/** Confirme le déplacement de candidature en attente. */
export function confirmDrag() {
  state.dragPopup = null
}

/** Annule le déplacement de candidature en cours. */
export function cancelDrag() {
  state.dragPopup = null
  state.dragState = null
}

/** Ouvre la corbeille de la session courante. */
export function openTrashPopup() {
  state.trashPopup = true
}

/** Ferme la corbeille. */
export function closeTrashPopup() {
  state.trashPopup = false
}

/**
 * Ouvre une popup du détail de candidature.
 *
 * @param key - Popup à ouvrir.
 */
export function openDetailPopup(key: PopupKey) {
  state.detailPopup = key
}

/** Ferme la popup courante du détail de candidature. */
export function closeDetailPopup() {
  state.detailPopup = null
}

/**
 * Ajoute un événement au suivi de la candidature ouverte.
 *
 * @param type - Type d'événement à ajouter.
 */
export function addEvent(type: EventType) {
  state.detailEvents.push({ id: state.nextEventId++, type })
  state.detailPopup = null
}

/**
 * Retire un événement du suivi de la candidature ouverte.
 *
 * @param id - Identifiant de l'événement à retirer.
 */
export function removeEvent(id: number) {
  state.detailEvents = state.detailEvents.filter((e: AppEvent) => e.id !== id)
}

/** Ajoute un recruteur de démonstration au suivi de la candidature ouverte. */
export function addRecruiter() {
  state.detailRecruiters.push({
    id: state.nextRecruiterId++,
    nom: 'Nouveau Recruteur',
    poste: 'Chargé·e RH',
    email: 'recruiter@entreprise.com',
  })
  state.detailPopup = null
}

/** Envoie la candidature ouverte à la corbeille et retourne au suivi. */
export function trashCard() {
  state.selectedCard = null
  state.screen = 'tracking'
}

/** Marque la candidature ouverte comme acceptée et retourne au suivi. */
export function acceptCard() {
  state.selectedCard = null
  state.screen = 'tracking'
}

/** Annule l'acceptation de la candidature ouverte et retourne au suivi. */
export function cancelAcceptation() {
  state.selectedCard = null
  state.screen = 'tracking'
}

// Expose raw state for components that need it
export { state }
