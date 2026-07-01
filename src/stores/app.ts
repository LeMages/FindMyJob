import { reactive, ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
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

export const currentTrashedList = computed<BoardCard[]>(() => [])

export const selectedCardColumn = computed<string>(() => '')

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

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

export function setUserInfo(displayName: string, initials: string) {
  state.userDisplayName = displayName
  state.userInitials = initials
}

export function navigate(screen: Screen) {
  state.screen = screen
}

export function selectSession(id: number) {
  state.sessionId = id
  state.screen = 'tracking'
  state.selectedCard = null
}

export function addSession() {
  state.createSessionPopup = true
}

export function cancelCreateSession() {
  state.createSessionPopup = false
}

export async function confirmCreateSession(nom: string) {
  const session = await invoke<Session>('create_session', { input: { nom } })
  backendSessions.value.unshift(session)
  state.sessionId = session.id
  state.createSessionPopup = false
  state.screen = 'tracking'
}

export function openDeleteSessionPopup(id: number, nom: string) {
  state.deleteSessionPopup = { id, nom }
}

export function cancelDeleteSession() {
  state.deleteSessionPopup = null
}

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

export function openCard(card: BoardCard) {
  state.selectedCard = card
  state.screen = 'detail'
  state.detailEvents = []
  state.detailRecruiters = [...MOCK_RECRUITERS]
}

export function addContact() {
  state.newContacts.push({ id: state.nextContactId++ })
}

export function removeContact(index: number) {
  if (state.newContacts.length <= 1) return
  state.newContacts.splice(index, 1)
}

export function startDrag(fromColIndex: number, cardIndex: number) {
  state.dragState = { fromColIndex, cardIndex }
}

export function showDragPopup(_toColIndex: number) {
  state.dragState = null
}

export function confirmDrag() {
  state.dragPopup = null
}

export function cancelDrag() {
  state.dragPopup = null
  state.dragState = null
}

export function openTrashPopup() {
  state.trashPopup = true
}

export function closeTrashPopup() {
  state.trashPopup = false
}

export function openDetailPopup(key: PopupKey) {
  state.detailPopup = key
}

export function closeDetailPopup() {
  state.detailPopup = null
}

export function addEvent(type: EventType) {
  state.detailEvents.push({ id: state.nextEventId++, type })
  state.detailPopup = null
}

export function removeEvent(id: number) {
  state.detailEvents = state.detailEvents.filter((e: AppEvent) => e.id !== id)
}

export function addRecruiter() {
  state.detailRecruiters.push({
    id: state.nextRecruiterId++,
    nom: 'Nouveau Recruteur',
    poste: 'Chargé·e RH',
    email: 'recruiter@entreprise.com',
  })
  state.detailPopup = null
}

export function trashCard() {
  state.selectedCard = null
  state.screen = 'tracking'
}

export function acceptCard() {
  state.selectedCard = null
  state.screen = 'tracking'
}

export function cancelAcceptation() {
  state.selectedCard = null
  state.screen = 'tracking'
}

// Expose raw state for components that need it
export { state }
