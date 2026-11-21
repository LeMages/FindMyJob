<script setup lang="ts">
/**
 * Popup d'ajout d'un événement à la timeline de suivi d'une candidature.
 * Étape 1 : choix du type d'événement parmi ceux renvoyés par le backend
 * pour le statut courant (`get_available_events`), en grille par catégorie.
 * Étape 2 : formulaire adapté au type choisi (flèche de retour, icône et
 * titre de l'événement, « Ajouter au suivi »). Refuse de s'ouvrir quand la
 * session est en lecture seule. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import InfoBanner from '@/components/InfoBanner.vue'
import UiIcon from '@/components/UiIcon.vue'
import EventTypeGrid from './EventTypeGrid.vue'
import PastInterviewFields from './PastInterviewFields.vue'
import ScheduledInterviewFields from './ScheduledInterviewFields.vue'
import TimelineEventIcon from './TimelineEventIcon.vue'

// Importation des modules internes
import { state, addTimelineEvent, isSessionLocked } from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'
import { eventActionLabel } from '@/composables/candidature_display'
import { groupEvents } from './event_categories'
import type { AvailableEvent, InterviewView, NewTimelineEvent } from '@/types'

const { t, te } = useI18n()

const visible = ref(false)
const step = ref<'select' | 'form'>('select')
const loadingEvents = ref(false)
const events = ref<AvailableEvent[]>([])
const selected = ref<AvailableEvent | null>(null)
const error = ref<string | null>(null)
const saving = ref(false)

// Champ commun à la plupart des événements (message, note, notes libres...).
const content = ref('')

// Champs propres à `entretien_planifie`.
const interviewType = ref('rh')
const interviewDate = ref('')
const interviewLieu = ref('')
const interviewModalite = ref('')
// Interlocuteurs : uniquement des contacts liés à la candidature.
const interviewContactIds = ref<number[]>([])

// Champs propres à `entretien_passe`.
const loadingInterviews = ref(false)
const pendingInterviews = ref<InterviewView[]>([])
const selectedInterviewId = ref<number | ''>('')
const interviewIssue = ref<'positif' | 'negatif'>('positif')

/** Réinitialise tous les champs du formulaire (étape 2). */
function resetFormFields() {
  content.value = ''
  interviewType.value = 'rh'
  interviewDate.value = ''
  interviewLieu.value = ''
  interviewModalite.value = ''
  interviewContactIds.value = []
  pendingInterviews.value = []
  selectedInterviewId.value = ''
  interviewIssue.value = 'positif'
}

/** Événements disponibles regroupés par catégorie. */
const groupedEvents = computed(() => groupEvents(events.value))

/**
 * Libellé du champ de texte libre, adapté au type d'événement choisi.
 *
 * @param eventType - Type d'événement sélectionné.
 * @returns Libellé du champ à afficher.
 */
function contentLabel(eventType: string): string {
  switch (eventType) {
    case 'relance':
      return t('detail.add_event.content.relance')
    case 'message_envoye':
    case 'message_recu':
      return t('detail.add_event.content.message')
    case 'appel':
      return t('detail.add_event.content.appel')
    case 'entretien_passe':
    case 'accepter_offre':
      return t('detail.add_event.content.optional_notes')
    case 'decliner_offre':
      return t('detail.add_event.content.decline_reason')
    case 'note':
      return t('detail.add_event.content.note')
    default:
      return t('detail.add_event.content.default')
  }
}

/** Ouvre la popup et charge les événements disponibles pour la candidature ouverte. */
async function open() {
  const candidature = state.detailCandidature
  // Session en pause ou terminée : lecture seule, aucun événement ne s'ajoute.
  if (!candidature || isSessionLocked(candidature.session_id)) return

  visible.value = true
  step.value = 'select'
  error.value = null
  selected.value = null
  resetFormFields()

  loadingEvents.value = true
  try {
    events.value = await invoke<AvailableEvent[]>('get_available_events', { candidatureId: candidature.id })
  } catch (e: unknown) {
    error.value = backendErrorMessage(e, t('error.fallback.load_available_events'))
  } finally {
    loadingEvents.value = false
  }
}

/** Ferme la popup. */
function close() {
  visible.value = false
}

/**
 * Sélectionne un type d'événement et passe au formulaire (étape 2). Charge
 * les entretiens en attente si l'événement choisi est `entretien_passe`.
 *
 * @param event - Événement choisi.
 */
async function selectEvent(event: AvailableEvent) {
  selected.value = event
  step.value = 'form'
  error.value = null
  resetFormFields()

  if (event.event_type === 'entretien_passe') {
    const candidatureId = state.detailCandidature?.id
    if (!candidatureId) return
    loadingInterviews.value = true
    try {
      pendingInterviews.value = await invoke<InterviewView[]>('get_pending_interviews', { candidatureId })
    } catch (e: unknown) {
      error.value = backendErrorMessage(e, t('error.fallback.load_pending_interviews'))
    } finally {
      loadingInterviews.value = false
    }
  }
}

/** Revient à la sélection du type d'événement (étape 1). */
function back() {
  step.value = 'select'
  selected.value = null
  error.value = null
}

/** Construit et enregistre l'événement saisi, puis ferme la popup en cas de succès. */
async function save() {
  if (!selected.value || saving.value) return

  const event: NewTimelineEvent = {
    event_type: selected.value.event_type,
    content: content.value.trim() || null,
    interview_data: null,
    interview_id: null,
    interview_issue: null,
  }

  if (selected.value.event_type === 'entretien_planifie') {
    event.interview_data = {
      type: interviewType.value,
      // `datetime-local` donne « YYYY-MM-DDTHH:mm » sans fuseau, que le
      // backend (`DateTime<Utc>`) refuse : conversion en RFC 3339 UTC.
      date_entretien: interviewDate.value ? new Date(interviewDate.value).toISOString() : null,
      lieu: interviewLieu.value.trim() || null,
      modalite: interviewModalite.value || null,
      contact_ids: interviewContactIds.value,
    }
  } else if (selected.value.event_type === 'entretien_passe') {
    event.interview_id = selectedInterviewId.value === '' ? null : Number(selectedInterviewId.value)
    event.interview_issue = interviewIssue.value
  }

  error.value = null
  saving.value = true
  const err = await addTimelineEvent(event)
  saving.value = false

  if (err) {
    error.value = err
    return
  }
  close()
}

/** Sous-titre du choix : poste et entreprise de la candidature. */
const candidatureSubtitle = computed(() => {
  const c = state.detailCandidature
  if (!c) return ''
  return c.entreprise ? `${c.poste} · ${c.entreprise}` : c.poste
})

/** Description d'une ligne de l'événement choisi (sous-titre du formulaire). */
const selectedDescription = computed(() => {
  const key = `event.description.${selected.value?.event_type}`
  return te(key) ? t(key) : undefined
})

defineExpose({ open })
</script>

<template>
  <!-- Étape 1 : choix du type d'événement -->
  <AppModal
    v-if="visible && (step === 'select' || !selected)"
    :title="t('detail.add_event.title')"
    :subtitle="candidatureSubtitle"
    :width="720"
    divided
    backdrop-close
    @close="close"
  >
    <div v-if="loadingEvents" class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('common.loading') }}</div>
    <InfoBanner v-else-if="error" tone="err">{{ error }}</InfoBanner>
    <template v-else>
      <EventTypeGrid :groups="groupedEvents" @select="selectEvent" />
      <div v-if="events.length === 0" class="text-[12px] text-neutral-500 mt-2">{{ t('detail.add_event.empty') }}</div>
    </template>
  </AppModal>

  <!-- Étape 2 : formulaire adapté au type choisi -->
  <AppModal
    v-else-if="visible && selected"
    :title="eventActionLabel(selected.event_type, selected.label)"
    :subtitle="selectedDescription"
    :width="680"
    divided
    back
    backdrop-close
    @back="back"
    @close="close"
  >
    <template #icon><TimelineEventIcon :event-type="selected.event_type" /></template>

    <ScheduledInterviewFields
      v-if="selected.event_type === 'entretien_planifie'"
      v-model:type="interviewType"
      v-model:date="interviewDate"
      v-model:modalite="interviewModalite"
      v-model:lieu="interviewLieu"
      v-model:contact-ids="interviewContactIds"
    />

    <template v-else-if="selected.event_type === 'entretien_passe'">
      <div v-if="loadingInterviews" class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('common.loading') }}</div>
      <PastInterviewFields
        v-else
        v-model:selected-id="selectedInterviewId"
        v-model:issue="interviewIssue"
        :interviews="pendingInterviews"
      />
    </template>

    <!-- Champ texte libre, commun à tous les autres événements -->
    <div v-if="selected.event_type !== 'entretien_planifie'" :class="selected.event_type === 'entretien_passe' ? 'mt-[14px]' : ''">
      <label class="text-[12px] text-neutral-600 mb-[5px] block" for="ae-content">{{ contentLabel(selected.event_type) }}</label>
      <textarea
        id="ae-content"
        v-model="content"
        class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[180px] focus:outline-none focus:border-neutral-400"
      />
    </div>

    <div v-if="error" class="mt-[14px]"><InfoBanner tone="err">{{ error }}</InfoBanner></div>

    <template #footer>
      <button
        type="button"
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        @click="close"
      >{{ t('common.cancel') }}</button>
      <button
        type="button"
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none"
        :disabled="saving"
        @click="save"
      >
        <template v-if="saving"><UiIcon name="spinner" /> {{ t('common.saving') }}</template>
        <template v-else>{{ t('detail.add_event.submit') }}</template>
      </button>
    </template>
  </AppModal>
</template>
