<script setup lang="ts">
/**
 * Modale bloquante affichée pendant la génération du CV et de la lettre de
 * motivation d'une candidature fraîchement créée (voir `useGenerationProgress`
 * dans `NewApplicationView.vue`) — sans bouton fermer tant que la génération
 * est en cours, un bouton apparaît uniquement en cas d'échec.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { backendTextMessage } from '@/composables/backend_error_message'
import type { GenerationProgress } from '@/types'

const props = defineProps<{
  progress: GenerationProgress | null
  pollError: string | null
}>()

defineEmits<{ close: [] }>()

const { t, te } = useI18n()

// Dernière étape traduite (`generation.step.0` à `generation.step.5`), alignée
// sur les étapes publiées par le backend (préparation puis `pipeline.rs`).
const LAST_TRANSLATED_STEP = 5

/**
 * Libellé de l'étape courante : traduit depuis `current_step`, le
 * `step_label` du backend (en français) n'étant affiché tel quel que pour
 * une étape inconnue du frontend.
 */
const stepLabel = computed(() => {
  const progress = props.progress
  if (!progress) return t('generation.starting')
  const step = progress.current_step
  const key = `generation.step.${step}`
  const known = Number.isInteger(step) && step >= 0 && step <= LAST_TRANSLATED_STEP && te(key)
  return known ? t(key) : progress.step_label
})

/** Message d'échec : erreur libre du backend (traduite en anglais) ou erreur de suivi. */
const failureMessage = computed(() => {
  const error = props.progress?.error
  if (error) return backendTextMessage(error, t('generation.failed'))
  return props.pollError
})
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/45 z-[1000] flex items-center justify-center">
      <div class="bg-surface-3 rounded-xl shadow-[0_8px_40px_rgba(0,0,0,0.22)] w-[min(440px,calc(100vw-48px))] px-6 py-6 flex flex-col gap-4">
        <span class="text-[14px] font-semibold text-neutral-950">{{ t('generation.title') }}</span>

        <div v-if="progress?.status === 'failed' || pollError" class="flex items-start gap-[10px] text-red-600 text-[13px] p-4 bg-red-50 border border-red-200 rounded-lg leading-[1.5]">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 mt-[1px]">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span>{{ failureMessage }}</span>
        </div>

        <template v-else>
          <div class="flex items-center gap-3">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin text-blue-500 shrink-0">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
            <span class="text-[13.5px] text-neutral-700">{{ stepLabel }}</span>
          </div>
          <div class="h-2 rounded-full bg-neutral-200 overflow-hidden">
            <div
              class="h-full bg-blue-500 transition-[width] duration-300"
              :style="{ width: `${((progress?.current_step ?? 0) / (progress?.total_steps ?? 6)) * 100}%` }"
            />
          </div>
          <span class="text-[12px] text-neutral-500 font-mono">{{ t('generation.step_counter', { current: progress?.current_step ?? 0, total: progress?.total_steps ?? 6 }) }}</span>
        </template>

        <button
          v-if="progress?.status === 'failed' || pollError"
          class="h-10 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 cursor-pointer font-sans hover:bg-surface-3 self-end"
          type="button"
          @click="$emit('close')"
        >{{ t('common.close') }}</button>
      </div>
    </div>
  </Teleport>
</template>
