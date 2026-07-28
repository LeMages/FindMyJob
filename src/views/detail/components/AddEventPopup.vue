<script setup lang="ts">
/**
 * Popup d'ajout d'un événement à la timeline de suivi d'une candidature.
 * Étape 1 : choix du type d'événement parmi ceux renvoyés par le backend
 * pour le statut courant (`get_available_events`). Étape 2 : formulaire
 * adapté au type choisi. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { state, addTimelineEvent } from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { AvailableEvent, InterviewView, NewTimelineEvent } from '@/types'
import TimelineEventIcon from './TimelineEventIcon.vue'

/** Types d'entretien proposés au formulaire de planification. */
const INTERVIEW_TYPE_OPTIONS = [
  { value: 'rh', label: 'RH' },
  { value: 'technique', label: 'Technique' },
  { value: 'manager', label: 'Manager' },
  { value: 'final', label: 'Final' },
  { value: 'autre', label: 'Autre' },
]

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
const interviewContactId = ref<number | ''>('')

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
  interviewContactId.value = ''
  pendingInterviews.value = []
  selectedInterviewId.value = ''
  interviewIssue.value = 'positif'
}

/**
 * Libellé du champ de texte libre, adapté au type d'événement choisi.
 *
 * @param eventType - Type d'événement sélectionné.
 * @returns Libellé du champ à afficher.
 */
function contentLabel(eventType: string): string {
  switch (eventType) {
    case 'relance':
      return 'Contenu du message'
    case 'message_envoye':
    case 'message_recu':
      return 'Contenu'
    case 'appel':
      return "Notes de l'appel"
    case 'entretien_passe':
      return 'Notes (optionnel)'
    case 'note':
      return 'Note'
    default:
      return 'Notes'
  }
}

/** Ouvre la popup et charge les événements disponibles pour la candidature ouverte. */
async function open() {
  const candidatureId = state.detailCandidature?.id
  if (!candidatureId) return

  visible.value = true
  step.value = 'select'
  error.value = null
  selected.value = null
  resetFormFields()

  loadingEvents.value = true
  try {
    events.value = await invoke<AvailableEvent[]>('get_available_events', { candidatureId })
  } catch (e: unknown) {
    error.value = extractErrorMessage(e, 'Impossible de charger les événements disponibles.')
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
      error.value = extractErrorMessage(e, 'Impossible de charger les entretiens en attente.')
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
  if (!selected.value) return

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
      date_entretien: interviewDate.value || null,
      lieu: interviewLieu.value.trim() || null,
      contact_id: interviewContactId.value === '' ? null : Number(interviewContactId.value),
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

defineExpose({ open })
</script>

<template>
  <div v-if="visible" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="close">
    <div class="bg-white rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>

      <!-- Étape 1 : choix du type d'événement -->
      <template v-if="step === 'select'">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">Ajouter un événement</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="close">×</button>
        </div>

        <div v-if="loadingEvents" class="text-[12px] text-neutral-400 font-mono py-4 text-center">Chargement…</div>
        <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-10 p-3 text-[13px] text-red-600 mb-3">{{ error }}</div>

        <template v-else>
          <div
            v-for="ev in events"
            :key="ev.event_type"
            class="border border-neutral-200 rounded-xl px-[18px] py-4 cursor-pointer flex items-center gap-[14px] mb-[10px] bg-neutral-50 hover:border-neutral-300 hover:bg-neutral-100"
            @click="selectEvent(ev)"
          >
            <div class="w-9 h-9 rounded-9 bg-white border border-neutral-200 flex items-center justify-center text-neutral-600 flex-none">
              <TimelineEventIcon :event-type="ev.event_type" />
            </div>
            <div class="text-[14px] font-semibold">{{ ev.label }}</div>
          </div>
          <div v-if="events.length === 0" class="text-[12px] text-neutral-400 font-mono py-2">Aucun événement disponible pour ce statut.</div>
        </template>
      </template>

      <!-- Étape 2 : formulaire adapté au type choisi -->
      <template v-else-if="selected">
        <div class="flex items-center justify-between mb-5">
          <div class="text-[17px] font-semibold">{{ selected.label }}</div>
          <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="close">×</button>
        </div>

        <!-- entretien_planifie -->
        <template v-if="selected.event_type === 'entretien_planifie'">
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Type d'entretien</label>
            <select v-model="interviewType" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400">
              <option v-for="opt in INTERVIEW_TYPE_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Date et heure</label>
            <input v-model="interviewDate" type="datetime-local" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" />
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Lieu (optionnel)</label>
            <input v-model="interviewLieu" placeholder="Visio, locaux de l'entreprise..." class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" />
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Contact (optionnel)</label>
            <select v-model="interviewContactId" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400">
              <option value="">Aucun</option>
              <option v-for="rec in state.detailContacts" :key="rec.id" :value="rec.id">{{ rec.nom ?? '—' }}</option>
            </select>
          </div>
        </template>

        <!-- entretien_passe -->
        <template v-else-if="selected.event_type === 'entretien_passe'">
          <div v-if="loadingInterviews" class="text-[12px] text-neutral-400 font-mono py-2">Chargement…</div>
          <template v-else>
            <div class="mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Entretien</label>
              <select v-model="selectedInterviewId" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400">
                <option value="" disabled>Choisir l'entretien passé</option>
                <option v-for="iv in pendingInterviews" :key="iv.id" :value="iv.id">
                  {{ iv.type ?? 'Entretien' }} — {{ iv.date_entretien ?? 'Date non renseignée' }}
                </option>
              </select>
            </div>
            <div class="mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Issue</label>
              <select v-model="interviewIssue" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400">
                <option value="positif">Positif</option>
                <option value="negatif">Négatif</option>
              </select>
            </div>
          </template>
        </template>

        <!-- Champ texte libre, commun à tous les autres événements -->
        <div v-if="!['entretien_planifie'].includes(selected.event_type)" class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ contentLabel(selected.event_type) }}</label>
          <textarea
            v-model="content"
            class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-[140px]"
          />
        </div>

        <div v-if="error" class="bg-red-50 border border-red-200 rounded-10 p-3 text-[13px] text-red-600 mb-3">{{ error }}</div>

        <div class="flex justify-between items-center mt-5 pt-4 border-t border-neutral-100">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
            @click="back"
          >← Retour</button>
          <div class="flex gap-[10px]">
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
              @click="close"
            >Annuler</button>
            <button
              class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
              :class="{ 'opacity-40 pointer-events-none': saving }"
              @click="save"
            >Enregistrer</button>
          </div>
        </div>
      </template>

    </div>
  </div>
</template>
