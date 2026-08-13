<script setup lang="ts">
/**
 * Onglet « Planning » de la page Candidatures : calendrier en lecture seule
 * des entretiens de la session courante, en vue semaine ou mois.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { currentSession } from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import {
  addDays,
  addMonths,
  formatMonthLabel,
  monthGridDays,
  parseInterviewDate,
  startOfMonth,
  startOfWeek,
} from './planning_dates'
import PlanningWeekView from './PlanningWeekView.vue'
import PlanningMonthView from './PlanningMonthView.vue'
import InterviewDetailPopup from './InterviewDetailPopup.vue'
import type { PlanningInterviewView } from '@/types'

const interviews = ref<PlanningInterviewView[]>([])
const loading = ref(true)
const loadError = ref<string | null>(null)
const mode = ref<'semaine' | 'mois'>('semaine')
const cursor = ref(new Date())
const detailPopup = ref<InstanceType<typeof InterviewDetailPopup> | null>(null)

const sessionId = computed(() => currentSession.value?.id ?? null)

const weekStart = computed(() => startOfWeek(cursor.value))
const weekEnd = computed(() => addDays(weekStart.value, 6))

/** Bornes de la plage actuellement affichée (semaine ou grille du mois). */
const visibleRange = computed(() => {
  if (mode.value === 'semaine') {
    return { start: weekStart.value, end: addDays(weekStart.value, 7) }
  }
  const gridDays = monthGridDays(cursor.value)
  return { start: gridDays[0], end: addDays(gridDays[gridDays.length - 1], 1) }
})

/** Entretiens dont la date parsée tombe dans la plage actuellement affichée. */
const visibleInterviews = computed(() => {
  const { start, end } = visibleRange.value
  return interviews.value.filter(iv => {
    const date = parseInterviewDate(iv.date_entretien)
    return date !== null && date >= start && date < end
  })
})

/** Libellé de la plage affichée, adapté au mode courant. */
const rangeLabel = computed(() => {
  if (mode.value === 'mois') return formatMonthLabel(startOfMonth(cursor.value))
  const startLabel = weekStart.value.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' })
  const endLabel = weekEnd.value.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })
  return `${startLabel} – ${endLabel}`
})

/** Charge tous les entretiens planifiés de la session courante depuis le backend. */
async function loadInterviews() {
  const sid = currentSession.value?.id
  if (!sid) {
    interviews.value = []
    loading.value = false
    return
  }
  loading.value = true
  loadError.value = null
  try {
    interviews.value = await invoke<PlanningInterviewView[]>('get_all_interviews_for_session', { sessionId: sid })
  } catch (e: unknown) {
    loadError.value = extractErrorMessage(e, 'Impossible de charger les entretiens.')
  } finally {
    loading.value = false
  }
}

onMounted(loadInterviews)
watch(sessionId, loadInterviews)

/** Déplace la plage affichée vers la période précédente (semaine ou mois). */
function goToPrevious() {
  cursor.value = mode.value === 'semaine' ? addDays(cursor.value, -7) : addMonths(cursor.value, -1)
}

/** Déplace la plage affichée vers la période suivante (semaine ou mois). */
function goToNext() {
  cursor.value = mode.value === 'semaine' ? addDays(cursor.value, 7) : addMonths(cursor.value, 1)
}

/** Recentre la plage affichée sur aujourd'hui. */
function goToToday() {
  cursor.value = new Date()
}

/**
 * Ouvre la popup de détail pour l'entretien sélectionné dans la grille.
 *
 * @param interview - Entretien sélectionné.
 */
function selectInterview(interview: PlanningInterviewView) {
  detailPopup.value?.open(interview)
}
</script>

<template>
  <div class="px-[30px] pt-[18px] pb-[30px]">
    <!-- Contrôles : compteur, semaine/mois, navigation temporelle -->
    <div class="flex items-center justify-between mb-4">
      <div class="text-[13px] text-neutral-500 font-mono">
        {{ visibleInterviews.length }} entretien{{ visibleInterviews.length > 1 ? 's' : '' }} · {{ rangeLabel }}
      </div>
      <div class="flex items-center gap-[10px]">
        <!-- Segmenté semaine / mois -->
        <div class="flex bg-neutral-100 border border-neutral-200 rounded-lg p-[3px]">
          <button
            class="h-[30px] px-3 rounded-md text-[13px] font-medium cursor-pointer font-sans transition-colors duration-150"
            :class="mode === 'semaine' ? 'bg-neutral-950 text-white' : 'bg-transparent text-neutral-600 hover:text-neutral-950'"
            @click="mode = 'semaine'"
          >Semaine</button>
          <button
            class="h-[30px] px-3 rounded-md text-[13px] font-medium cursor-pointer font-sans transition-colors duration-150"
            :class="mode === 'mois' ? 'bg-neutral-950 text-white' : 'bg-transparent text-neutral-600 hover:text-neutral-950'"
            @click="mode = 'mois'"
          >Mois</button>
        </div>

        <!-- Navigation temporelle -->
        <button class="w-[38px] h-[38px] flex-none rounded-lg border border-neutral-200 bg-white flex items-center justify-center cursor-pointer hover:bg-neutral-100" title="Période précédente" @click="goToPrevious">
          <svg width="15" height="15" viewBox="0 0 15 15" fill="none"><path d="M9.5 3 5 7.5l4.5 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
          @click="goToToday"
        >Aujourd'hui</button>
        <button class="w-[38px] h-[38px] flex-none rounded-lg border border-neutral-200 bg-white flex items-center justify-center cursor-pointer hover:bg-neutral-100" title="Période suivante" @click="goToNext">
          <svg width="15" height="15" viewBox="0 0 15 15" fill="none"><path d="M5.5 3 10 7.5l-4.5 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">Chargement…</div>

    <!-- Load error -->
    <div v-else-if="loadError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>

    <!-- Content -->
    <template v-else>
      <PlanningWeekView v-if="mode === 'semaine'" :items="visibleInterviews" :week-start="weekStart" @select="selectInterview" />
      <PlanningMonthView v-else :items="visibleInterviews" :month-anchor="cursor" @select="selectInterview" />
      <div v-if="visibleInterviews.length === 0" class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucun entretien</div>
    </template>
  </div>

  <!-- Popup de détail -->
  <InterviewDetailPopup ref="detailPopup" />
</template>
