/**
 * Store applicatif global (état réactif partagé entre toutes les vues) :
 * navigation, sessions, candidature sélectionnée, popups, carnet de
 * contacts. Les données (candidatures, sessions, contacts) viennent du
 * backend ; le store ne garde que l'état d'affichage.
 */

// Importation des modules Vue / framework
import { reactive, ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import type {
  Screen, PopupKey,
  BoardCard, Session, UpdateSessionInput,
  CandidatureDetail, ContactView, StatusHistoryView, DocumentFileInfo,
  ContactDraft, ContactInput, NewTimelineEvent,
  UpdateCandidatureInput,
} from '@/types'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import { downloadApplyDocuments, type ApplyMessageExport } from '@/composables/document_download'
import { draftToContactInput, emptyContactDraft } from '@/composables/contact_draft'

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
  userNom: null as string | null,
  userPrenom: null as string | null,

  // Notification transitoire globale (voir `showToast`)
  toast: null as { message: string; kind: 'success' | 'error' } | null,

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
  // Contact modifié (popup `contact`) ou à supprimer (`confirmDeleteContact`) ;
  // `null` dans la popup `contact` = ajout d'un contact
  detailContact: null as ContactView | null,

  // Carnet de contacts (tous les contacts, toutes candidatures confondues)
  contactBook: [] as ContactView[],

  // Trash popup
  trashPopup: false,

  // Create session popup
  createSessionPopup: false,

  // Delete session popup
  deleteSessionPopup: null as { id: number; nom: string } | null,

  // Contacts saisis dans le formulaire de nouvelle candidature
  newContacts: [] as ContactDraft[],
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
 * Met à jour l'identité de l'utilisateur : nom et initiales affichés dans la
 * barre latérale, nom/prénom utilisés pour nommer les documents téléchargés.
 *
 * @param displayName - Nom complet à afficher.
 * @param initials - Initiales (fallback avatar).
 * @param nom - Nom de famille.
 * @param prenom - Prénom.
 */
export function setUserInfo(displayName: string, initials: string, nom: string | null, prenom: string | null) {
  state.userDisplayName = displayName
  state.userInitials = initials
  state.userNom = nom
  state.userPrenom = prenom
}

/** Durée d'affichage d'un toast avant disparition automatique. */
const TOAST_DURATION_MS = 4000
let toastTimer: ReturnType<typeof setTimeout> | null = null

/**
 * Affiche une notification transitoire globale (remplace la précédente).
 *
 * @param message - Texte affiché.
 * @param kind - `'success'` ou `'error'` (style du toast).
 */
export function showToast(message: string, kind: 'success' | 'error' = 'success') {
  if (toastTimer) clearTimeout(toastTimer)
  state.toast = { message, kind }
  toastTimer = setTimeout(() => {
    state.toast = null
    toastTimer = null
  }, TOAST_DURATION_MS)
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
 * Met à jour un ou plusieurs champs éditables d'une session (titre, statut)
 * et synchronise la liste des sessions. Fusionne le
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
    statut: current.statut,
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
 * Recharge le détail de la candidature actuellement ouverte (candidature,
 * timeline de suivi) depuis le backend. Utilisé après l'ajout d'un événement
 * à la timeline, qui peut avoir changé le statut de la candidature.
 */
export async function reloadDetail(): Promise<void> {
  const current = state.detailCandidature
  if (!current) return
  await loadCardDetail(current.id)
}

/**
 * Ajoute un événement à la timeline de suivi de la candidature ouverte
 * (relance, entretien, message, appel, note...), puis recharge le détail en
 * cas de succès — l'événement peut avoir changé le statut de la candidature.
 *
 * @param event - Événement à ajouter (voir `get_available_events` pour les
 *   types disponibles selon le statut courant).
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function addTimelineEvent(event: NewTimelineEvent): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null

  try {
    await invoke<void>('add_timeline_event', { candidatureId: current.id, event })
    await reloadDetail()
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, "Impossible d'ajouter l'événement.")
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

/**
 * Ajoute une ligne de contact au formulaire de nouvelle candidature.
 *
 * @param draft - Ligne à ajouter (vide par défaut).
 */
export function addContactDraft(draft: ContactDraft = emptyContactDraft()) {
  state.newContacts.push(draft)
}

/**
 * Retire une ligne de contact du formulaire de nouvelle candidature.
 *
 * @param index - Index de la ligne à retirer.
 */
export function removeContactDraft(index: number) {
  state.newContacts.splice(index, 1)
}

/** Vide les lignes de contact du formulaire de nouvelle candidature. */
export function resetContactDrafts() {
  state.newContacts = []
}

/**
 * Charge le carnet de contacts depuis le backend.
 *
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function loadContactBook(): Promise<string | null> {
  try {
    state.contactBook = await invoke<ContactView[]>('list_contacts')
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de charger le carnet de contacts.')
  }
}

/**
 * Recharge les contacts de la candidature ouverte et le carnet, après une
 * opération sur un contact.
 */
async function reloadContacts() {
  const current = state.detailCandidature
  if (current) {
    state.detailContacts = await invoke<ContactView[]>('get_contacts', { candidatureId: current.id })
  }
  const error = await loadContactBook()
  if (error) throw new Error(error)
}

/**
 * Exécute une opération sur un contact, recharge les contacts et ferme la
 * popup du détail en cas de succès.
 *
 * @param operation - Appel(s) au backend.
 * @param errorFallback - Message d'erreur par défaut.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
async function runContactOperation(operation: () => Promise<unknown>, errorFallback: string): Promise<string | null> {
  try {
    await operation()
    await reloadContacts()
    state.detailPopup = null
    state.detailContact = null
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, errorFallback)
  }
}

/**
 * Ajoute un contact à la candidature ouverte : lie un contact du carnet, ou
 * crée un nouveau contact lié à la candidature.
 *
 * @param draft - Ligne saisie dans la popup.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function addContactToDetail(draft: ContactDraft): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null
  if (draft.mode === 'existing') {
    if (draft.contact_id === null) return 'Choisis un contact du carnet.'
    const contactId = draft.contact_id
    return runContactOperation(
      () => invoke<void>('link_contact', { candidatureId: current.id, contactId }),
      'Impossible de lier le contact.',
    )
  }
  return runContactOperation(
    () => invoke<ContactView>('create_contact', { input: draftToContactInput(draft), candidatureId: current.id }),
    'Impossible de créer le contact.',
  )
}

/**
 * Modifie un contact du carnet.
 *
 * @param id - Contact à modifier.
 * @param input - Nouvelle saisie.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function updateContact(id: number, input: ContactInput): Promise<string | null> {
  return runContactOperation(
    () => invoke<ContactView>('update_contact', { id, input }),
    'Impossible de modifier le contact.',
  )
}

/**
 * Retire un contact de la candidature ouverte (et de ses entretiens) ; le
 * contact reste dans le carnet.
 *
 * @param contactId - Contact à retirer.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function unlinkContact(contactId: number): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null
  return runContactOperation(
    () => invoke<void>('unlink_contact', { candidatureId: current.id, contactId }),
    'Impossible de retirer le contact de la candidature.',
  )
}

/**
 * Supprime définitivement un contact du carnet (toutes ses liaisons partent avec).
 *
 * @param id - Contact à supprimer.
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function deleteContact(id: number): Promise<string | null> {
  return runContactOperation(
    () => invoke<void>('delete_contact', { id }),
    'Impossible de supprimer le contact.',
  )
}

/**
 * Ouvre la popup d'ajout (sans contact) ou de modification d'un contact.
 *
 * @param contact - Contact à modifier, `null` pour un ajout.
 */
export function openContactPopup(contact: ContactView | null) {
  state.detailContact = contact
  state.detailPopup = 'contact'
  void loadContactBook().then((error) => {
    if (error) showToast(error, 'error')
  })
}

/**
 * Ouvre la confirmation de suppression définitive d'un contact.
 *
 * @param contact - Contact à supprimer.
 */
export function openDeleteContactPopup(contact: ContactView) {
  state.detailContact = contact
  state.detailPopup = 'confirmDeleteContact'
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
  state.detailContact = null
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

/**
 * Supprime définitivement la candidature ouverte (statut « à postuler ») et
 * retourne au suivi de la session.
 *
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
export async function deleteCandidature(): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null

  try {
    await invoke<void>('delete_candidature', { id: current.id })
    state.detailPopup = null
    state.selectedCard = null
    state.screen = 'tracking'
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de supprimer la candidature.')
  }
}

/**
 * Télécharge les documents d'une candidature venant d'être postulée, sans
 * bloquer la navigation : le résultat (ou l'échec) est signalé par un toast,
 * la postulation elle-même restant acquise.
 *
 * @param candidature - Candidature postulée, capturée avant la navigation.
 * @param message - Mail ou message rédigé par l'IA à exporter en `.txt`, `null` sinon.
 */
async function downloadDocumentsAfterApply(candidature: CandidatureDetail, message: ApplyMessageExport | null) {
  try {
    const owner = { nom: state.userNom, prenom: state.userPrenom }
    const dir = await downloadApplyDocuments(candidature, owner, message)
    if (dir) showToast(`Documents téléchargés dans ${dir}`)
  } catch (e: unknown) {
    showToast(extractErrorMessage(e, 'Impossible de télécharger les documents de la candidature.'), 'error')
  }
}

/**
 * Invoque une commande Tauri « postuler » (mail, message ou sans message)
 * pour la candidature ouverte, retourne au suivi de la session en cas de
 * succès, puis télécharge ses documents en arrière-plan.
 *
 * @param command - Nom de la commande Tauri à invoquer (reçoit `{ id }`).
 * @param errorFallback - Message d'erreur par défaut si l'erreur n'est pas exploitable.
 * @param messageFileName - Nom du `.txt` où exporter le texte renvoyé par la
 *   commande (mail/message rédigé par l'IA) ; absent pour « sans message ».
 * @returns Message d'erreur en cas d'échec, sinon `null`.
 */
async function applyVia(command: string, errorFallback: string, messageFileName?: string): Promise<string | null> {
  const current = state.detailCandidature
  if (!current) return null

  try {
    const generated = await invoke<string | null>(command, { id: current.id })
    const message = messageFileName && generated ? { fileName: messageFileName, content: generated } : null
    state.detailPopup = null
    state.selectedCard = null
    state.screen = 'tracking'
    // Non attendu : le téléchargement ne doit pas retarder le retour au suivi
    // (downloadDocumentsAfterApply gère ses propres erreurs).
    void downloadDocumentsAfterApply(current, message)
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
  return applyVia('postuler_par_mail', 'Impossible de postuler par mail.', 'Mail_de_candidature.txt')
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
  return applyVia('postuler_par_message', 'Impossible de postuler par message.', 'Message_candidature.txt')
}

// Expose raw state for components that need it
export { state }
