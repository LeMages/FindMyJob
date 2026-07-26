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
  Screen, PopupKey,
  BoardCard, Session, UpdateSessionInput,
  CandidatureDetail, ContactView, StatusHistoryView, DocumentFileInfo,
  NewContact,
  DragState, DragPopupState,
  UpdateCandidatureInput,
} from '@/types'
import { columnTitleForStatus } from '@/composables/candidature_display'
import { extractErrorMessage } from '@/composables/useErrorMessage'

// ---------------------------------------------------------------------------
// Sessions — chargées depuis le backend
// ---------------------------------------------------------------------------

const backendSessions = ref<Session[]>([])

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
  detailCandidature: null as CandidatureDetail | null,
  detailContacts: [] as ContactView[],
  detailHistory: [] as StatusHistoryView[],
  detailCvInfo: null as DocumentFileInfo | null,
  detailLettreInfo: null as DocumentFileInfo | null,
  detailLoading: false,
  detailError: null as string | null,
  detailStatusHistoryContent: null as string | null,
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

// Colonne du kanban où se trouve la candidature sélectionnée, dérivée de son
// statut réel une fois le détail chargé depuis le backend.
export const selectedCardColumn = computed<string>(() =>
  state.detailCandidature ? columnTitleForStatus(state.detailCandidature.status) : ''
)

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
 * Met à jour un ou plusieurs champs éditables d'une session (titre, contexte,
 * statut, zone cible) et synchronise la liste des sessions. Fusionne le
 * correctif avec les valeurs actuelles avant envoi, car le backend attend le
 * jeu complet des champs éditables.
 *
 * @param id - Identifiant de la session à modifier.
 * @param patch - Champs modifiés à envoyer au backend.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function updateSession(id: number, patch: Partial<UpdateSessionInput>): Promise<string | null> {
  const current = backendSessions.value.find(s => s.id === id)
  if (!current) return null

  const input: UpdateSessionInput = {
    nom: current.nom,
    contexte: current.contexte,
    statut: current.statut,
    zone: current.zone,
    ...patch,
  }

  try {
    const updated = await invoke<Session>('update_session', { id, input })
    const idx = backendSessions.value.findIndex(s => s.id === id)
    if (idx !== -1) backendSessions.value[idx] = updated
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de mettre à jour la session.')
  }
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
 * Ouvre le détail d'une candidature et charge ses données depuis le backend.
 *
 * @param card - Candidature à afficher.
 */
export function openCard(card: BoardCard) {
  state.selectedCard = card
  state.screen = 'detail'
  state.detailCandidature = null
  state.detailContacts = []
  state.detailHistory = []
  state.detailCvInfo = null
  state.detailLettreInfo = null
  state.detailError = null
  void loadCardDetail(card.id)
}

/**
 * Charge le détail complet d'une candidature (infos, contacts, historique,
 * état des documents joints) depuis le backend.
 *
 * @param id - Identifiant de la candidature à charger.
 */
async function loadCardDetail(id: number) {
  state.detailLoading = true
  state.detailError = null
  try {
    const [candidature, contacts, history] = await Promise.all([
      invoke<CandidatureDetail>('get_candidature', { id }),
      invoke<ContactView[]>('get_contacts', { candidatureId: id }),
      invoke<StatusHistoryView[]>('get_status_history', { candidatureId: id }),
    ])
    state.detailCandidature = candidature
    state.detailContacts = contacts
    state.detailHistory = history

    const [cvInfo, lettreInfo] = await Promise.all([
      candidature.cv_path
        ? invoke<DocumentFileInfo>('get_document_info', { path: candidature.cv_path })
        : Promise.resolve(null),
      candidature.lettre_motivation_path
        ? invoke<DocumentFileInfo>('get_document_info', { path: candidature.lettre_motivation_path })
        : Promise.resolve(null),
    ])
    state.detailCvInfo = cvInfo
    state.detailLettreInfo = lettreInfo
  } catch (e: unknown) {
    state.detailError = extractErrorMessage(e, 'Impossible de charger la candidature.')
  } finally {
    state.detailLoading = false
  }
}

/**
 * Met à jour un ou plusieurs champs éditables de la candidature ouverte
 * (détails du poste, notes). Fusionne le correctif avec les valeurs actuelles
 * avant envoi, car le backend attend le jeu complet des champs éditables.
 * N'a d'effet que si la candidature est encore au statut « à postuler ».
 *
 * @param patch - Champs modifiés à envoyer au backend.
 * @returns Message d'erreur en cas d'échec, sinon `null` (affiché par le champ appelant).
 */
export async function updateDetailCandidature(patch: Partial<UpdateCandidatureInput>): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null

  const input: UpdateCandidatureInput = {
    poste: current.poste,
    entreprise: current.entreprise,
    localisation: current.localisation,
    type_contrat: current.type_contrat,
    source: current.source,
    remuneration: current.remuneration,
    url_offre: current.url_offre,
    notes: current.notes,
    ...patch,
  }

  try {
    const updated = await invoke<CandidatureDetail>('update_candidature', { id: current.id, input })
    state.detailCandidature = updated
    if (state.selectedCard && state.selectedCard.id === updated.id) {
      state.selectedCard = {
        ...state.selectedCard,
        role: updated.poste,
        company: updated.entreprise ?? '—',
        location: updated.localisation ?? '',
      }
    }
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de mettre à jour la candidature.')
  }
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
 * Ouvre la popup de lecture du contenu associé à une entrée de l'historique
 * de statut (mail ou message envoyé lors du changement de statut).
 *
 * @param content - Contenu à afficher.
 */
export function openStatusHistoryPopup(content: string) {
  state.detailStatusHistoryContent = content
  state.detailPopup = 'statusHistoryContent'
}

// Ajout de recruteur non persistant en base : la commande Tauri `create_contact`
// reste à implémenter (voir plan de gate suivante). En attendant, on affiche le
// contact ajouté localement pour ne pas régresser l'UI existante.
/** Ajoute un recruteur de démonstration au suivi de la candidature ouverte. */
export function addRecruiter() {
  state.detailContacts.push({
    id: state.nextRecruiterId++,
    candidature_id: state.detailCandidature?.id ?? 0,
    nom: 'Nouveau Recruteur',
    role: 'Chargé·e RH',
    email: 'recruiter@entreprise.com',
    telephone: null,
    notes: null,
    created_at: new Date().toISOString(),
  })
  state.detailPopup = null
}

/** Envoie la candidature ouverte à la corbeille et retourne au suivi. */
export function trashCard() {
  state.selectedCard = null
  state.screen = 'tracking'
}

/**
 * Abandonne la candidature ouverte (statut « à postuler ») et retourne au
 * suivi de la session.
 *
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function deleteCandidature(): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null

  try {
    await invoke<void>('abandon_candidature', { id: current.id })
    state.detailPopup = null
    state.selectedCard = null
    state.screen = 'tracking'
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de supprimer la candidature.')
  }
}

/**
 * Invoque une commande Tauri « postuler » (mail, message ou sans message)
 * pour la candidature ouverte, puis retourne au suivi de la session en cas
 * de succès.
 *
 * @param command - Nom de la commande Tauri à invoquer (reçoit `{ id }`).
 * @param errorFallback - Message d'erreur par défaut si l'erreur n'est pas exploitable.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
async function applyVia(command: string, errorFallback: string): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null

  try {
    await invoke<void>(command, { id: current.id })
    state.detailPopup = null
    state.selectedCard = null
    state.screen = 'tracking'
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, errorFallback)
  }
}

/**
 * Postule sans message : exporte le CV et la lettre de motivation de la
 * candidature ouverte vers le dossier de téléchargements, marque la
 * candidature comme postulée, puis retourne au suivi de la session.
 *
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function applyWithoutMessage(): Promise<string | null> {
  return applyVia('postuler_sans_message', 'Impossible de postuler sans message.')
}

/**
 * Postule par mail : fait rédiger un mail de candidature par l'IA, exporte
 * CV, lettre de motivation et mail vers le dossier de téléchargements,
 * marque la candidature comme postulée, puis retourne au suivi de la session.
 *
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function applyByMail(): Promise<string | null> {
  return applyVia('postuler_par_mail', 'Impossible de postuler par mail.')
}

/**
 * Postule par message : fait rédiger un message court (LinkedIn ou
 * équivalent) par l'IA, exporte CV, lettre de motivation et message vers le
 * dossier de téléchargements, marque la candidature comme postulée, puis
 * retourne au suivi de la session.
 *
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function applyByMessage(): Promise<string | null> {
  return applyVia('postuler_par_message', 'Impossible de postuler par message.')
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
