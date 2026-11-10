<script setup lang="ts">
/**
 * Popup lecture seule de détail d'un entretien, ouverte depuis la vue
 * semaine ou mois de l'onglet planning. Propre à `PlanningPanel`.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { interviewTypeLabel, interviewTypeColor, modaliteLabel } from '@/composables/candidature_display'
import { parseInterviewDate, formatFullDate } from './planning_dates'
import type { PlanningInterviewView } from '@/types'

const { t } = useI18n()

const visible = ref(false)
const interview = ref<PlanningInterviewView | null>(null)

const date = computed(() => (interview.value ? parseInterviewDate(interview.value.date_entretien) : null))

/** Badge d'issue de l'entretien (couleur + libellé), `null` si pas encore passé. */
const issueBadge = computed(() => {
  switch (interview.value?.issue) {
    case 'positif':
      return { label: t('interview.issue.positif'), classes: 'bg-green-50 border-green-200 text-green-700' }
    case 'negatif':
      return { label: t('interview.issue.negatif'), classes: 'bg-red-50 border-red-200 text-red-700' }
    default:
      return { label: t('interview.issue.pending'), classes: 'bg-neutral-100 border-neutral-300 text-neutral-600' }
  }
})

/**
 * Ouvre la popup sur l'entretien donné.
 *
 * @param iv - Entretien à afficher.
 */
function open(iv: PlanningInterviewView) {
  interview.value = iv
  visible.value = true
}

/** Ferme la popup. */
function close() {
  visible.value = false
}

defineExpose({ open })
</script>

<template>
  <div v-if="visible && interview" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="close">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[480px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div>
          <div class="text-[17px] font-semibold">{{ interview.entreprise ?? t('common.unknown_company') }}</div>
          <div class="text-[13px] text-neutral-500">{{ interview.poste }}</div>
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="close">×</button>
      </div>

      <div class="text-[13.5px] text-neutral-800 mb-4 font-mono">
        {{ date ? formatFullDate(date) : t('common.date_missing') }}
      </div>

      <div class="flex flex-wrap gap-[8px] mb-5">
        <span class="inline-block font-medium text-[12px] rounded-5 px-[9px] py-1 border" :class="interviewTypeColor(interview.type)">
          {{ interviewTypeLabel(interview.type) }}
        </span>
        <span class="inline-block font-medium text-[12px] rounded-5 px-[9px] py-1 border" :class="issueBadge.classes">
          {{ issueBadge.label }}
        </span>
      </div>

      <div class="mb-[14px]">
        <div class="text-[12px] text-neutral-600 mb-[5px]">{{ t('interview.detail.modality') }}</div>
        <div class="text-[13.5px] text-neutral-950">
          {{ interview.modalite ? modaliteLabel(interview.modalite) : (interview.lieu ?? t('interview.detail.modality_missing')) }}
        </div>
      </div>

      <div v-if="interview.lieu" class="mb-[14px]">
        <div class="text-[12px] text-neutral-600 mb-[5px] flex items-center gap-1">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" /><circle cx="12" cy="10" r="3" /></svg>
          {{ t('interview.detail.location') }}
        </div>
        <div class="text-[13.5px] text-neutral-950">{{ interview.lieu }}</div>
      </div>

      <div v-if="interview.contacts.length > 0" class="mb-[14px]">
        <div class="text-[12px] text-neutral-600 mb-[5px]">{{ t('interview.detail.contacts') }}</div>
        <div v-for="contact in interview.contacts" :key="contact.id" class="text-[13.5px] text-neutral-950">
          {{ contact.nom }}<span v-if="contact.role" class="text-neutral-500"> — {{ contact.role }}</span>
        </div>
      </div>

      <div v-if="interview.notes" class="mb-[14px]">
        <div class="text-[12px] text-neutral-600 mb-[5px]">{{ t('interview.detail.notes') }}</div>
        <div class="text-[13.5px] text-neutral-950 whitespace-pre-wrap leading-[1.5]">{{ interview.notes }}</div>
      </div>
    </div>
  </div>
</template>
