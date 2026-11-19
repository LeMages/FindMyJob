<script setup lang="ts">
/**
 * Section « Interface » des Paramètres : thème en trois miniatures et langue
 * de l'interface en sélecteur segmenté FR / EN, appliqués à chaud et
 * persistés. La langue de l'interface est sans effet sur celle des documents
 * générés, qui suit toujours celle de l'offre.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import SettingsRow from '@/components/SettingsRow.vue'
import SettingsSection from '@/components/SettingsSection.vue'
import ThemeThumbnail from '@/views/settings/components/ThemeThumbnail.vue'

// Importation des modules internes
import { UI_LANGUAGE_OPTIONS, useUiLanguage } from '@/composables/useUiLanguage'
import { useTheme } from '@/composables/useTheme'
import type { ThemeSetting, UiLanguage } from '@/types'

const { t } = useI18n()
const { theme, setTheme } = useTheme()
const { uiLanguage, setUiLanguage } = useUiLanguage()

const THEMES: ThemeSetting[] = ['clair', 'sombre', 'systeme']

const themeFeedback = ref<string | null>(null)
const languageFeedback = ref<string | null>(null)

async function onThemeChange(value: ThemeSetting) {
  themeFeedback.value = await setTheme(value)
}

/** Applique et persiste la langue choisie. */
async function onLanguageChange(value: UiLanguage) {
  languageFeedback.value = await setUiLanguage(value)
}
</script>

<template>
  <SettingsSection :title="t('settings.appearance.section')">
    <SettingsRow :label="t('settings.appearance.theme_label')" :help="t('settings.appearance.theme_help')">
      <div class="flex gap-[10px]">
        <button
          v-for="value in THEMES"
          :key="value"
          type="button"
          class="flex flex-col items-center gap-[6px] bg-transparent border-0 p-0 cursor-pointer font-sans"
          @click="onThemeChange(value)"
        >
          <ThemeThumbnail :theme="value" :selected="theme === value" />
          <span class="text-[12px]" :class="theme === value ? 'text-neutral-950 font-medium' : 'text-neutral-600'">{{ t(`settings.appearance.theme.${value}`) }}</span>
        </button>
      </div>
      <template #below>
        <div v-if="themeFeedback" class="text-[12px] font-mono mt-2 text-red-500">{{ themeFeedback }}</div>
      </template>
    </SettingsRow>

    <SettingsRow :label="t('settings.language.label')" :help="t('settings.language.help')">
      <div class="inline-flex p-[3px] rounded-9 bg-neutral-100 gap-0.5">
        <button
          v-for="lang in UI_LANGUAGE_OPTIONS"
          :key="lang.value"
          type="button"
          class="border-0 px-[14px] py-[5px] rounded-7 text-[12.5px] font-sans cursor-pointer"
          :class="uiLanguage === lang.value
            ? 'bg-white text-neutral-950 font-medium shadow-[0_1px_2px_rgba(0,0,0,.08)]'
            : 'bg-transparent text-neutral-600'"
          @click="onLanguageChange(lang.value)"
        >{{ lang.label }}</button>
      </div>
      <template #below>
        <div v-if="languageFeedback" class="text-[12px] font-mono mt-2 text-red-500">{{ languageFeedback }}</div>
      </template>
    </SettingsRow>
  </SettingsSection>
</template>
