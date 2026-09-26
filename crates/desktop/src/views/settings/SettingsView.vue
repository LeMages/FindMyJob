<script setup lang="ts">
/**
 * Vue « Paramètres » : modèle Claude, polices, thème d'affichage, export
 * des données en JSON.
 */

// Importation des modules Vue / framework
import { onMounted, ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import FontsSection from '@/views/settings/components/FontsSection.vue'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import { useTheme } from '@/composables/useTheme'
import type { TestConnectionResult, ThemeSetting } from '@/types'

const MODEL_OPTIONS: { value: string; label: string }[] = [
  { value: 'claude-sonnet-4-6', label: 'Claude Sonnet 4.6 (recommandé)' },
  { value: 'claude-haiku-4-5-20251001', label: 'Claude Haiku 4.5 (rapide, économique)' },
]

const selectedModel = ref('claude-sonnet-4-6')

const testing = ref(false)
const testResult = ref<TestConnectionResult | null>(null)

const { theme, setTheme } = useTheme()
const themeFeedback = ref<string | null>(null)

const exporting = ref(false)
const exportFeedback = ref<{ type: 'success' | 'error'; text: string } | null>(null)

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
    testResult.value = await invoke<TestConnectionResult>('test_api_connection')
  } catch (e: unknown) {
    testResult.value = {
      success: false,
      model: selectedModel.value,
      message: extractErrorMessage(e, 'Le test de connexion a échoué.'),
      latency_ms: 0,
    }
  } finally {
    testing.value = false
  }
}

async function onThemeChange(value: ThemeSetting) {
  themeFeedback.value = await setTheme(value)
}

async function exportData() {
  exporting.value = true
  exportFeedback.value = null
  try {
    const path = await invoke<string | null>('export_data_json')
    if (path) {
      exportFeedback.value = { type: 'success', text: `Export enregistré : ${path}` }
    }
    // path === null : annulation par l'utilisateur, comportement neutre voulu.
  } catch (e: unknown) {
    exportFeedback.value = { type: 'error', text: extractErrorMessage(e, "Impossible d'exporter les données.") }
  } finally {
    exporting.value = false
  }
}
</script>

<template>
  <div class="px-9 pt-[26px] pb-11">
    <h1 class="text-2xl font-semibold leading-tight">Paramètres</h1>
    <div class="text-[13px] text-neutral-500 mt-[5px] font-mono mb-[22px]">Configuration de l'application FindMyJob.</div>

    <!-- Intelligence artificielle -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">INTELLIGENCE ARTIFICIELLE</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Modèle</label>
        <select
          v-model="selectedModel"
          class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 w-full font-sans cursor-pointer"
          @change="onModelChange"
        >
          <option v-for="opt in MODEL_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>

      <div class="flex items-center justify-between gap-5 pt-2">
        <div>
          <div class="text-[14px] font-medium">Tester la connexion</div>
          <div class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">Vérifie que la clé et le modèle sélectionné répondent correctement.</div>
          <div v-if="testResult" class="text-[12.5px] font-mono mt-[6px]" :class="testResult.success ? 'text-green-600' : 'text-red-500'">
            {{ testResult.success ? `Connexion réussie (${testResult.latency_ms}ms)` : testResult.message }}
          </div>
        </div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
          :disabled="testing"
          @click="testConnection"
        >{{ testing ? 'Test en cours…' : 'Tester →' }}</button>
      </div>
    </div>

    <!-- Polices -->
    <FontsSection />

    <!-- Apparence -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">APPARENCE</div>
      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Thème</label>
        <div class="flex gap-2">
          <button
            v-for="t in (['clair', 'sombre', 'systeme'] as const)"
            :key="t"
            type="button"
            class="flex-1 h-[38px] rounded-lg flex items-center justify-center text-[13px] cursor-pointer capitalize font-sans"
            :class="theme === t ? 'border-2 border-neutral-950 font-semibold' : 'border border-neutral-300 hover:bg-surface-2'"
            @click="onThemeChange(t)"
          >
            {{ t.charAt(0).toUpperCase() + t.slice(1) }}
          </button>
        </div>
        <div v-if="themeFeedback" class="text-[11.5px] font-mono mt-[6px] text-red-500">{{ themeFeedback }}</div>
      </div>
    </div>

    <!-- Données -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DONNÉES</div>
      <div class="flex items-center justify-between gap-5">
        <div>
          <div class="text-[14px] font-medium">Sauvegarde / Export</div>
          <div class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">Exporter toutes les données en JSON pour archivage.</div>
          <div v-if="exportFeedback" class="text-[12.5px] font-mono mt-[6px]" :class="exportFeedback.type === 'success' ? 'text-green-600' : 'text-red-500'">
            {{ exportFeedback.text }}
          </div>
        </div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
          :disabled="exporting"
          @click="exportData"
        >{{ exporting ? 'Export en cours…' : 'Exporter' }}</button>
      </div>
    </div>
  </div>
</template>
