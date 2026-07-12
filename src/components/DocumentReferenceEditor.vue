<script setup lang="ts">
/**
 * Éditeur de document (CV ou lettre de motivation), paramétré par `docType`
 * ET par `mode` :
 * - `mode="reference"` (profil) : document Typst persistant, sauvegardé
 *   immédiatement côté backend, avec compilation et aperçu PDF.
 * - `mode="attachment"` (nouvelle candidature) : PDF ponctuellement joint à
 *   une candidature (glisser-déposer ou sélecteur natif), sans aperçu.
 *
 * Les deux modes partagent la même mécanique de paramétrage par `docType`
 * (labels et noms de commande dérivés de `cv` | `lettre_motivation`), ce qui
 * évite de dupliquer deux fois la même logique par document ET par contexte.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { DocumentReferenceView } from '@/types/index'

interface Props {
  docType: 'cv' | 'lettre_motivation'
  /** Mode d'édition : document de référence persistant, ou pièce jointe ponctuelle. */
  mode?: 'reference' | 'attachment'
  /** Mode "reference" uniquement : contrôle l'affichage de l'upload. */
  editMode?: boolean
  /** Mode "reference" uniquement : référence actuellement sauvegardée. */
  reference?: DocumentReferenceView | null
  /** Mode "attachment" uniquement : chemin du fichier mis en scène (v-model). */
  modelValue?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'reference',
  editMode: true,
  reference: null,
  modelValue: null,
})

const emit = defineEmits<{
  'update:reference': [value: DocumentReferenceView]
  'update:modelValue': [value: string | null]
}>()

const isCv = computed(() => props.docType === 'cv')
const isAttachment = computed(() => props.mode === 'attachment')

const saving = ref(false)
const error = ref<string | null>(null)

// ── Mode "reference" (profil) ────────────────────────────────────────────────

const saveCommand = computed(() => (isCv.value ? 'save_cv' : 'save_lettre_motivation'))
const compileCommand = computed(() => (isCv.value ? 'compile_cv' : 'compile_lettre_motivation'))

const fieldsetTitle = computed(() => (isCv.value ? 'CV (TYPST)' : 'LETTRE DE MOTIVATION (TYPST)'))
const emptyLabel = computed(() => (isCv.value ? 'Aucun CV renseigné' : 'Aucune lettre de motivation renseignée'))
const savedLabel = computed(() => (isCv.value ? 'Enregistré le' : 'Enregistrée le'))
const previewTitle = computed(() => (isCv.value ? 'Visualiser le CV compilé' : 'Visualiser la lettre compilée'))
const uploadLabel = computed(() => {
  if (!isCv.value) return props.reference ? 'Remplacer la lettre' : 'Ajouter une lettre'
  return props.reference ? 'Remplacer le CV' : 'Ajouter un CV'
})
const modalTitle = computed(() => (isCv.value ? 'Aperçu du CV' : 'Aperçu de la lettre de motivation'))
const iframeTitle = computed(() => (isCv.value ? 'Aperçu PDF du CV' : 'Aperçu PDF de la lettre de motivation'))
const saveErrorFallback = computed(() =>
  isCv.value ? 'Erreur lors de la sauvegarde du CV.' : 'Erreur lors de la sauvegarde de la lettre de motivation.',
)
const compileErrorFallback = computed(() =>
  isCv.value ? 'Erreur lors de la compilation du CV.' : 'Erreur lors de la compilation de la lettre de motivation.',
)

const fileInput = ref<HTMLInputElement | null>(null)
const justSaved = ref(false)

const previewVisible = ref(false)
const previewLoading = ref(false)
const previewError = ref<string | null>(null)
const previewData = ref<string | null>(null)

/**
 * Formate le timestamp epoch (ms) de dernière sauvegarde d'une référence.
 *
 * @param updatedAt - Timestamp en millisecondes, sous forme de chaîne.
 * @returns Date formatée, ou `'?'` si le timestamp est absent/invalide.
 */
function fmtReferenceDate(updatedAt: string): string {
  const ms = Number(updatedAt)
  if (!ms) return '?'
  return new Date(ms).toLocaleDateString('fr-FR')
}

function triggerPicker() {
  error.value = null
  fileInput.value?.click()
}

async function onFileSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  if (!file.name.toLowerCase().endsWith('.typ')) {
    error.value = 'Seuls les fichiers .typ sont acceptés.'
    input.value = ''
    return
  }

  saving.value = true
  error.value = null

  try {
    const content = await file.text()
    const saved = await invoke<DocumentReferenceView>(saveCommand.value, { filename: file.name, content })
    emit('update:reference', saved)
    justSaved.value = true
  } catch (e: unknown) {
    error.value = extractErrorMessage(e, saveErrorFallback.value)
  } finally {
    saving.value = false
    input.value = ''
  }
}

async function preview() {
  if (!props.reference) return
  previewVisible.value = true
  previewLoading.value = true
  previewError.value = null
  previewData.value = null
  try {
    previewData.value = await invoke<string>(compileCommand.value)
  } catch (e: unknown) {
    previewError.value = extractErrorMessage(e, compileErrorFallback.value)
  } finally {
    previewLoading.value = false
  }
}

/** `true` si un nouveau document a été sauvegardé depuis l'entrée en mode édition. */
const hasChanges = computed(() => justSaved.value)

/** Efface l'indicateur de sauvegarde récente (utilisé quand l'utilisateur annule). */
function reset() {
  justSaved.value = false
  error.value = null
}

defineExpose({ hasChanges, reset })

// ── Mode "attachment" (nouvelle candidature) ─────────────────────────────────

const uploadCommand = computed(() => (isCv.value ? 'upload_candidature_cv' : 'upload_candidature_lettre_motivation'))
const selectCommand = computed(() => (isCv.value ? 'select_candidature_cv' : 'select_candidature_lettre_motivation'))
const referenceDirCommand = computed(() => (isCv.value ? 'get_cv_reference_dir' : 'get_lettre_motivation_reference_dir'))
const browseDialogTitle = computed(() => (isCv.value ? 'Choisir un CV' : 'Choisir une lettre de motivation'))
const attachmentTitle = computed(() => (isCv.value ? 'Curriculum Vitae' : 'Lettre de motivation'))
const attachedLabel = computed(() => (isCv.value ? 'Joint' : 'Jointe'))
const generateLabel = computed(() => (isCv.value ? 'Générer mon CV' : 'Générer ma lettre'))
const uploadErrorFallback = computed(() =>
  isCv.value ? 'Erreur lors du téléversement du CV.' : 'Erreur lors du téléversement de la lettre.',
)
const selectErrorFallback = computed(() =>
  isCv.value ? 'Erreur lors de la sélection du CV.' : 'Erreur lors de la sélection de la lettre.',
)

const dragOver = ref(false)

/** Extrait le nom de fichier d'un chemin (Windows ou Unix). */
function basename(path: string): string {
  return path.split(/[\\/]/).pop() ?? path
}

/** Encode un fichier en base64, pour l'envoyer à une commande Tauri. */
async function fileToBase64(file: File): Promise<string> {
  const buffer = await file.arrayBuffer()
  const uint8 = new Uint8Array(buffer)
  let binary = ''
  for (let i = 0; i < uint8.length; i++) binary += String.fromCharCode(uint8[i])
  return btoa(binary)
}

/** Met en scène côté backend un PDF glissé-déposé ou choisi via l'input fichier. */
async function handleAttachmentFile(file: File) {
  if (!file.name.toLowerCase().endsWith('.pdf')) {
    error.value = 'Seuls les fichiers PDF sont acceptés.'
    return
  }
  saving.value = true
  error.value = null
  try {
    const content = await fileToBase64(file)
    const path = await invoke<string>(uploadCommand.value, { filename: file.name, content })
    emit('update:modelValue', path)
  } catch (e: unknown) {
    error.value = extractErrorMessage(e, uploadErrorFallback.value)
  } finally {
    saving.value = false
  }
}

function onDrop(event: DragEvent) {
  dragOver.value = false
  const file = event.dataTransfer?.files?.[0]
  if (file) handleAttachmentFile(file)
}

/** Ouvre le sélecteur de fichiers natif pour choisir un PDF existant. */
async function browse() {
  error.value = null
  try {
    const dir = await invoke<string>(referenceDirCommand.value)
    const path = await openDialog({
      title: browseDialogTitle.value,
      defaultPath: dir,
      multiple: false,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    })
    if (!path || Array.isArray(path)) return
    saving.value = true
    const staged = await invoke<string>(selectCommand.value, { path })
    emit('update:modelValue', staged)
  } catch (e: unknown) {
    error.value = extractErrorMessage(e, selectErrorFallback.value)
  } finally {
    saving.value = false
  }
}

function clearAttachment() {
  emit('update:modelValue', null)
  error.value = null
}
</script>

<template>
  <!-- Mode "reference" (profil) : document persistant, compilation + aperçu -->
  <div v-if="!isAttachment" class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">{{ fieldsetTitle }}</div>

    <!-- Affichage du document actuel -->
    <div v-if="reference" class="flex items-center gap-3 pt-[10px] pb-[6px]">
      <div class="w-9 h-9 rounded-lg bg-blue-50 border border-blue-200 flex items-center justify-center text-blue-500 flex-none">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[13px] font-semibold text-neutral-950 whitespace-nowrap overflow-hidden text-ellipsis">reference.typ</div>
        <div class="text-[11px] text-neutral-500 mt-px">{{ savedLabel }} {{ fmtReferenceDate(reference.updated_at) }}</div>
      </div>
      <button
        class="inline-flex items-center justify-center flex-none w-[30px] h-[30px] rounded-7 border border-blue-200 bg-blue-50 text-blue-500 cursor-pointer transition-colors duration-150 hover:bg-blue-100 hover:border-blue-300"
        type="button"
        :title="previewTitle"
        @click="preview"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
    </div>
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ emptyLabel }}</div>

    <!-- Bouton en mode édition -->
    <div v-if="editMode" class="mt-[10px] flex items-center gap-[10px] flex-wrap">
      <input
        ref="fileInput"
        type="file"
        accept=".typ"
        class="hidden"
        @change="onFileSelected"
      />
      <button
        class="h-9 px-[14px] rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] font-sans whitespace-nowrap bg-neutral-50 border border-neutral-300 text-neutral-950 cursor-pointer transition-colors duration-150 hover:bg-neutral-100 disabled:opacity-60 disabled:cursor-default"
        type="button"
        :disabled="saving"
        @click="triggerPicker"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-[6px]">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        {{ saving ? 'Sauvegarde…' : uploadLabel }}
      </button>
      <span v-if="error" class="text-[12px] text-red-600">{{ error }}</span>
    </div>
  </div>

  <!-- Mode "attachment" (nouvelle candidature) : document ponctuellement joint -->
  <div v-else>
    <div class="text-[13px] font-semibold mb-2 text-neutral-800">{{ attachmentTitle }}</div>

    <div v-if="modelValue" class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
      <div class="w-7 h-9 rounded-md flex-none flex items-center justify-center bg-neutral-100 border border-neutral-200 text-neutral-500">
        <svg width="14" height="17" viewBox="0 0 14 17" fill="none">
          <path d="M1.5 1.5h7l4 4v9.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1v-12.5a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
          <path d="M8.5 1.5v4h4" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[13.5px] font-semibold leading-[1.25]">{{ basename(modelValue) }}</div>
        <div class="font-mono text-[11px] text-neutral-500">{{ attachedLabel }}</div>
      </div>
      <button
        class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-neutral-100"
        type="button"
        title="Retirer"
        @click="clearAttachment"
      >✕</button>
    </div>
    <template v-else>
      <div
        class="border-2 border-dashed rounded-10 py-7 px-4 flex flex-col items-center gap-3 bg-neutral-50"
        :class="dragOver ? 'border-blue-400' : 'border-neutral-300'"
        @dragover.prevent="dragOver = true"
        @dragleave.prevent="dragOver = false"
        @drop.prevent="onDrop"
      >
        <div class="text-[13px] text-neutral-500 text-center font-mono">→ Glisser le PDF ici</div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
          type="button"
          :disabled="saving"
          @click="browse"
        >
          {{ saving ? 'Chargement…' : 'Parcourir' }}
        </button>
      </div>
      <button
        class="w-full justify-center mt-2 h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
        type="button"
      >
        ✦ {{ generateLabel }}
      </button>
    </template>
    <div v-if="error" class="text-red-500 text-[12px] mt-1">{{ error }}</div>
  </div>

  <!-- Modale prévisualisation (mode "reference" uniquement) -->
  <Teleport to="body">
    <div v-if="previewVisible" class="fixed inset-0 bg-black/45 z-[1000] flex items-center justify-center" @click.self="previewVisible = false">
      <div class="bg-white rounded-xl shadow-[0_8px_40px_rgba(0,0,0,0.22)] w-[min(860px,calc(100vw-48px))] h-[min(92vh,1000px)] flex flex-col overflow-hidden">
        <div class="flex items-center justify-between px-5 py-[14px] border-b border-neutral-200 flex-none">
          <span class="text-[14px] font-semibold text-neutral-950">{{ modalTitle }}</span>
          <button
            class="w-7 h-7 rounded-7 border border-neutral-200 bg-neutral-50 flex items-center justify-center cursor-pointer text-neutral-600 hover:bg-neutral-100 hover:text-neutral-950"
            type="button"
            @click="previewVisible = false"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="flex-1 overflow-hidden flex items-center justify-center">
          <div v-if="previewLoading" class="flex flex-col items-center gap-[14px] text-neutral-500 text-[13.5px]">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin text-blue-500">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
            Compilation en cours…
          </div>
          <div v-else-if="previewError" class="flex items-start gap-[10px] text-red-600 text-[13px] max-w-[480px] p-5 bg-red-50 border border-red-200 rounded-lg leading-[1.5]">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            {{ previewError }}
          </div>
          <iframe
            v-else-if="previewData"
            :src="`data:application/pdf;base64,${previewData}`"
            class="w-full h-full border-none block"
            :title="iframeTitle"
          />
        </div>
      </div>
    </div>
  </Teleport>
</template>
