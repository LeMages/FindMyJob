<script setup lang="ts">
/**
 * Grille semaine de l'onglet planning : 7 colonnes (lundi → dimanche), une
 * ligne par heure. Chaque entretien est positionné selon son heure. Propre à
 * `PlanningPanel`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'

// Importation des modules internes
import { interviewTypeColor } from '@/composables/candidature_display'
import { addDays, formatDayLabel, formatHour, formatTime, isSameDay, isToday, parseInterviewDate } from './planning_dates'
import type { PlanningInterviewView } from '@/types'

const props = defineProps<{
  items: PlanningInterviewView[]
  weekStart: Date
}>()

const emit = defineEmits<{ select: [interview: PlanningInterviewView] }>()

/** Hauteur d'une heure de grille, en pixels. */
const HOUR_HEIGHT = 48

const days = computed(() => Array.from({ length: 7 }, (_, i) => addDays(props.weekStart, i)))

/** Entretiens de la semaine, avec leur date parsée une seule fois. */
const parsedItems = computed(() =>
  props.items
    .map(interview => ({ interview, date: parseInterviewDate(interview.date_entretien) }))
    .filter((entry): entry is { interview: PlanningInterviewView; date: Date } => entry.date !== null)
)

// Plage horaire 8h-20h par défaut, élargie pour englober tout entretien de la
// semaine affichée (bornée à une journée complète) — un entretien à 7h30 ne
// doit pas disparaître de la grille.
const hourMin = computed(() => Math.min(8, ...parsedItems.value.map(e => e.date.getHours()), 8))
const hourMax = computed(() => Math.max(20, ...parsedItems.value.map(e => e.date.getHours()), 20))
const hours = computed(() => Array.from({ length: hourMax.value - hourMin.value + 1 }, (_, i) => hourMin.value + i))
const gridHeight = computed(() => hours.value.length * HOUR_HEIGHT)

/**
 * Entretiens tombant sur un jour donné, avec leur position verticale dans la grille.
 *
 * @param day - Jour de la colonne.
 * @returns Entretiens du jour avec leur `top` en pixels.
 */
function itemsForDay(day: Date) {
  return parsedItems.value
    .filter(e => isSameDay(e.date, day))
    .map(e => ({
      ...e,
      top: (e.date.getHours() - hourMin.value) * HOUR_HEIGHT + (e.date.getMinutes() / 60) * HOUR_HEIGHT,
    }))
}

/**
 * Indique si un entretien est déjà passé (atténué dans la grille).
 *
 * @param date - Date de l'entretien.
 * @returns `true` si la date est dans le passé.
 */
function isPast(date: Date): boolean {
  return date.getTime() < Date.now()
}
</script>

<template>
  <div class="flex border border-neutral-200 rounded-xl overflow-hidden bg-white">
    <!-- Colonne des heures -->
    <div class="w-[52px] flex-none border-r border-neutral-200">
      <div class="h-10 border-b border-neutral-200" />
      <div v-for="h in hours" :key="h" class="border-b border-neutral-100 text-[10.5px] font-mono text-neutral-400 text-right pr-2 pt-1" :style="{ height: `${HOUR_HEIGHT}px` }">
        {{ formatHour(h) }}
      </div>
    </div>

    <!-- Colonnes des jours -->
    <div v-for="day in days" :key="day.toISOString()" class="flex-1 min-w-0 border-r border-neutral-100 last:border-r-0">
      <div class="h-10 border-b border-neutral-200 flex flex-col items-center justify-center text-[11.5px] font-mono">
        <span :class="isToday(day) ? 'text-neutral-950 font-semibold' : 'text-neutral-500'">{{ formatDayLabel(day) }}</span>
      </div>
      <div class="relative" :class="isToday(day) ? 'bg-neutral-100/60' : ''" :style="{ height: `${gridHeight}px` }">
        <div v-for="h in hours" :key="h" class="border-b border-neutral-100" :style="{ height: `${HOUR_HEIGHT}px` }" />
        <div
          v-for="entry in itemsForDay(day)"
          :key="entry.interview.id"
          class="absolute left-1 right-1 rounded-lg border px-2 py-1 text-[11.5px] cursor-pointer overflow-hidden"
          :class="[interviewTypeColor(entry.interview.type), isPast(entry.date) ? 'opacity-60' : '']"
          :style="{ top: `${entry.top}px`, height: '44px' }"
          @click="emit('select', entry.interview)"
        >
          <div class="font-semibold truncate">{{ entry.interview.entreprise ?? 'Entreprise inconnue' }}</div>
          <div class="truncate text-[10.5px]">{{ entry.interview.poste }} · <span class="font-mono">{{ formatTime(entry.date) }}</span></div>
        </div>
      </div>
    </div>
  </div>
</template>
