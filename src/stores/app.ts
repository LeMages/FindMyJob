import { reactive, computed } from 'vue'
import type {
  Screen, PopupKey, EventType,
  BoardCard, BoardColumn, Session,
  Recruiter, AppEvent, NewContact,
  DragState, DragPopupState,
} from '@/types'

// ---------------------------------------------------------------------------
// Mock data — will be replaced by Tauri invoke() calls when backend is ready
// ---------------------------------------------------------------------------

const MOCK_SESSIONS: Session[] = [
  { id: 0, name: 'Stage M2 – 2025' },
  { id: 1, name: 'Stage M1 – 2024' },
  { id: 2, name: 'Alternance ML' },
  { id: 3, name: 'CDI – 2026' },
]

const MOCK_BOARD: Record<number, BoardColumn[]> = {
  0: [
    {
      title: 'À postuler',
      cards: [
        { company: 'Doctolib', role: 'Stage Data Scientist', location: 'Paris', date: 'Ajouté 12 juin', meta: 'Deadline 30 juin' },
        { company: 'Datadog', role: 'Stage ML Engineer', location: 'Paris', date: 'Ajouté 10 juin', meta: '' },
      ],
    },
    {
      title: 'Candidature envoyée',
      cards: [
        { company: 'Criteo', role: 'Stage Software Engineer', location: 'Paris', date: 'Envoyé 5 juin', meta: 'Relance 19 juin' },
        { company: 'OVHcloud', role: 'Stage Backend', location: 'Lille', date: 'Envoyé 2 juin', meta: '' },
        { company: 'Dassault Systèmes', role: 'Stage R&D Simulation', location: 'Vélizy', date: 'Envoyé 28 mai', meta: '' },
      ],
    },
    { title: 'Discussion', cards: [] },
    {
      title: 'Entretien',
      cards: [
        { company: 'Ubisoft', role: 'Stage Gameplay Data', location: 'Montréal', date: 'Entretien 24 juin · 14h', meta: 'Visio' },
        { company: 'BNP Paribas', role: 'Stage Quant Analyst', location: 'Paris', date: 'Test technique 27 juin', meta: '' },
      ],
    },
    {
      title: 'Accepté',
      cards: [
        { company: 'Thales', role: 'Stage IA & Défense', location: 'Palaiseau', date: 'Offre reçue · 18 juin', meta: 'Offre acceptée' },
      ],
    },
  ],
  1: [
    { title: 'À postuler', cards: [] },
    {
      title: 'Candidature envoyée',
      cards: [
        { company: 'Sopra Steria', role: 'Stage Développeur Web', location: 'Toulouse', date: 'Envoyé sept 2024', meta: '' },
      ],
    },
    { title: 'Discussion', cards: [] },
    {
      title: 'Entretien',
      cards: [
        { company: 'Capgemini', role: 'Stage Data Analyst', location: 'Paris', date: 'Entretien oct 2024', meta: '' },
      ],
    },
    {
      title: 'Accepté',
      cards: [
        { company: 'Orange', role: 'Stage Data Science', location: 'Paris', date: 'Offre · acceptée', meta: 'Réalisé' },
      ],
    },
  ],
  2: [
    {
      title: 'À postuler',
      cards: [
        { company: 'Hugging Face', role: 'Alternance ML Engineer', location: 'Remote', date: 'Ajouté 14 juin', meta: '' },
        { company: 'Mistral AI', role: 'Alternance Research', location: 'Paris', date: 'Ajouté 11 juin', meta: 'Référent : Léa' },
      ],
    },
    {
      title: 'Candidature envoyée',
      cards: [
        { company: 'Contentsquare', role: 'Alternance Data', location: 'Paris', date: 'Envoyé 8 juin', meta: '' },
      ],
    },
    { title: 'Discussion', cards: [] },
    { title: 'Entretien', cards: [] },
    {
      title: 'Accepté',
      cards: [
        { company: 'Qonto', role: 'Alternance Backend', location: 'Paris', date: 'Refus · 6 juin', meta: 'Refus' },
      ],
    },
  ],
  3: [
    {
      title: 'À postuler',
      cards: [
        { company: 'Stripe', role: 'Software Engineer, New Grad', location: 'Paris', date: 'Ajouté 20 juin', meta: '' },
        { company: 'Alan', role: 'Backend Engineer', location: 'Paris', date: 'Ajouté 19 juin', meta: '' },
      ],
    },
    { title: 'Candidature envoyée', cards: [] },
    { title: 'Discussion', cards: [] },
    { title: 'Entretien', cards: [] },
    { title: 'Accepté', cards: [] },
  ],
}

const MOCK_TRASHED: Record<number, BoardCard[]> = { 0: [], 1: [], 2: [], 3: [] }

const MOCK_RECRUITERS: Recruiter[] = [
  { id: 1, nom: 'Sophie Lecomte', poste: 'Tech Recruiter', email: 's.lecomte@criteo.com' },
]

// ---------------------------------------------------------------------------
// Reactive state — UI state always lives here; data will move to backend
// ---------------------------------------------------------------------------

const state = reactive({
  // Navigation
  screen: 'tracking' as Screen,

  // Sessions
  sessionId: 0,
  deletedSessionIds: [] as number[],
  nextSessionId: 4,

  // Board (mock — will be backend)
  boardData: MOCK_BOARD as Record<number, BoardColumn[]>,
  trashedCards: MOCK_TRASHED as Record<number, BoardCard[]>,

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

  // Delete session popup
  deleteSessionPopup: null as { id: number; name: string } | null,

  // New application form contacts
  newContacts: [{ id: 0 }] as NewContact[],
  nextContactId: 1,
})

// ---------------------------------------------------------------------------
// Computed values (these will eventually come from backend)
// ---------------------------------------------------------------------------

export const sessions = computed<(Session & { count: number; hasAccepted: boolean })[]>(() =>
  MOCK_SESSIONS
    .concat(/* dynamically added sessions stored separately */ [])
    .filter(s => !state.deletedSessionIds.includes(s.id))
    .map(s => {
      const cols = state.boardData[s.id] ?? []
      const count = cols.reduce((acc, c) => acc + c.cards.length, 0)
      const hasAccepted = cols.some(c => c.title === 'Accepté' && c.cards.length > 0)
      return { ...s, count, hasAccepted }
    }),
)

// Keep track of dynamically added sessions separately
const extraSessions = reactive<Session[]>([])

export const allSessions = computed<(Session & { count: number; hasAccepted: boolean })[]>(() => {
  const base = [...MOCK_SESSIONS, ...extraSessions]
  return base
    .filter(s => !state.deletedSessionIds.includes(s.id))
    .map(s => {
      const cols = state.boardData[s.id] ?? []
      const count = cols.reduce((acc, c) => acc + c.cards.length, 0)
      const hasAccepted = cols.some(c => c.title === 'Accepté' && c.cards.length > 0)
      return { ...s, count, hasAccepted }
    })
})

export const currentSession = computed(() =>
  allSessions.value.find(s => s.id === state.sessionId) ?? allSessions.value[0],
)

export const currentBoard = computed<BoardColumn[]>(() =>
  state.boardData[state.sessionId] ?? [],
)

export const currentStats = computed(() => {
  const cols = currentBoard.value
  const counts: Record<string, number> = {}
  let total = 0
  for (const col of cols) {
    counts[col.title] = col.cards.length
    total += col.cards.length
  }
  return {
    total,
    sent: counts['Candidature envoyée'] ?? 0,
    interviews: counts['Entretien'] ?? 0,
    accepted: counts['Accepté'] ?? 0,
  }
})

export const currentTrashedList = computed<BoardCard[]>(() =>
  state.trashedCards[state.sessionId] ?? [],
)

export const sessionIsAccepted = computed(() =>
  currentBoard.value.some(c => c.title === 'Accepté' && c.cards.length > 0),
)

export const selectedCardColumn = computed<string>(() => {
  if (!state.selectedCard) return ''
  for (const col of currentBoard.value) {
    if (col.cards.some((c: BoardCard) => c.company === state.selectedCard!.company && c.role === state.selectedCard!.role))
      return col.title
  }
  return ''
})

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

export function navigate(screen: Screen) {
  state.screen = screen
}

export function selectSession(id: number) {
  state.sessionId = id
  state.screen = 'tracking'
  state.selectedCard = null
}

export function addSession() {
  const name = window.prompt('Nom de la session :')
  if (!name?.trim()) return
  const id = state.nextSessionId++
  extraSessions.push({ id, name: name.trim() })
  if (!state.boardData[id]) {
    state.boardData[id] = [
      { title: 'À postuler', cards: [] },
      { title: 'Candidature envoyée', cards: [] },
      { title: 'Discussion', cards: [] },
      { title: 'Entretien', cards: [] },
      { title: 'Accepté', cards: [] },
    ]
    state.trashedCards[id] = []
  }
}

export function openDeleteSessionPopup(id: number, name: string) {
  state.deleteSessionPopup = { id, name }
}

export function cancelDeleteSession() {
  state.deleteSessionPopup = null
}

export function confirmDeleteSession() {
  const popup = state.deleteSessionPopup
  if (!popup) return
  state.deletedSessionIds.push(popup.id)
  state.deleteSessionPopup = null
  const remaining = allSessions.value
  state.sessionId = remaining.length ? remaining[0].id : 0
  state.screen = 'tracking'
}

export function openCard(card: BoardCard) {
  state.selectedCard = card
  state.screen = 'detail'
  state.detailEvents = []
  state.detailRecruiters = [...MOCK_RECRUITERS]
}

export function validateNewApp(role: string, company: string, location: string) {
  const months = ['jan.', 'fév.', 'mar.', 'avr.', 'mai', 'juin', 'juil.', 'août', 'sep.', 'oct.', 'nov.', 'déc.']
  const d = new Date()
  const date = `Ajouté ${d.getDate()} ${months[d.getMonth()]}`
  const newCard: BoardCard = {
    company: company || '—',
    role: role || 'Nouvelle candidature',
    location,
    date,
    meta: '',
  }
  if (!state.boardData[state.sessionId]) return
  state.boardData[state.sessionId][0].cards.unshift(newCard)
  state.newContacts = [{ id: 0 }]
  state.nextContactId = 1
  state.screen = 'tracking'
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

export function showDragPopup(toColIndex: number) {
  const ds = state.dragState
  if (!ds || ds.fromColIndex === toColIndex) {
    state.dragState = null
    return
  }
  const cols = state.boardData[state.sessionId]
  const card = cols[ds.fromColIndex].cards[ds.cardIndex]
  state.dragPopup = {
    card,
    fromColTitle: cols[ds.fromColIndex].title,
    toColTitle: cols[toColIndex].title,
    fromColIndex: ds.fromColIndex,
    toColIndex,
    cardIndex: ds.cardIndex,
  }
  state.dragState = null
}

export function confirmDrag() {
  const p = state.dragPopup
  if (!p) return
  const cols = state.boardData[state.sessionId]
  const [card] = cols[p.fromColIndex].cards.splice(p.cardIndex, 1)
  cols[p.toColIndex].cards.push(card)
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
  const card = state.selectedCard
  if (!card) return
  const cols: BoardColumn[] = state.boardData[state.sessionId]
  for (const col of cols) {
    const idx = col.cards.findIndex((c: BoardCard) => c.company === card.company && c.role === card.role)
    if (idx !== -1) {
      col.cards.splice(idx, 1)
      break
    }
  }
  if (!state.trashedCards[state.sessionId]) state.trashedCards[state.sessionId] = []
  state.trashedCards[state.sessionId].push(card)
  state.selectedCard = null
  state.screen = 'tracking'
}

export function acceptCard() {
  const card = state.selectedCard
  if (!card) return
  const cols: BoardColumn[] = state.boardData[state.sessionId]
  for (const col of cols) {
    const idx = col.cards.findIndex((c: BoardCard) => c.company === card.company && c.role === card.role)
    if (idx !== -1) {
      col.cards.splice(idx, 1)
      break
    }
  }
  const acceptedCol = cols.find((c: BoardColumn) => c.title === 'Accepté')
  if (acceptedCol) acceptedCol.cards.push({ ...card, meta: 'Offre acceptée' })
  state.screen = 'tracking'
  state.selectedCard = null
}

export function cancelAcceptation() {
  const card = state.selectedCard
  if (!card) return
  const cols: BoardColumn[] = state.boardData[state.sessionId]
  const acceptedCol = cols.find((c: BoardColumn) => c.title === 'Accepté')
  if (acceptedCol) {
    const idx = acceptedCol.cards.findIndex((c: BoardCard) => c.company === card.company && c.role === card.role)
    if (idx !== -1) acceptedCol.cards.splice(idx, 1)
  }
  const firstCol = cols.find((c: BoardColumn) => c.title === 'À postuler')
  if (firstCol) firstCol.cards.unshift({ ...card, meta: '' })
  state.screen = 'tracking'
  state.selectedCard = null
}

// Expose raw state for components that need it
export { state }
