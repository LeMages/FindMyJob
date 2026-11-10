<script setup lang="ts">
/**
 * Bloc « Clé API Anthropic » des Paramètres. En mode PROD, la clé est
 * enregistrée dans le trousseau du système (saisie, remplacement,
 * suppression). En mode DEV, elle est lue depuis le `.env` : l'état est
 * affiché en lecture seule. La clé complète ne revient jamais du backend.
 */

// Importation des modules Vue / framework
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import type { ApiKeyStatus } from '@/types'

const { t } = useI18n()

const emit = defineEmits<{
  /** Une clé vient d'être enregistrée : le parent relance le test de connexion. */
  saved: []
}>()

const status = ref<ApiKeyStatus | null>(null)
const keyInput = ref('')
const saving = ref(false)
const confirmDelete = ref(false)
const feedback = ref<string | null>(null)

const editable = computed(() => status.value?.source === 'keyring')

const statusLabel = computed(() => {
  if (!status.value) return feedback.value ? t('settings.api_key.status_unavailable') : t('common.loading')
  if (!status.value.present) return t('settings.api_key.none')
  return t('settings.api_key.configured', { last4: status.value.last4 ?? '' })
})

/** Charge l'état de la clé. */
async function loadStatus() {
  try {
    status.value = await invoke<ApiKeyStatus>('get_api_key_status')
  } catch (e: unknown) {
    feedback.value = backendErrorMessage(e, t('error.fallback.read_api_key'))
  }
}

onMounted(() => {
  void loadStatus()
})

/** Enregistre la clé saisie dans le trousseau, puis demande un test de connexion. */
async function saveKey() {
  if (!keyInput.value.trim()) return
  saving.value = true
  feedback.value = null
  try {
    status.value = await invoke<ApiKeyStatus>('save_api_key', { key: keyInput.value })
    keyInput.value = ''
    emit('saved')
  } catch (e: unknown) {
    feedback.value = backendErrorMessage(e, t('error.fallback.save_api_key'))
  } finally {
    saving.value = false
  }
}

/** Supprime la clé du trousseau après confirmation. */
async function deleteKey() {
  confirmDelete.value = false
  feedback.value = null
  try {
    status.value = await invoke<ApiKeyStatus>('delete_api_key')
  } catch (e: unknown) {
    feedback.value = backendErrorMessage(e, t('error.fallback.delete_api_key'))
  }
}
</script>

<template>
  <div class="mb-[14px]">
    <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('settings.api_key.label') }}</label>
    <div class="flex items-center justify-between gap-5">
      <div>
        <div class="text-[13.5px] font-mono" :class="status?.present ? 'text-neutral-950' : 'text-neutral-500'">{{ statusLabel }}</div>
        <div v-if="status" class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">
          {{ editable ? t('settings.api_key.source_keyring') : t('settings.api_key.source_env') }}
        </div>
      </div>
      <button
        v-if="editable && status?.present"
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 hover:text-neutral-800"
        type="button"
        @click="confirmDelete = true"
      >{{ t('common.delete') }}</button>
    </div>

    <form v-if="editable" class="flex gap-2 mt-[10px]" @submit.prevent="saveKey">
      <input
        v-model="keyInput"
        type="password"
        autocomplete="off"
        spellcheck="false"
        :placeholder="status?.present ? t('settings.api_key.placeholder_replace') : 'sk-ant-…'"
        class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 min-w-0 font-mono"
      >
      <button
        class="h-[38px] px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
        type="submit"
        :disabled="saving || !keyInput.trim()"
      >{{ saving ? t('common.saving') : t('common.save') }}</button>
    </form>

    <div v-if="feedback" class="text-[11.5px] font-mono mt-[6px] text-red-500">{{ feedback }}</div>
  </div>

  <!-- Confirmation de suppression -->
  <div
    v-if="confirmDelete"
    class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]"
    @click="confirmDelete = false"
  >
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[440px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">{{ t('settings.api_key.delete_title') }}</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="confirmDelete = false">×</button>
      </div>

      <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[6px]">{{ t('common.irreversible') }}</div>
      <div class="bg-neutral-50 border border-neutral-200 rounded-9 px-[14px] py-3 mb-5 text-[13.5px]">
        <i18n-t keypath="settings.api_key.delete_body" tag="span" scope="global">
          <template #key><strong>…{{ status?.last4 }}</strong></template>
        </i18n-t>
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="confirmDelete = false"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-surface-3 hover:text-neutral-800"
          type="button"
          @click="deleteKey"
        >{{ t('common.delete') }}</button>
      </div>
    </div>
  </div>
</template>
