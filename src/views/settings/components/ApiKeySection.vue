<script setup lang="ts">
/**
 * Ligne « Clé API Anthropic » des Paramètres. En mode PROD, la clé est
 * enregistrée dans le trousseau du système : pastille d'état, « Remplacer »
 * qui révèle la saisie, suppression par une icône discrète. En mode DEV, elle
 * est lue depuis le `.env` : l'état est affiché en lecture seule. La clé
 * complète ne revient jamais du backend.
 */

// Importation des modules Vue / framework
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import StatusPill from '@/components/StatusPill.vue'
import SettingsRow from '@/views/settings/components/SettingsRow.vue'
import TrashIcon from '@/views/settings/components/TrashIcon.vue'

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
const replacing = ref(false)
const confirmDelete = ref(false)
const feedback = ref<string | null>(null)
const inputEl = ref<HTMLInputElement | null>(null)

const editable = computed(() => status.value?.source === 'keyring')
/** Saisie visible : aucune clé enregistrée, ou remplacement demandé. */
const showInput = computed(() => editable.value && (!status.value?.present || replacing.value))

const help = computed(() => {
  if (!status.value) return ''
  return editable.value ? t('settings.api_key.source_keyring') : t('settings.api_key.source_env')
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

/** Révèle le champ de saisie et y place le curseur. */
async function startReplace() {
  replacing.value = true
  await nextTick()
  inputEl.value?.focus()
}

function cancelReplace() {
  replacing.value = false
  keyInput.value = ''
  feedback.value = null
}

/** Enregistre la clé saisie dans le trousseau, puis demande un test de connexion. */
async function saveKey() {
  if (!keyInput.value.trim()) return
  saving.value = true
  feedback.value = null
  try {
    status.value = await invoke<ApiKeyStatus>('save_api_key', { key: keyInput.value })
    keyInput.value = ''
    replacing.value = false
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
    replacing.value = false
  } catch (e: unknown) {
    feedback.value = backendErrorMessage(e, t('error.fallback.delete_api_key'))
  }
}
</script>

<template>
  <SettingsRow :label="t('settings.api_key.label')" :help="help">
    <StatusPill v-if="!status">{{ feedback ? t('settings.api_key.status_unavailable') : t('common.loading') }}</StatusPill>
    <StatusPill v-else-if="!status.present" tone="warn">{{ t('settings.api_key.none') }}</StatusPill>
    <StatusPill v-else tone="ok">
      <i18n-t keypath="settings.api_key.configured" tag="span" scope="global">
        <template #key><span class="font-mono">…{{ status.last4 ?? '' }}</span></template>
      </i18n-t>
    </StatusPill>

    <template v-if="editable && status?.present && !replacing">
      <button
        type="button"
        class="h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        @click="startReplace"
      >{{ t('settings.api_key.replace') }}</button>
      <button
        type="button"
        class="w-8 h-8 rounded-lg border border-transparent bg-transparent text-neutral-400 flex items-center justify-center cursor-pointer flex-none hover:text-red-600 hover:bg-red-50 hover:border-red-200"
        :title="t('common.delete')"
        :aria-label="t('common.delete')"
        @click="confirmDelete = true"
      ><TrashIcon /></button>
    </template>

    <template #below>
      <form v-if="showInput" class="flex flex-wrap gap-2 mt-3" @submit.prevent="saveKey">
        <input
          ref="inputEl"
          v-model="keyInput"
          type="password"
          autocomplete="off"
          spellcheck="false"
          placeholder="sk-ant-…"
          class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 min-w-[200px] font-mono"
        >
        <button
          class="h-[38px] px-4 rounded-lg border-0 bg-neutral-950 text-white text-[13px] font-medium inline-flex items-center cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-800 disabled:opacity-50 disabled:cursor-not-allowed"
          type="submit"
          :disabled="saving || !keyInput.trim()"
        >{{ saving ? t('common.saving') : t('common.save') }}</button>
        <button
          v-if="status?.present"
          class="h-[38px] px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="cancelReplace"
        >{{ t('common.cancel') }}</button>
      </form>
      <div v-if="feedback" class="text-[12px] font-mono mt-2 text-red-500">{{ feedback }}</div>
    </template>
  </SettingsRow>

  <!-- Confirmation de suppression -->
  <div
    v-if="confirmDelete"
    class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100] p-4"
    @click="confirmDelete = false"
  >
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[440px] max-w-full max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
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
