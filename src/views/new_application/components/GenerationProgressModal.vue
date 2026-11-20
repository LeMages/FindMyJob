<script setup lang="ts">
/**
 * Popup bloquante affichée pendant la génération du CV et de la lettre de
 * motivation d'une candidature fraîchement créée (voir `useGenerationProgress`
 * dans `NewApplicationView.vue`) : indicateur d'étapes, barre de progression
 * et liste des étapes du pipeline (faites, en cours, à venir). Sans bouton
 * fermer tant que la génération est en cours ; en cas d'échec, l'étape fautive
 * passe en rouge, le message s'affiche et un bouton Fermer apparaît.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import UiIcon from '@/components/UiIcon.vue'
import ApplicationStepper from './ApplicationStepper.vue'

// Importation des modules internes
import { backendTextMessage } from '@/composables/backend_error_message'
import { generationStepStates } from '../generation_steps'
import type { GenerationProgress } from '@/types'

const props = defineProps<{
  progress: GenerationProgress | null
  pollError: string | null
}>()

defineEmits<{ close: [] }>()

const { t, te } = useI18n()

const failed = computed(() => props.progress?.status === 'failed' || props.pollError !== null)

const view = computed(() => generationStepStates(props.progress, failed.value))

/**
 * Libellé d'une étape, traduit depuis son index (`generation.step.<i>`) sans
 * les points de suspension ; le `step_label` du backend (en français) n'est
 * affiché tel quel que pour l'étape courante inconnue du frontend.
 *
 * @param index - Index de l'étape.
 * @returns Libellé à afficher.
 */
function stepLabel(index: number): string {
  const key = `generation.step.${index}`
  if (te(key)) return t(key).replace(/…$/, '')
  return index === props.progress?.current_step ? props.progress.step_label : ''
}

/** Message d'échec : erreur libre du backend (traduite en anglais) ou erreur de suivi. */
const failureMessage = computed(() => {
  const error = props.progress?.error
  if (error) return backendTextMessage(error, t('generation.failed'))
  return props.pollError
})

const ROW_CLASSES = {
  done: 'text-neutral-500',
  run: 'text-neutral-950 font-medium',
  todo: 'text-neutral-400',
  fail: 'text-red-600 font-medium',
}
const ICON_CLASSES = {
  done: 'bg-green-500 border-green-500 text-white',
  run: 'border-0 text-neutral-950',
  todo: 'border-neutral-200',
  fail: 'bg-red-50 border-red-200 text-red-600',
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/45 z-[1000] flex items-center justify-center p-4">
      <div
        class="bg-white rounded-2xl shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[min(520px,calc(100vw-32px))] max-h-[88vh] overflow-y-auto px-7 py-[26px]"
        role="dialog"
        aria-modal="true"
        :aria-label="failed ? t('generation.failed_title') : t('generation.running_title')"
      >
        <ApplicationStepper :active="2" class="mb-5" />
        <div class="text-[17px] font-semibold text-neutral-950">{{ failed ? t('generation.failed_title') : t('generation.running_title') }}</div>
        <div class="text-[12.5px] text-neutral-500 mt-[3px] mb-4">{{ failed ? t('generation.failed_hint') : t('generation.running_hint') }}</div>

        <div class="h-[6px] rounded-[3px] bg-neutral-100 overflow-hidden mb-4">
          <div
            class="h-full rounded-[3px] transition-[width] duration-300"
            :class="failed ? 'bg-red-500' : 'bg-neutral-950'"
            :style="{ width: `${view.percent}%` }"
          />
        </div>

        <ul class="list-none mt-0 mb-[14px] p-0 flex flex-col gap-[9px]">
          <li v-for="(state, i) in view.states" :key="i" class="flex items-center gap-[10px] text-[13px]" :class="ROW_CLASSES[state]">
            <span class="w-[18px] h-[18px] rounded-full flex items-center justify-center flex-none border-[1.5px]" :class="ICON_CLASSES[state]">
              <UiIcon v-if="state === 'done'" name="check" />
              <UiIcon v-else-if="state === 'run'" name="spinner" :size="13" />
              <UiIcon v-else-if="state === 'fail'" name="x" />
            </span>
            {{ stepLabel(i) }}
          </li>
        </ul>

        <template v-if="failed">
          <InfoBanner tone="err">{{ failureMessage }}</InfoBanner>
          <div class="flex justify-end">
            <button
              type="button"
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
              @click="$emit('close')"
            >{{ t('common.close') }}</button>
          </div>
        </template>
      </div>
    </div>
  </Teleport>
</template>
