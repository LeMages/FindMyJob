<script setup lang="ts">
/**
 * Carte « Ce qui marche » d'un critère : une ligne par valeur avec
 * « réponses/envois · % » et le nombre d'entretiens. Les lignes trop peu
 * fournies sont grisées avec la mention « trop peu » ; la meilleure est
 * mise en avant.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import StatsCard from '@/views/tracking/components/stats/StatsCard.vue'

// Importation des modules internes
import { sourceLabel } from '@/composables/candidature_display'
import { percent, shortLabel } from '@/views/tracking/components/stats/stats_format'
import type { Comparison, ComparisonRow } from '@/types'

const { t } = useI18n()

const props = defineProps<{
  comparison: Comparison
  /** Seuil sous lequel une ligne est « trop peu » (fourni par le backend). */
  minSent: number
}>()

/** Libellé affiché d'une ligne selon le critère comparé. */
function rowLabel(row: ComparisonRow): string {
  const criterion = props.comparison.criterion
  if (criterion === 'contact') return t(`stats.works.rows.${row.key === 'with' ? 'with_contact' : 'without_contact'}`)
  if (criterion === 'source') return shortLabel(sourceLabel(row.key))
  if (row.key === 'unknown') return t('stats.works.rows.unknown_version')
  return t(criterion === 'cv_template' ? 'stats.works.rows.cv_version' : 'stats.works.rows.lm_version', { n: row.key })
}

const rows = computed(() => props.comparison.rows.map((row) => ({
  ...row,
  label: rowLabel(row),
  rate: percent(row.replied, row.sent),
  hint: row.low_sample
    ? t('stats.works.few_hint', { n: props.minSent })
    : t('stats.works.row_hint', { replied: row.replied, sent: row.sent }),
})))

const GRID = 'grid grid-cols-[minmax(0,140px)_1fr_60px] gap-3'
</script>

<template>
  <StatsCard :title="t(`stats.works.criteria.${comparison.criterion}.title`)" :hint="t(`stats.works.criteria.${comparison.criterion}.hint`)">
    <div :class="GRID" class="text-[10.5px] font-mono text-neutral-400 pb-1 border-b border-neutral-100">
      <span />
      <span>{{ t('stats.works.reply_rate') }}</span>
      <span class="text-right">{{ t('stats.works.interviews') }}</span>
    </div>
    <div
      v-for="row in rows"
      :key="row.key"
      :class="[GRID, row.low_sample ? 'opacity-45' : '']"
      class="items-center py-2 border-b border-neutral-100 last:border-b-0"
      :title="row.hint"
    >
      <div class="text-[12.5px] text-neutral-950 min-w-0 break-words">
        {{ row.label }}
        <small class="block text-[11px] text-neutral-500 font-mono">
          {{ t('stats.works.sends', { n: row.sent }, row.sent) }}<template v-if="row.low_sample"> · {{ t('stats.works.too_few') }}</template>
        </small>
      </div>
      <div class="flex items-center gap-[10px] min-w-0">
        <div class="flex-1 h-[10px] rounded-[5px] bg-neutral-100 overflow-hidden">
          <div class="h-full rounded-[5px]" :class="row.best ? 'bg-neutral-950' : 'bg-neutral-500'" :style="{ width: `${row.rate}%` }" />
        </div>
        <span class="text-[11.5px] font-mono text-neutral-600 whitespace-nowrap min-w-[84px] text-right">{{ row.replied }}/{{ row.sent }} · {{ row.rate }} %</span>
      </div>
      <div class="text-[12.5px] font-semibold text-right text-neutral-950">{{ row.interviewed }}</div>
    </div>
  </StatsCard>
</template>
