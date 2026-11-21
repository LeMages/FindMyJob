<script setup lang="ts">
/**
 * Onglet « Statistiques » du suivi : chiffres clés, entonnoir, issues,
 * activité hebdomadaire, « Ce qui marche », entretiens et délais. Les
 * chiffres viennent de la commande `get_session_stats` (calcul côté Rust) ;
 * état vide tant que la session compte trop peu d'envois.
 */

// Importation des modules Vue / framework
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import ComparisonCard from '@/views/tracking/components/stats/ComparisonCard.vue'
import FunnelCard from '@/views/tracking/components/stats/FunnelCard.vue'
import InterviewsCard from '@/views/tracking/components/stats/InterviewsCard.vue'
import KeyValueRow from '@/views/tracking/components/stats/KeyValueRow.vue'
import OutcomesCard from '@/views/tracking/components/stats/OutcomesCard.vue'
import StatsCard from '@/views/tracking/components/stats/StatsCard.vue'
import StatsKpi from '@/views/tracking/components/stats/StatsKpi.vue'
import WeeklyActivityCard from '@/views/tracking/components/stats/WeeklyActivityCard.vue'

// Importation des composables
import { backendErrorMessage } from '@/composables/backend_error_message'

// Importation des modules internes
import { percent, roundDays } from '@/views/tracking/components/stats/stats_format'
import type { SessionStats } from '@/types'

const { t } = useI18n()

const props = defineProps<{ sessionId: number }>()

const stats = ref<SessionStats | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)

async function loadStats() {
  loading.value = true
  loadError.value = null
  try {
    stats.value = await invoke<SessionStats>('get_session_stats', { sessionId: props.sessionId })
  } catch (e: unknown) {
    loadError.value = backendErrorMessage(e, t('stats.load_error'))
  } finally {
    loading.value = false
  }
}

watch(() => props.sessionId, loadStats, { immediate: true })

/** Durée affichée (« 9 j »), tiret si non mesurable. */
function days(value: number | null): string {
  const n = roundDays(value)
  return n === null ? '—' : t('stats.days', { n })
}

const kpis = computed(() => {
  const s = stats.value
  if (!s) return []
  return [
    { label: t('stats.kpi.sent'), value: String(s.sent), sub: t('stats.kpi.sent_week', { n: s.sent_this_week }) },
    { label: t('stats.kpi.reply_rate'), value: `${percent(s.replied, s.sent)} %`, sub: t('stats.kpi.of', { a: s.replied, b: s.sent }) },
    { label: t('stats.kpi.interview_rate'), value: `${percent(s.interviewed, s.sent)} %`, sub: t('stats.kpi.of', { a: s.interviewed, b: s.sent }) },
    { label: t('stats.kpi.first_reply'), value: days(s.avg_first_reply_days), sub: t('stats.kpi.first_reply_sub') },
  ]
})
</script>

<template>
  <div class="px-5 sm:px-[30px] pt-[22px] pb-12 @container">
    <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[40px]">{{ t('common.loading') }}</div>
    <div v-else-if="loadError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[40px]">{{ loadError }}</div>

    <!-- État vide : trop peu d'envois pour des pourcentages significatifs -->
    <div v-else-if="stats && !stats.enough_data" class="pt-[18px]">
      <div class="border border-neutral-200 rounded-xl bg-white p-9 text-center">
        <div class="text-[14px] font-medium">{{ t('stats.empty_title') }}</div>
        <div class="text-[12.5px] text-neutral-500 mt-1">{{ t('stats.empty_body', { n: stats.min_sent }) }}</div>
      </div>
    </div>

    <div v-else-if="stats" class="flex flex-col gap-4">
      <div class="grid grid-cols-1 @[520px]:grid-cols-2 @[900px]:grid-cols-4 gap-3">
        <StatsKpi v-for="k in kpis" :key="k.label" :label="k.label" :value="k.value" :sub="k.sub" />
      </div>

      <div class="grid grid-cols-1 @[900px]:grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)] gap-4">
        <FunnelCard :stats="stats" />
        <OutcomesCard :stats="stats" />
      </div>

      <WeeklyActivityCard :weeks="stats.weeks" />

      <h3 class="text-[15px] font-semibold text-neutral-950 mt-[10px] mb-0">{{ t('stats.works.title') }}</h3>
      <div class="text-[12px] text-neutral-500 -mt-[6px] mb-0">{{ t('stats.works.note', { n: stats.min_sent_per_row }) }}</div>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(min(360px,100%),1fr))] gap-4">
        <ComparisonCard v-for="c in stats.comparisons" :key="c.criterion" :comparison="c" :min-sent="stats.min_sent_per_row" />
      </div>

      <h3 class="text-[15px] font-semibold text-neutral-950 mt-[10px] mb-0">{{ t('stats.delays_title') }}</h3>
      <div class="grid grid-cols-1 @[620px]:grid-cols-2 @[960px]:grid-cols-3 gap-4">
        <InterviewsCard :types="stats.interview_types" />
        <StatsCard :title="t('stats.durations.title')" :hint="t('stats.durations.hint')">
          <KeyValueRow :label="t('stats.durations.to_offer')" :value="days(stats.median_days.to_offer)" />
          <KeyValueRow :label="t('stats.durations.to_rejection')" :value="days(stats.median_days.to_rejection)" />
          <KeyValueRow :label="t('stats.durations.to_ghosting')" :value="days(stats.median_days.to_ghosting)" />
        </StatsCard>
        <StatsCard :title="t('stats.follow_ups.title')" :hint="t('stats.follow_ups.hint')">
          <KeyValueRow :label="t('stats.follow_ups.sent')" :value="String(stats.follow_ups.sent)" />
          <KeyValueRow
            :label="t('stats.follow_ups.answered')"
            :value="`${stats.follow_ups.answered} · ${percent(stats.follow_ups.answered, stats.follow_ups.sent)} %`"
          />
          <KeyValueRow :label="t('stats.follow_ups.delay')" :value="days(stats.follow_ups.avg_days_before)" />
        </StatsCard>
      </div>
    </div>
  </div>
</template>
