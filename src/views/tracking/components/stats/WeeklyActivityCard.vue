<script setup lang="ts">
/**
 * Activité des dernières semaines : candidatures envoyées et entretiens,
 * en barres groupées sur un seul axe, avec une infobulle au survol de
 * chaque semaine.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import StatsCard from '@/views/tracking/components/stats/StatsCard.vue'

// Importation des modules internes
import { dateLocale } from '@/i18n'
import type { WeekActivity } from '@/types'

const { t } = useI18n()

const props = defineProps<{ weeks: WeekActivity[] }>()

// Géométrie du graphique (unités du viewBox) ; reprise de la maquette.
const W = 1200
const H = 200
const BAR = 24
const BASE = H - 24

const hovered = ref<number | null>(null)

const maxValue = computed(() => Math.max(1, ...props.weeks.flatMap((w) => [w.sent, w.interviews])))
const gap = computed(() => W / Math.max(1, props.weeks.length))

/** Ordonnée d'une valeur. */
function y(v: number): number {
  return BASE - (v / maxValue.value) * (H - 44)
}

function weekLabel(week: WeekActivity): string {
  return new Date(`${week.week_start}T12:00:00`).toLocaleDateString(dateLocale(), { day: 'numeric', month: 'short' })
}

const gridValues = computed(() => [...new Set([0, Math.ceil(maxValue.value / 2), maxValue.value])])

const tooltip = computed(() => {
  if (hovered.value === null) return null
  const week = props.weeks[hovered.value]
  return {
    left: `${((hovered.value + 0.5) * gap.value / (W + 24)) * 100}%`,
    title: t('stats.weekly.tooltip', { date: weekLabel(week) }),
    sent: t('stats.weekly.sent_count', { n: week.sent }, week.sent),
    interviews: t('stats.weekly.interview_count', { n: week.interviews }, week.interviews),
  }
})
</script>

<template>
  <StatsCard :title="t('stats.weekly.title')" :hint="t('stats.weekly.hint', { n: weeks.length })">
    <div class="relative">
      <svg :viewBox="`0 0 ${W + 24} ${H}`" class="w-full h-auto block" role="img" :aria-label="t('stats.weekly.title')" @mouseleave="hovered = null">
        <g v-for="v in gridValues" :key="`g${v}`">
          <line x1="0" :x2="W" :y1="y(v)" :y2="y(v)" class="stroke-neutral-100" stroke-width="1" />
          <text :x="W + 6" :y="y(v) + 4" class="fill-neutral-500 font-mono text-[10px]">{{ v }}</text>
        </g>
        <g v-for="(week, i) in weeks" :key="week.week_start" @mouseenter="hovered = i">
          <rect
            :x="i * gap + gap / 2 - BAR - 1"
            :y="y(week.sent)"
            :width="BAR"
            :height="BASE - y(week.sent)"
            rx="3"
            class="fill-neutral-950"
            :opacity="hovered === null || hovered === i ? 1 : 0.45"
          />
          <rect
            :x="i * gap + gap / 2 + 1"
            :y="y(week.interviews)"
            :width="BAR"
            :height="Math.max(0, BASE - y(week.interviews))"
            rx="3"
            fill="#8b5cf6"
            :opacity="hovered === null || hovered === i ? 1 : 0.45"
          />
          <!-- Zone de survol pleine hauteur, plus large que les barres -->
          <rect :x="i * gap" y="0" :width="gap" :height="H" fill="transparent" />
          <text :x="i * gap + gap / 2" :y="H - 6" text-anchor="middle" class="fill-neutral-500 font-mono text-[10px]">{{ weekLabel(week) }}</text>
        </g>
      </svg>
      <div
        v-if="tooltip"
        class="absolute top-0 -translate-x-1/2 pointer-events-none z-10 bg-neutral-950 text-white text-[12px] rounded-md px-[10px] py-[7px] shadow-[0_6px_18px_rgba(0,0,0,.18)] whitespace-nowrap"
        :style="{ left: tooltip.left }"
      >
        <div class="font-medium mb-0.5">{{ tooltip.title }}</div>
        <div class="flex items-center gap-[6px]"><i class="inline-block w-2 h-2 rounded-[2px] bg-white" />{{ tooltip.sent }}</div>
        <div class="flex items-center gap-[6px]"><i class="inline-block w-2 h-2 rounded-[2px] bg-[#8b5cf6]" />{{ tooltip.interviews }}</div>
      </div>
    </div>
    <div class="flex flex-wrap gap-x-[14px] gap-y-[6px] mt-[10px] text-[12px] text-neutral-600">
      <span><i class="inline-block w-[9px] h-[9px] rounded-[2px] mr-[6px] align-[-1px] bg-neutral-950" />{{ t('stats.weekly.sent') }}</span>
      <span><i class="inline-block w-[9px] h-[9px] rounded-[2px] mr-[6px] align-[-1px] bg-[#8b5cf6]" />{{ t('stats.weekly.interviews') }}</span>
    </div>
  </StatsCard>
</template>
