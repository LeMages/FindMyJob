<script setup lang="ts">
/**
 * Carte « Suivi » de la fiche candidature : timeline verticale des
 * événements (icône, libellé, date) avec « Voir le contenu » pour un mail ou
 * un message envoyé. L'ajout d'un événement se fait depuis l'en-tête de la
 * fiche. Masquée tant que la candidature est « à postuler ». Propre à
 * `DetailView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'
import TimelineEventIcon from './TimelineEventIcon.vue'

// Importation des modules internes
import { state, openStatusHistoryPopup } from '@/stores/app'
import { eventLabel } from '@/composables/candidature_display'
import { dateLocale } from '@/i18n'

const { t } = useI18n()

/** Formate une date ISO pour l'affichage (ex. "12 juin 2025"). */
const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString(dateLocale(), { day: 'numeric', month: 'long', year: 'numeric' })
</script>

<template>
  <SectionCard
    v-if="state.detailCandidature?.status !== 'a_postuler'"
    :title="t('detail.timeline.title')"
    :hint="t('detail.timeline.count', state.detailHistory.length)"
  >
    <ol v-if="state.detailHistory.length > 0" class="list-none m-0 p-0">
      <li
        v-for="(entry, i) in state.detailHistory"
        :key="entry.id"
        class="relative flex items-start gap-[14px]"
        :class="i < state.detailHistory.length - 1 ? 'pb-[18px]' : ''"
      >
        <!-- Trait vertical reliant chaque événement au suivant -->
        <span v-if="i < state.detailHistory.length - 1" class="absolute left-4 top-[34px] bottom-[2px] w-px bg-neutral-200" aria-hidden="true" />
        <span class="w-[33px] h-[33px] rounded-full bg-neutral-100 text-neutral-700 flex items-center justify-center flex-none [&_svg]:w-[15px] [&_svg]:h-[15px]">
          <TimelineEventIcon :event-type="entry.event_type" />
        </span>
        <div class="flex-1 min-w-0 pt-0.5">
          <div class="text-[13.5px] font-medium text-neutral-950">{{ eventLabel(entry.event_type) }}</div>
          <div class="text-[12px] text-neutral-500 mt-px">{{ formatDate(entry.changed_at) }}</div>
        </div>
        <button
          v-if="entry.content"
          type="button"
          class="mt-[7px] border-0 bg-transparent p-0 font-sans text-[12.5px] text-blue-500 cursor-pointer inline-flex items-center gap-[5px] hover:underline"
          @click="openStatusHistoryPopup(entry.content)"
        ><UiIcon name="eye" /> {{ t('detail.timeline.view_content') }}</button>
      </li>
    </ol>
    <div v-else class="text-[12px] text-neutral-500 mt-2">{{ t('detail.timeline.empty') }}</div>
  </SectionCard>
</template>
