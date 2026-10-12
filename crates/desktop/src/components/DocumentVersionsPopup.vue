<script setup lang="ts">
/**
 * Popup listant les versions d'un document de référence (CV ou lettre de
 * motivation) d'une session, ouverte depuis `DocumentReferenceRow`. Permet de
 * visualiser une version et de la restaurer comme référence courante (celle
 * utilisée pour les prochaines générations). La liste est chargée par le
 * parent et rechargée par lui après chaque restauration.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import { formatDateTime } from '@/composables/date_time_format'
import type { DocumentReferenceView, DocumentVersionView } from '@/types/index'

interface Props {
  docType: 'cv' | 'lettre_motivation'
  /** Identifiant de la session propriétaire des documents. */
  sessionId: number | null
  /** Versions du document, chargées par le parent. */
  versions: DocumentVersionView[]
  /** Erreur de chargement des versions, le cas échéant. */
  loadError: string | null
  /** Autorise la restauration (mode édition des paramètres de session). */
  editMode: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  /** Demande de fermeture (croix, Échap, clic sur le fond). */
  close: []
  /** Demande l'aperçu PDF d'une version (modale portée par le parent). */
  preview: [version: number]
  /** Une version a été restaurée : nouvelle référence courante. */
  restored: [value: DocumentReferenceView]
}>()

const title = computed(() => (props.docType === 'cv' ? 'Versions – CV' : 'Versions – Lettre de motivation'))

// Copie triée : on ne dépend pas de l'ordre renvoyé par le backend.
const sortedVersions = computed(() => [...props.versions].sort((a, b) => b.version - a.version))

const restoringVersion = ref<number | null>(null)
const restoreError = ref<string | null>(null)

/**
 * Restaure une version comme référence courante.
 *
 * @param v - Version à restaurer.
 */
async function restore(v: DocumentVersionView) {
  if (v.is_current || props.sessionId == null) return
  restoringVersion.value = v.version
  restoreError.value = null
  try {
    const restored = await invoke<DocumentReferenceView>('restore_document_version', {
      sessionId: props.sessionId,
      docType: props.docType,
      version: v.version,
    })
    emit('restored', restored)
  } catch (e: unknown) {
    restoreError.value = extractErrorMessage(e, 'Impossible de restaurer cette version.')
  } finally {
    restoringVersion.value = null
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="emit('close')">
    <div
      class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[560px] max-w-[calc(100vw-48px)] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
      @click.stop
    >
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">{{ title }}</div>
        <button
          class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950"
          type="button"
          aria-label="Fermer"
          @click="emit('close')"
        >×</button>
      </div>

      <div v-if="loadError" class="text-[12px] text-red-600">{{ loadError }}</div>
      <div v-else-if="sortedVersions.length === 0" class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucune version enregistrée</div>
      <div v-else>
        <div
          v-for="(v, i) in sortedVersions"
          :key="v.id"
          class="flex items-center gap-3 py-[10px]"
          :class="{ 'border-t border-neutral-200': i > 0 }"
        >
          <span class="text-[13px] font-semibold font-mono text-neutral-950 flex-none w-8">v{{ v.version }}</span>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-[13px] text-neutral-950 whitespace-nowrap overflow-hidden text-ellipsis">{{ v.original_filename }}</span>
              <span
                v-if="v.is_current"
                class="inline-block flex-none font-medium text-[11px] rounded-5 px-[7px] py-px border bg-blue-50 border-blue-200 text-blue-600"
              >actuelle</span>
            </div>
            <div class="text-[11px] text-neutral-500 mt-px font-mono">{{ formatDateTime(v.created_at) }}</div>
          </div>
          <div class="flex items-center gap-2 flex-none">
            <button
              class="inline-flex items-center justify-center flex-none w-[30px] h-[30px] rounded-7 border border-blue-200 bg-blue-50 text-blue-500 cursor-pointer transition-colors duration-150 hover:bg-blue-100 hover:border-blue-300"
              type="button"
              :title="`Visualiser la version ${v.version}`"
              :aria-label="`Visualiser la version ${v.version}`"
              @click="emit('preview', v.version)"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            </button>
            <button
              v-if="editMode"
              class="h-[30px] px-3 rounded-7 border border-neutral-300 bg-surface-2 text-[12px] font-medium text-neutral-900 cursor-pointer whitespace-nowrap hover:bg-surface-3 disabled:opacity-60 disabled:cursor-default"
              type="button"
              title="Utiliser cette version pour les prochaines générations"
              :disabled="v.is_current || restoringVersion !== null"
              @click="restore(v)"
            >{{ restoringVersion === v.version ? 'Restauration…' : 'Restaurer' }}</button>
          </div>
        </div>
      </div>
      <div v-if="restoreError" class="mt-[6px] text-[12px] text-red-600">{{ restoreError }}</div>
    </div>
  </div>
</template>
