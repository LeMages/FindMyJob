<script setup lang="ts">
/**
 * En-tête de la fiche candidature : avatar de l'entreprise, poste, pastille
 * de statut, entreprise et lieu, puis l'action propre au statut — supprimer
 * et « Postuler » (à postuler), « Ajouter un événement » (en cours), rien
 * (clôturée ou acceptée). Actions désactivées avec infobulle quand la session
 * est en lecture seule. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import IconButton from '@/components/IconButton.vue'
import UiIcon from '@/components/UiIcon.vue'

// Importation des modules internes
import {
  TRASHED_STATUSES,
  avatarColor,
  closedStatusColor,
  nameInitials,
  statusColor,
  statusLabel,
} from '@/composables/candidature_display'
import type { CandidatureDetail } from '@/types'

const props = defineProps<{
  candidature: CandidatureDetail
  /** Infobulle des actions bloquées (session en lecture seule), sinon `null`. */
  lockedHint: string | null
}>()

defineEmits<{ apply: []; delete: []; 'add-event': [] }>()

const { t } = useI18n()

/** Statuts terminaux : plus aucun événement ne peut être ajouté (miroir de `available_events`). */
const FINAL_STATUSES = [...TRASHED_STATUSES, 'accepte']

const status = computed(() => props.candidature.status)
const pillColor = computed(() =>
  TRASHED_STATUSES.includes(status.value) ? closedStatusColor(status.value) : statusColor(status.value),
)
const canAddEvent = computed(() => status.value !== 'a_postuler' && !FINAL_STATUSES.includes(status.value))

const PRIMARY = 'h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none'
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-4 gap-y-3 mt-1 mb-[18px]">
    <div
      class="w-11 h-11 rounded-10 flex-none flex items-center justify-center font-semibold text-[15px]"
      :class="avatarColor(candidature.id)"
      aria-hidden="true"
    >{{ nameInitials(candidature.entreprise) }}</div>
    <div class="flex-1 min-w-[220px]">
      <div class="flex items-center gap-3 flex-wrap">
        <h1 class="text-2xl font-semibold leading-tight">{{ candidature.poste }}</h1>
        <span class="inline-block font-medium text-[12px] rounded-5 px-[9px] py-1" :class="pillColor">{{ statusLabel(status) }}</span>
      </div>
      <div class="text-[13px] text-neutral-500 mt-1">
        {{ candidature.entreprise ?? '—' }}<template v-if="candidature.localisation"> · {{ candidature.localisation }}</template>
      </div>
    </div>
    <div v-if="status === 'a_postuler'" class="flex items-center gap-2">
      <IconButton
        :title="t('detail.actions.delete')" :tooltip="lockedHint"
        danger
        :size="32"
        :disabled="lockedHint !== null"
        @click="$emit('delete')"
      ><UiIcon name="trash" /></IconButton>
      <span class="inline-flex" :title="lockedHint ?? undefined">
        <button type="button" :class="PRIMARY" :disabled="lockedHint !== null" @click="$emit('apply')">{{ t('detail.actions.apply') }}</button>
      </span>
    </div>
    <span v-else-if="canAddEvent" class="inline-flex" :title="lockedHint ?? undefined">
      <button type="button" :class="PRIMARY" :disabled="lockedHint !== null" @click="$emit('add-event')">
        <UiIcon name="plus" /> {{ t('detail.actions.add_event') }}
      </button>
    </span>
  </div>
</template>
