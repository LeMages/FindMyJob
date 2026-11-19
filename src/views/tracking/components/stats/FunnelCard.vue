<script setup lang="ts">
/**
 * Entonnoir envoyées → réponse → entretien → offre, avec le taux de passage
 * d'une étape à la suivante.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import StatsCard from '@/views/tracking/components/stats/StatsCard.vue'

// Importation des modules internes
import { barWidth, percent } from '@/views/tracking/components/stats/stats_format'
import type { SessionStats } from '@/types'

const { t } = useI18n()

const props = defineProps<{ stats: SessionStats }>()

const steps = computed(() => {
  const s = props.stats
  const raw = [
    { key: 'sent', n: s.sent },
    { key: 'replied', n: s.replied },
    { key: 'interviewed', n: s.interviewed },
    { key: 'offer', n: s.offers },
  ]
  return raw.map((step, i) => ({
    ...step,
    label: t(`stats.funnel.${step.key}`),
    width: barWidth(step.n, s.sent),
    rate: i > 0 ? percent(step.n, raw[i - 1].n) : null,
  }))
})
</script>

<template>
  <StatsCard :title="t('stats.funnel.title')" :hint="t('stats.funnel.hint')">
    <div
      v-for="step in steps"
      :key="step.key"
      class="grid grid-cols-[80px_1fr_34px_46px] items-center gap-[10px] py-[5px]"
      :title="`${step.label} : ${step.n}`"
    >
      <div class="text-[12.5px] text-neutral-700">{{ step.label }}</div>
      <div class="h-[10px] rounded-[5px] bg-neutral-100 overflow-hidden">
        <div class="h-full bg-neutral-950 rounded-[5px]" :style="{ width: `${step.width}%` }" />
      </div>
      <div class="text-[13px] font-semibold text-right text-neutral-950">{{ step.n }}</div>
      <div class="text-[11.5px] font-mono text-neutral-500">{{ step.rate === null ? '' : `${step.rate} %` }}</div>
    </div>
  </StatsCard>
</template>
