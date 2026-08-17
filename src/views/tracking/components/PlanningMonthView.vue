<script setup lang="ts">
/**
 * Grille mois de l'onglet planning : 7 colonnes × 6 semaines, entretiens
 * affichés en chips compactes par jour. Propre à `PlanningPanel`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'

// Importation des modules internes
import { interviewStatusDotColor, interviewTypeColor } from '@/composables/candidature_display'
import { formatTime, interviewStatusDot, isSameDay, isToday, monthGridDays, parseInterviewDate } from './planning_dates'
import type { PlanningInterviewView } from '@/types'

const props = defineProps<{
  items: PlanningInterviewView[]
  monthAnchor: Date
}>()

const emit = defineEmits<{ select: [interview: PlanningInterviewView] }>()

/** Nombre maximum de chips affichées par jour avant de replier le reste sous « +N ». */
const MAX_CHIPS_PER_DAY = 3

const NOMS_JOURS = ['Lun', 'Mar', 'Mer', 'Jeu', 'Ven', 'Sam', 'Dim']

const days = computed(() => monthGridDays(props.monthAnchor))

/** Entretiens du mois affiché, avec leur date parsée une seule fois. */
const parsedItems = computed(() =>
  props.items
    .map(interview => ({ interview, date: parseInterviewDate(interview.date_entretien) }))
    .filter((entry): entry is { interview: PlanningInterviewView; date: Date } => entry.date !== null)
)

/**
 * Entretiens tombant sur un jour donné, triés par heure.
 *
 * @param day - Jour de la cellule.
 * @returns Entretiens du jour, triés chronologiquement.
 */
function itemsForDay(day: Date) {
  return parsedItems.value
    .filter(e => isSameDay(e.date, day))
    .sort((a, b) => a.date.getTime() - b.date.getTime())
    .map(e => ({ ...e, statusDot: interviewStatusDot(e.interview.issue, e.date) }))
}

/**
 * Indique si un jour appartient au mois affiché (par opposition aux jours de
 * débordement des mois voisins).
 *
 * @param day - Jour à tester.
 * @returns `true` si `day` est dans le mois de `monthAnchor`.
 */
function inCurrentMonth(day: Date): boolean {
  return day.getMonth() === props.monthAnchor.getMonth() && day.getFullYear() === props.monthAnchor.getFullYear()
}
</script>

<template>
  <div class="border border-neutral-200 rounded-xl overflow-hidden bg-surface-1">
    <div class="grid grid-cols-7 border-b border-neutral-200">
      <div v-for="nom in NOMS_JOURS" :key="nom" class="py-2 text-center font-mono text-[10.5px] tracking-[.07em] text-neutral-400">{{ nom }}</div>
    </div>
    <div class="grid grid-cols-7">
      <div
        v-for="day in days"
        :key="day.toISOString()"
        class="min-h-[110px] border-b border-r border-neutral-100 p-[6px]"
        :class="isToday(day) ? 'bg-neutral-100/60' : ''"
      >
        <div
          class="w-5 h-5 flex items-center justify-center text-[11.5px] rounded-full mb-1"
          :class="[
            isToday(day) ? 'bg-neutral-950 text-white font-semibold' : '',
            !isToday(day) && inCurrentMonth(day) ? 'text-neutral-800' : '',
            !inCurrentMonth(day) ? 'text-neutral-300' : '',
          ]"
        >
          {{ day.getDate() }}
        </div>

        <div class="flex flex-col gap-[3px]">
          <div
            v-for="entry in itemsForDay(day).slice(0, MAX_CHIPS_PER_DAY)"
            :key="entry.interview.id"
            class="flex items-center gap-1 rounded-md border px-[6px] py-[2px] text-[10.5px] cursor-pointer"
            :class="interviewTypeColor(entry.interview.type)"
            @click="emit('select', entry.interview)"
          >
            <span v-if="entry.statusDot" class="w-1.5 h-1.5 rounded-full flex-none" :class="interviewStatusDotColor(entry.statusDot)" />
            <span class="truncate min-w-0 flex-1"><span class="font-mono">{{ formatTime(entry.date) }}</span> {{ entry.interview.entreprise ?? 'Entreprise inconnue' }}</span>
          </div>
          <div v-if="itemsForDay(day).length > MAX_CHIPS_PER_DAY" class="text-[10.5px] text-neutral-500 px-[6px]">
            +{{ itemsForDay(day).length - MAX_CHIPS_PER_DAY }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
