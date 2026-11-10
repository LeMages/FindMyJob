<script setup lang="ts">
/**
 * Card « STATUT & SUIVI » de la page de détail : timeline complète des
 * événements de suivi d'une candidature (relances, entretiens, messages,
 * appels, notes...) et point d'entrée pour en ajouter un nouveau.
 * Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { state, openStatusHistoryPopup } from '@/stores/app'
import { eventLabel } from '@/composables/candidature_display'
import { dateLocale } from '@/i18n'
import TimelineEventIcon from './TimelineEventIcon.vue'
import AddEventPopup from './AddEventPopup.vue'

const { t } = useI18n()

/** Formate une date ISO pour l'affichage court (ex. "12 juin 2025"). */
const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString(dateLocale(), { day: 'numeric', month: 'long', year: 'numeric' })

// Statuts terminaux : plus aucun événement de suivi ne peut être ajouté une
// fois la candidature dans l'un de ces statuts (voir aussi
// db::timeline_events::available_events côté backend). `offre` n'y figure
// pas : c'est un statut intermédiaire, on peut encore accepter/décliner
// l'offre ou se désister.
const STATUTS_TERMINAUX = ['accepte', 'offre_refusee', 'refus', 'abandon', 'ghosting']

const canAddEvent = computed(() =>
  !!state.detailCandidature && !STATUTS_TERMINAUX.includes(state.detailCandidature.status),
)

const addEventPopup = ref<InstanceType<typeof AddEventPopup> | null>(null)
</script>

<template>
  <div v-if="state.detailCandidature?.status !== 'a_postuler'" class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">{{ t('detail.timeline.section') }}</div>

    <div v-for="entry in state.detailHistory" :key="entry.id" class="flex items-center gap-3 bg-neutral-50 border border-neutral-200 rounded-10 px-[14px] py-3 mb-[10px]">
      <div class="w-9 h-9 rounded-9 bg-surface-2 border border-neutral-200 flex items-center justify-center text-neutral-600 flex-none">
        <TimelineEventIcon :event-type="entry.event_type" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[13.5px] font-semibold">{{ eventLabel(entry.event_type) }}</div>
        <div class="text-[12px] text-neutral-500 font-mono mt-0.5">{{ formatDate(entry.changed_at) }}</div>
      </div>
      <button
        v-if="entry.content"
        class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none hover:text-neutral-950 hover:bg-surface-2"
        :title="t('detail.timeline.view_content')"
        @click="openStatusHistoryPopup(entry.content)"
      >
        <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
          <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
          <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
        </svg>
      </button>
    </div>

    <div v-if="state.detailHistory.length === 0" class="text-[12px] text-neutral-400 font-mono py-2">{{ t('detail.timeline.empty') }}</div>

    <button
      v-if="canAddEvent"
      class="w-full justify-center mt-1 h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
      @click="addEventPopup?.open()"
    >{{ t('detail.timeline.add_event') }}</button>

    <AddEventPopup ref="addEventPopup" />
  </div>
</template>
