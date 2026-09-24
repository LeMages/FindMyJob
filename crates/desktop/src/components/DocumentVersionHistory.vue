<script setup lang="ts">
/**
 * Historique des versions d'un document de référence (CV ou lettre de
 * motivation) d'une session, affiché sous la référence courante dans
 * `DocumentReferenceEditor`. Permet de visualiser une ancienne version et de
 * la restaurer comme référence courante (celle utilisée pour les prochaines
 * générations). Rien n'est affiché tant qu'il n'existe qu'une seule version.
 */

// Importation des modules Vue / framework
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { DocumentReferenceView, DocumentVersionView } from '@/types/index'

interface Props {
  docType: 'cv' | 'lettre_motivation'
  /** Identifiant de la session propriétaire des documents. */
  sessionId: number | null
  /** Référence courante : chaque changement (upload, restauration) recharge l'historique. */
  reference: DocumentReferenceView | null
  /** Autorise la restauration (mode édition des paramètres de session). */
  editMode: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  /** Demande l'aperçu PDF d'une version (modale portée par le parent). */
  preview: [version: number]
  /** Une version a été restaurée : nouvelle référence courante. */
  restored: [value: DocumentReferenceView]
}>()

const versions = ref<DocumentVersionView[]>([])
const selectedVersion = ref<number | null>(null)
const loadError = ref<string | null>(null)
const restoring = ref(false)
const restoreError = ref<string | null>(null)

const selected = computed(() => versions.value.find(v => v.version === selectedVersion.value) ?? null)

/**
 * Formate la date d'upload d'une version.
 *
 * @param iso - Date ISO 8601.
 * @returns Date et heure au format français (ex. "24/09/2026 14:02").
 */
function fmtVersionDate(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '?'
  return `${d.toLocaleDateString('fr-FR')} ${d.toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })}`
}

/**
 * Libellé d'une version dans le sélecteur.
 *
 * @param v - Version à décrire.
 * @returns Ex. "v3 · 24/09/2026 14:02 · cv.typ (actuelle)".
 */
function versionLabel(v: DocumentVersionView): string {
  const current = v.is_current ? ' (actuelle)' : ''
  return `v${v.version} · ${fmtVersionDate(v.created_at)} · ${v.original_filename}${current}`
}

async function loadVersions() {
  loadError.value = null
  restoreError.value = null
  if (props.sessionId == null) {
    versions.value = []
    return
  }
  try {
    versions.value = await invoke<DocumentVersionView[]>('get_document_versions', {
      sessionId: props.sessionId,
      docType: props.docType,
    })
  } catch (e: unknown) {
    versions.value = []
    loadError.value = extractErrorMessage(e, "Impossible de charger l'historique des versions.")
  }
  // Par défaut on sélectionne la version courante (la plus récente si
  // aucune n'est marquée, cas d'une référence antérieure au versioning).
  const current = versions.value.find(v => v.is_current) ?? versions.value[0] ?? null
  selectedVersion.value = current?.version ?? null
}

async function restore() {
  if (!selected.value || selected.value.is_current || props.sessionId == null) return
  restoring.value = true
  restoreError.value = null
  try {
    const restored = await invoke<DocumentReferenceView>('restore_document_version', {
      sessionId: props.sessionId,
      docType: props.docType,
      version: selected.value.version,
    })
    emit('restored', restored)
  } catch (e: unknown) {
    restoreError.value = extractErrorMessage(e, 'Impossible de restaurer cette version.')
  } finally {
    restoring.value = false
  }
}

watch(() => [props.sessionId, props.reference?.file_path, props.reference?.updated_at], loadVersions, {
  immediate: true,
})
</script>

<template>
  <div v-if="loadError" class="mt-[10px] text-[12px] text-red-600">{{ loadError }}</div>
  <div v-else-if="versions.length > 1" class="mt-[10px] pt-[10px] border-t border-neutral-200">
    <label class="text-[12px] text-neutral-600 mb-[5px] block">Historique des versions</label>
    <div class="flex items-center gap-2">
      <select
        v-model="selectedVersion"
        class="flex-1 min-w-0 h-[30px] px-2 rounded-7 border border-neutral-300 bg-surface-2 text-[12px] text-neutral-900 font-mono"
      >
        <option v-for="v in versions" :key="v.id" :value="v.version">{{ versionLabel(v) }}</option>
      </select>
      <button
        class="inline-flex items-center justify-center flex-none w-[30px] h-[30px] rounded-7 border border-blue-200 bg-blue-50 text-blue-500 cursor-pointer transition-colors duration-150 hover:bg-blue-100 hover:border-blue-300 disabled:opacity-60 disabled:cursor-default"
        type="button"
        title="Visualiser cette version"
        :disabled="!selected"
        @click="selected && emit('preview', selected.version)"
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
        :disabled="!selected || selected.is_current || restoring"
        @click="restore"
      >{{ restoring ? 'Restauration…' : 'Restaurer' }}</button>
    </div>
    <div v-if="restoreError" class="mt-[6px] text-[12px] text-red-600">{{ restoreError }}</div>
  </div>
</template>
