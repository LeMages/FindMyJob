<script setup lang="ts">
/**
 * Vue « Paramètres » : modèle Claude, clé API, polices, thème d'affichage,
 * langue de l'interface.
 */

// Importation des modules Vue / framework
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import ApiKeySection from '@/views/settings/components/ApiKeySection.vue'
import FontsSection from '@/views/settings/components/FontsSection.vue'
import LanguageSection from '@/views/settings/components/LanguageSection.vue'

// Importation des modules internes
import { backendErrorMessage, backendTextMessage } from '@/composables/backend_error_message'
import { useTheme } from '@/composables/useTheme'
import type { TestConnectionResult, ThemeSetting } from '@/types'

const { t } = useI18n()

const THEMES: ThemeSetting[] = ['clair', 'sombre', 'systeme']

/** Modèles proposés : identifiants inchangés, libellés traduits. */
const modelOptions = computed<{ value: string; label: string }[]>(() => [
  { value: 'claude-sonnet-4-6', label: t('settings.ai.models.sonnet') },
  { value: 'claude-haiku-4-5-20251001', label: t('settings.ai.models.haiku') },
])

const selectedModel = ref('claude-sonnet-4-6')

const testing = ref(false)
const testResult = ref<TestConnectionResult | null>(null)

const { theme, setTheme } = useTheme()
const themeFeedback = ref<string | null>(null)

async function loadModel() {
  try {
    selectedModel.value = await invoke<string>('get_llm_model')
  } catch (e: unknown) {
    console.error('Impossible de charger le modèle configuré.', e)
  }
}

onMounted(() => {
  void loadModel()
})

async function onModelChange() {
  try {
    await invoke('set_llm_model', { model: selectedModel.value })
  } catch (e: unknown) {
    console.error('Impossible d\'enregistrer le modèle sélectionné.', e)
  }
}

async function testConnection() {
  testing.value = true
  testResult.value = null
  try {
    const result = await invoke<TestConnectionResult>('test_api_connection')
    // Un échec renvoyé par le backend porte un message libre en français.
    testResult.value = result.success
      ? result
      : { ...result, message: backendTextMessage(result.message, t('error.fallback.connection_test')) }
  } catch (e: unknown) {
    testResult.value = {
      success: false,
      model: selectedModel.value,
      message: backendErrorMessage(e, t('error.fallback.connection_test')),
      latency_ms: 0,
    }
  } finally {
    testing.value = false
  }
}

async function onThemeChange(value: ThemeSetting) {
  themeFeedback.value = await setTheme(value)
}
</script>

<template>
  <div class="px-9 pt-[26px] pb-11">
    <h1 class="text-2xl font-semibold leading-tight">{{ t('settings.title') }}</h1>
    <div class="text-[13px] text-neutral-500 mt-[5px] font-mono mb-[22px]">{{ t('settings.subtitle') }}</div>

    <!-- Intelligence artificielle -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">{{ t('settings.ai.section') }}</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('settings.ai.model') }}</label>
        <select
          v-model="selectedModel"
          class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 w-full font-sans cursor-pointer"
          @change="onModelChange"
        >
          <option v-for="opt in modelOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>

      <ApiKeySection @saved="testConnection" />

      <div class="flex items-center justify-between gap-5 pt-2">
        <div>
          <div class="text-[14px] font-medium">{{ t('settings.ai.test.title') }}</div>
          <div class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">{{ t('settings.ai.test.description') }}</div>
          <div v-if="testResult" class="text-[12.5px] font-mono mt-[6px]" :class="testResult.success ? 'text-green-600' : 'text-red-500'">
            {{ testResult.success ? t('settings.ai.test.success', { latency: testResult.latency_ms }) : testResult.message }}
          </div>
        </div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
          :disabled="testing"
          @click="testConnection"
        >{{ testing ? t('settings.ai.test.running') : t('settings.ai.test.button') }}</button>
      </div>
    </div>

    <!-- Polices -->
    <FontsSection />

    <!-- Apparence -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">{{ t('settings.appearance.section') }}</div>
      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('settings.appearance.theme_label') }}</label>
        <div class="flex gap-2">
          <button
            v-for="value in THEMES"
            :key="value"
            type="button"
            class="flex-1 h-[38px] rounded-lg flex items-center justify-center text-[13px] cursor-pointer font-sans"
            :class="theme === value ? 'border-2 border-neutral-950 font-semibold' : 'border border-neutral-300 hover:bg-surface-2'"
            @click="onThemeChange(value)"
          >
            {{ t(`settings.appearance.theme.${value}`) }}
          </button>
        </div>
        <div v-if="themeFeedback" class="text-[11.5px] font-mono mt-[6px] text-red-500">{{ themeFeedback }}</div>
      </div>
    </div>

    <!-- Langue de l'interface -->
    <LanguageSection />
  </div>
</template>
