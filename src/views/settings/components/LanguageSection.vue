<script setup lang="ts">
/**
 * Section « Langue » des Paramètres : choix de la langue de l'interface
 * (français / anglais), appliquée à chaud et persistée. Sans effet sur la
 * langue des documents générés, qui suit toujours celle de l'offre.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { UI_LANGUAGE_OPTIONS, useUiLanguage } from '@/composables/useUiLanguage'
import type { UiLanguage } from '@/types'

const { t } = useI18n()
const { uiLanguage, setUiLanguage } = useUiLanguage()

const feedback = ref<string | null>(null)

/** Applique et persiste la langue choisie. */
async function onLanguageChange(value: UiLanguage) {
  feedback.value = await setUiLanguage(value)
}
</script>

<template>
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">{{ t('settings.language.section') }}</div>
    <div class="mb-0">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('settings.language.label') }}</label>
      <div class="flex gap-2">
        <button
          v-for="lang in UI_LANGUAGE_OPTIONS"
          :key="lang.value"
          type="button"
          class="flex-1 h-[38px] rounded-lg flex items-center justify-center text-[13px] cursor-pointer font-sans"
          :class="uiLanguage === lang.value ? 'border-2 border-neutral-950 font-semibold' : 'border border-neutral-300 hover:bg-surface-2'"
          @click="onLanguageChange(lang.value)"
        >
          {{ lang.label }}
        </button>
      </div>
      <div v-if="feedback" class="text-[11.5px] font-mono mt-[6px] text-red-500">{{ feedback }}</div>
    </div>
  </div>
</template>
