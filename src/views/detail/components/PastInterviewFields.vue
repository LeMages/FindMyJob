<script setup lang="ts">
/**
 * Champs de l'événement « Entretien passé » : les entretiens en attente en
 * cartes à choisir (type et date lisible), puis l'issue en deux boutons —
 * Positif (vert) ou Négatif (rouge). Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { interviewTypeLabel } from '@/composables/candidature_display'
import { formatDateTime } from '@/composables/date_time_format'
import type { InterviewView } from '@/types'

defineProps<{ interviews: InterviewView[] }>()

const selectedId = defineModel<number | ''>('selectedId', { required: true })
const issue = defineModel<'positif' | 'negatif'>('issue', { required: true })

const { t } = useI18n()

const ISSUE_BUTTON = 'h-10 rounded-10 border font-sans text-[13px] font-medium cursor-pointer'
</script>

<template>
  <div class="flex flex-col gap-[14px]">
    <div>
      <span class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.add_event.interview') }}</span>
      <div class="flex flex-col gap-2" role="radiogroup" :aria-label="t('detail.add_event.interview')">
        <button
          v-for="iv in interviews"
          :key="iv.id"
          type="button"
          role="radio"
          :aria-checked="selectedId === iv.id"
          class="flex items-center gap-3 px-[14px] py-[11px] border rounded-10 bg-transparent cursor-pointer font-sans text-left"
          :class="selectedId === iv.id ? 'border-neutral-950 shadow-[0_0_0_1px_var(--color-neutral-950)]' : 'border-neutral-200'"
          @click="selectedId = iv.id"
        >
          <span
            class="w-4 h-4 rounded-full flex-none"
            :class="selectedId === iv.id ? 'border-[5px] border-neutral-950' : 'border-[1.5px] border-neutral-300'"
          />
          <span>
            <span class="block text-[13.5px] font-semibold text-neutral-950">{{ interviewTypeLabel(iv.type) }}</span>
            <span class="text-[12px] text-neutral-500">{{ iv.date_entretien ? formatDateTime(iv.date_entretien) : t('common.date_missing') }}</span>
          </span>
        </button>
      </div>
    </div>
    <div>
      <span class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.add_event.issue') }}</span>
      <div class="grid grid-cols-2 gap-2" role="radiogroup" :aria-label="t('detail.add_event.issue')">
        <button
          type="button"
          role="radio"
          :aria-checked="issue === 'positif'"
          :class="[ISSUE_BUTTON, issue === 'positif' ? 'bg-green-50 border-green-500 text-green-700' : 'bg-transparent border-neutral-200 text-neutral-700']"
          @click="issue = 'positif'"
        >{{ t('interview.issue.positif') }}</button>
        <button
          type="button"
          role="radio"
          :aria-checked="issue === 'negatif'"
          :class="[ISSUE_BUTTON, issue === 'negatif' ? 'bg-red-50 border-red-400 text-red-600' : 'bg-transparent border-neutral-200 text-neutral-700']"
          @click="issue = 'negatif'"
        >{{ t('interview.issue.negatif') }}</button>
      </div>
    </div>
  </div>
</template>
