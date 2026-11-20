<script setup lang="ts">
/**
 * Barre de progression en cinq étapes de la fiche candidature. Pour une
 * candidature clôturée, l'étape où elle s'est arrêtée passe en rose avec le
 * libellé du statut final, les étapes franchies en gris (voir
 * `application_progress.ts`). Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { statusLabel } from '@/composables/candidature_display'
import { applicationProgress } from '../application_progress'
import type { StatusHistoryView } from '@/types'

const props = defineProps<{
  status: string
  history: StatusHistoryView[]
}>()

const { t } = useI18n()

const progress = computed(() => applicationProgress(props.status, props.history))

const STAGE_LABEL_KEYS = {
  to_apply: 'detail.progress.to_apply',
  sent: 'detail.progress.sent',
  interviews: 'detail.progress.interviews',
  offer: 'detail.progress.offer',
  accepted: 'detail.progress.accepted',
} as const

const LABEL_CLASSES = {
  done: 'text-neutral-600',
  on: 'text-neutral-950 font-semibold',
  stop: 'text-rose-700 font-semibold',
  todo: 'text-neutral-400',
}

/** Couleur du segment : gris pour les étapes franchies d'une candidature clôturée. */
function barClass(state: 'done' | 'on' | 'stop' | 'todo'): string {
  if (state === 'stop') return 'bg-rose-400'
  if (state === 'on') return 'bg-neutral-950'
  if (state === 'done') return progress.value.closed ? 'bg-neutral-400' : 'bg-neutral-950'
  return 'bg-neutral-200'
}
</script>

<template>
  <ol class="grid grid-cols-5 gap-[6px] list-none m-0 mb-5 p-0">
    <li v-for="stage in progress.stages" :key="stage.key" class="min-w-0" :aria-current="stage.state === 'on' || stage.state === 'stop' ? 'step' : undefined">
      <div class="h-[5px] rounded-[3px]" :class="barClass(stage.state)" />
      <span class="block text-[11.5px] mt-[7px] truncate" :class="LABEL_CLASSES[stage.state]">
        {{ stage.state === 'stop' ? statusLabel(status) : t(STAGE_LABEL_KEYS[stage.key]) }}
      </span>
    </li>
  </ol>
</template>
