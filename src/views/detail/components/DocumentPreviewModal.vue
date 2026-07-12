<script setup lang="ts">
/**
 * Modale d'aperçu PDF d'un document joint à une candidature (CV, lettre de
 * motivation), propre à `DetailView`. Même mécanique que l'aperçu des
 * documents de référence du profil (`DocumentReferenceEditor.vue`) : lecture
 * du PDF encodé en base64 côté backend, affiché dans une iframe.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'

const visible = ref(false)
const loading = ref(false)
const error = ref<string | null>(null)
const data = ref<string | null>(null)
const title = ref('')

/**
 * Ouvre la modale et charge l'aperçu PDF d'un document joint à la candidature.
 *
 * @param path - Chemin absolu du document PDF.
 * @param label - Titre affiché dans la modale (ex. "Aperçu du CV").
 */
async function open(path: string, label: string) {
  visible.value = true
  loading.value = true
  error.value = null
  data.value = null
  title.value = label
  try {
    data.value = await invoke<string>('get_document_pdf', { path })
  } catch (e: unknown) {
    error.value = extractErrorMessage(e, "Impossible de charger l'aperçu du document.")
  } finally {
    loading.value = false
  }
}

function close() {
  visible.value = false
}

defineExpose({ open })
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 bg-black/45 z-[1000] flex items-center justify-center" @click.self="close">
      <div class="bg-white rounded-xl shadow-[0_8px_40px_rgba(0,0,0,0.22)] w-[min(860px,calc(100vw-48px))] h-[min(92vh,1000px)] flex flex-col overflow-hidden">
        <div class="flex items-center justify-between px-5 py-[14px] border-b border-neutral-200 flex-none">
          <span class="text-[14px] font-semibold text-neutral-950">{{ title }}</span>
          <button
            class="w-7 h-7 rounded-7 border border-neutral-200 bg-neutral-50 flex items-center justify-center cursor-pointer text-neutral-600 hover:bg-neutral-100 hover:text-neutral-950"
            type="button"
            @click="close"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="flex-1 overflow-hidden flex items-center justify-center">
          <div v-if="loading" class="flex flex-col items-center gap-[14px] text-neutral-500 text-[13.5px]">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin text-blue-500">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
            Chargement…
          </div>
          <div v-else-if="error" class="flex items-start gap-[10px] text-red-600 text-[13px] max-w-[480px] p-5 bg-red-50 border border-red-200 rounded-lg leading-[1.5]">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            {{ error }}
          </div>
          <iframe
            v-else-if="data"
            :src="`data:application/pdf;base64,${data}`"
            class="w-full h-full border-none block"
            title="Aperçu PDF"
          />
        </div>
      </div>
    </div>
  </Teleport>
</template>
