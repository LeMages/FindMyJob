<script setup lang="ts">
/**
 * Répartition des candidatures envoyées par issue : barre empilée et
 * légende avec les effectifs (l'identité ne repose jamais sur la couleur
 * seule).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import StatsCard from '@/views/tracking/components/stats/StatsCard.vue'

// Importation des modules internes
import type { SessionStats } from '@/types'

const { t } = useI18n()

defineProps<{ stats: SessionStats }>()

/** Couleur fixe de chaque issue (ne dépend pas du rang). */
const COLORS: Record<string, string> = {
  in_progress: '#3b82f6',
  offer: '#10b981',
  refus: '#e11d48',
  ghosting: '#c4c4bd',
  abandon: '#8a8a85',
  offre_refusee: '#d97706',
}
</script>

<template>
  <StatsCard :title="t('stats.outcomes.title')" :hint="t('stats.outcomes.hint')">
    <div class="flex gap-0.5 h-[18px] rounded-[5px] overflow-hidden">
      <div
        v-for="o in stats.outcomes"
        :key="o.outcome"
        :title="`${t(`stats.outcomes.labels.${o.outcome}`)} : ${o.count}`"
        :style="{ flex: o.count, background: COLORS[o.outcome] }"
      />
    </div>
    <div class="flex flex-wrap gap-x-[14px] gap-y-[6px] mt-[10px] text-[12px] text-neutral-600">
      <span v-for="o in stats.outcomes" :key="o.outcome">
        <i class="inline-block w-[9px] h-[9px] rounded-[2px] mr-[6px] align-[-1px]" :style="{ background: COLORS[o.outcome] }" />{{ t(`stats.outcomes.labels.${o.outcome}`) }}
        <b class="text-neutral-950 font-semibold ml-0.5">{{ o.count }}</b>
      </span>
    </div>
    <div class="text-[12px] text-neutral-500 mt-2">{{ t('stats.outcomes.note', { n: stats.sent }, stats.sent) }}</div>
  </StatsCard>
</template>
