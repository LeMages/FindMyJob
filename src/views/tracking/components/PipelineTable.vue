<script setup lang="ts">
/**
 * Onglet « Candidatures » du suivi : filtres rapides avec compteurs, tableau
 * des candidatures ouvertes, puis groupe « Clôturées » repliable résumant
 * les issues (refus, abandon, ghosté, offre déclinée).
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import PipelineRow from '@/views/tracking/components/PipelineRow.vue'

// Importation des modules internes
import {
  PIPELINE_FILTERS,
  closedSummary,
  filterCounts,
  pipelineView,
  type PipelineFilter,
} from '@/views/tracking/components/pipeline_groups'
import type { CandidatureView } from '@/types'

const { t } = useI18n()

const props = defineProps<{ candidatures: CandidatureView[] }>()
defineEmits<{ open: [candidature: CandidatureView] }>()

const filter = ref<PipelineFilter>('all')
const closedExpanded = ref(false)

const counts = computed(() => filterCounts(props.candidatures))
const view = computed(() => pipelineView(props.candidatures, filter.value))
const closedOpen = computed(() => view.value.closedAlwaysOpen || closedExpanded.value)
const summary = computed(() =>
  closedSummary(view.value.closed).map((p) => t(`tracking.closed_outcome.${p.outcome}`, { n: p.count }, p.count)).join(' · '),
)
const isEmpty = computed(() => !view.value.open.length && !view.value.showClosedGroup)

function toggleClosed() {
  if (!view.value.closedAlwaysOpen) closedExpanded.value = !closedExpanded.value
}

const TH = 'py-3 px-4 font-semibold font-mono text-[11px] tracking-[.06em] text-white whitespace-nowrap'
</script>

<template>
  <div class="px-5 sm:px-[30px] pt-[18px] pb-[30px]">
    <div class="flex flex-wrap items-center gap-2 mb-[14px]">
      <button
        v-for="f in PIPELINE_FILTERS"
        :key="f"
        type="button"
        class="inline-flex items-center gap-[7px] h-[30px] px-3 rounded-full border text-[12.5px] font-sans cursor-pointer"
        :class="filter === f
          ? 'bg-neutral-950 border-neutral-950 text-white'
          : 'bg-white border-neutral-200 text-neutral-700 hover:border-neutral-300 hover:text-neutral-950'"
        @click="filter = f"
      >
        {{ t(`tracking.filters.${f}`) }}
        <span class="font-mono text-[11px] text-neutral-400">{{ counts[f] }}</span>
      </button>
    </div>

    <!-- Conteneur de requête : la colonne « Candidaté » disparaît sous 620 px -->
    <div class="@container overflow-x-auto bg-surface-1 border border-neutral-200 rounded-10">
      <table class="w-full min-w-[520px] text-[13px] border-collapse">
        <thead>
          <tr class="text-left bg-neutral-950">
            <th :class="TH">{{ t('tracking.columns.company') }}</th>
            <th :class="TH">{{ t('tracking.columns.role') }}</th>
            <th :class="TH">{{ t('tracking.columns.status') }}</th>
            <th :class="TH" class="hidden @[620px]:table-cell">{{ t('tracking.columns.applied') }}</th>
            <th :class="TH">{{ filter === 'closed' ? t('tracking.columns.closed_on') : t('tracking.columns.inactivity') }}</th>
          </tr>
        </thead>
        <tbody>
          <PipelineRow v-for="c in view.open" :key="c.id" :candidature="c" :closed="false" @open="$emit('open', c)" />

          <tr
            v-if="view.showClosedGroup"
            class="group/closed"
            :class="view.closedAlwaysOpen ? '' : 'cursor-pointer'"
            @click="toggleClosed"
          >
            <td colspan="5" class="py-[10px] px-4 bg-neutral-50 text-[12.5px] text-neutral-700 border-t border-neutral-200 border-b border-b-neutral-100 group-hover/closed:bg-neutral-100">
              <span class="inline-block w-3 text-neutral-500 transition-transform duration-150" :class="closedOpen ? 'rotate-90' : ''">›</span>
              <strong>{{ t('tracking.filters.closed') }}</strong>
              <span class="font-mono text-[11px] text-neutral-500 ml-1 mr-[10px]">{{ view.closed.length }}</span>
              <span class="text-neutral-500">{{ summary }}</span>
            </td>
          </tr>
          <template v-if="view.showClosedGroup && closedOpen">
            <PipelineRow v-for="c in view.closed" :key="c.id" :candidature="c" :closed="true" @open="$emit('open', c)" />
          </template>
        </tbody>
      </table>
    </div>

    <div v-if="isEmpty" class="text-[12px] text-neutral-400 text-center py-6 font-mono">
      {{ candidatures.length ? t('tracking.filter_empty') : t('tracking.empty') }}
    </div>
  </div>
</template>
