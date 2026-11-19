<script setup lang="ts">
/**
 * Entretiens passés par type, avec leurs issues positives.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import StatsCard from '@/views/tracking/components/stats/StatsCard.vue'

// Importation des modules internes
import { barWidth } from '@/views/tracking/components/stats/stats_format'
import type { InterviewTypeCount } from '@/types'

const { t } = useI18n()

const props = defineProps<{ types: InterviewTypeCount[] }>()

const max = computed(() => Math.max(0, ...props.types.map((c) => c.passed)))
</script>

<template>
  <StatsCard :title="t('stats.interviews.title')" :hint="t('stats.interviews.hint')">
    <div v-if="!types.length" class="text-[12.5px] text-neutral-500">{{ t('stats.interviews.empty') }}</div>
    <div
      v-for="c in types"
      :key="c.interview_type"
      class="grid grid-cols-[72px_1fr_18px_70px] gap-[10px] items-center py-[5px] text-[12.5px] text-neutral-700"
      :title="t('stats.interviews.row_hint', { type: t(`interview.type.${c.interview_type}`), positive: c.positive, passed: c.passed })"
    >
      <span class="truncate">{{ t(`interview.type.${c.interview_type}`) }}</span>
      <div class="h-[10px] rounded-[5px] bg-neutral-100 overflow-hidden">
        <div class="h-full rounded-[5px] bg-[#8b5cf6]" :style="{ width: `${barWidth(c.passed, max)}%` }" />
      </div>
      <b class="text-right text-neutral-950">{{ c.passed }}</b>
      <em class="not-italic text-[11px] text-neutral-500 font-mono">{{ t('stats.interviews.positive', { n: c.positive }, c.positive) }}</em>
    </div>
  </StatsCard>
</template>
