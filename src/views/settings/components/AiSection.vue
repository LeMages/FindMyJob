<script setup lang="ts">
/**
 * Section « Intelligence artificielle » des Paramètres : choix du modèle en
 * deux cartes, clé API, test de connexion avec résultat en pastille.
 */

// Importation des modules Vue / framework
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import StatusPill from '@/components/StatusPill.vue'
import ApiKeySection from '@/views/settings/components/ApiKeySection.vue'
import SettingsRow from '@/views/settings/components/SettingsRow.vue'
import SettingsSection from '@/views/settings/components/SettingsSection.vue'

// Importation des modules internes
import { backendErrorMessage, backendTextMessage } from '@/composables/backend_error_message'
import type { TestConnectionResult } from '@/types'

const { t } = useI18n()

/** Modèles proposés : identifiants inchangés, libellés traduits. */
const modelOptions = computed(() => [
  { value: 'claude-sonnet-4-6', key: 'sonnet', recommended: true },
  { value: 'claude-haiku-4-5-20251001', key: 'haiku', recommended: false },
].map((m) => ({
  ...m,
  name: t(`settings.ai.models.${m.key}.name`),
  description: t(`settings.ai.models.${m.key}.description`),
})))

const selectedModel = ref('claude-sonnet-4-6')
const modelError = ref<string | null>(null)

const testing = ref(false)
const testResult = ref<TestConnectionResult | null>(null)

async function loadModel() {
  try {
    selectedModel.value = await invoke<string>('get_llm_model')
  } catch (e: unknown) {
    modelError.value = backendErrorMessage(e, t('settings.ai.model_load_error'))
  }
}

onMounted(() => {
  void loadModel()
})

/** Sélectionne et persiste le modèle ; le dernier résultat de test ne vaut plus. */
async function selectModel(value: string) {
  if (value === selectedModel.value) return
  const previous = selectedModel.value
  selectedModel.value = value
  testResult.value = null
  modelError.value = null
  try {
    await invoke('set_llm_model', { model: value })
  } catch (e: unknown) {
    selectedModel.value = previous
    modelError.value = backendErrorMessage(e, t('settings.ai.model_save_error'))
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
</script>

<template>
  <SettingsSection :title="t('settings.ai.section')" :description="t('settings.ai.description')">
    <SettingsRow :label="t('settings.ai.model')">
      <template #below>
        <div class="grid grid-cols-1 @[560px]:grid-cols-2 gap-[10px] mt-3">
          <button
            v-for="m in modelOptions"
            :key="m.value"
            type="button"
            class="flex items-start gap-3 px-[14px] py-3 rounded-10 bg-transparent cursor-pointer font-sans text-inherit text-left border"
            :class="selectedModel === m.value
              ? 'border-neutral-950 shadow-[0_0_0_1px_var(--color-neutral-950)]'
              : 'border-neutral-200 hover:border-neutral-300 hover:bg-neutral-50'"
            @click="selectModel(m.value)"
          >
            <span
              class="w-4 h-4 rounded-full flex-none mt-px"
              :class="selectedModel === m.value ? 'border-[5px] border-neutral-950' : 'border-[1.5px] border-neutral-300'"
            />
            <span class="flex-1 min-w-0">
              <span class="flex items-center gap-2 text-[13.5px] font-semibold text-neutral-950">
                {{ m.name }}
                <span v-if="m.recommended" class="text-[10.5px] font-medium px-[7px] py-px rounded-full bg-neutral-100 text-neutral-700">{{ t('settings.ai.recommended') }}</span>
              </span>
              <span class="block text-[12px] text-neutral-500 mt-0.5">{{ m.description }}</span>
            </span>
          </button>
        </div>
        <div v-if="modelError" class="text-[12px] font-mono mt-2 text-red-500">{{ modelError }}</div>
      </template>
    </SettingsRow>

    <ApiKeySection @saved="testConnection" />

    <SettingsRow :label="t('settings.ai.test.title')" :help="t('settings.ai.test.description')">
      <StatusPill v-if="testing" :dot="false">{{ t('settings.ai.test.running') }}</StatusPill>
      <template v-else-if="testResult">
        <StatusPill v-if="testResult.success" tone="ok">{{ t('settings.ai.test.success', { latency: testResult.latency_ms }) }}</StatusPill>
        <StatusPill v-else tone="err">{{ t('settings.ai.test.failed') }}</StatusPill>
      </template>
      <button
        type="button"
        class="h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="testing"
        @click="testConnection"
      >{{ t('settings.ai.test.button') }}</button>
      <template #below>
        <div v-if="testResult && !testResult.success" class="text-[12px] mt-2 text-red-500">{{ testResult.message }}</div>
      </template>
    </SettingsRow>
  </SettingsSection>
</template>
