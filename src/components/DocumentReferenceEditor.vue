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
  <div v-if="!isAttachment" class="fieldset">
    <div class="fstitle">{{ fieldsetTitle }}</div>

    <!-- Affichage du document actuel -->
    <div v-if="reference" class="cv-current">
      <div class="cv-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
      </div>
      <div class="cv-info">
        <div class="cv-label">reference.typ</div>
        <div class="cv-meta">{{ savedLabel }} {{ fmtReferenceDate(reference.updated_at) }}</div>
      </div>
      <button class="cv-preview-btn" type="button" :title="previewTitle" @click="preview">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
    </div>
    <div v-else class="cempty">{{ emptyLabel }}</div>

    <!-- Bouton en mode édition -->
    <div v-if="editMode" class="cv-actions">
      <input
        ref="fileInput"
        type="file"
        accept=".typ"
        style="display:none"
        @change="onFileSelected"
      />
      <button
        class="btn cv-upload-btn"
        type="button"
        :disabled="saving"
        @click="triggerPicker"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right:6px">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        {{ saving ? 'Sauvegarde…' : uploadLabel }}
      </button>
      <span v-if="error" class="cv-error">{{ error }}</span>
    </div>
  </div>

  <!-- Mode "attachment" (nouvelle candidature) : document ponctuellement joint -->
  <div v-else>
    <div class="doczonetitle">{{ attachmentTitle }}</div>

    <div v-if="modelValue" class="doc">
      <div class="docicon docicon-file">
        <svg width="14" height="17" viewBox="0 0 14 17" fill="none">
          <path d="M1.5 1.5h7l4 4v9.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1v-12.5a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
          <path d="M8.5 1.5v4h4" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
      </div>
      <div class="f1">
        <div class="crole">{{ basename(modelValue) }}</div>
        <div class="cmeta">{{ attachedLabel }}</div>
      </div>
      <button class="eyebtn" type="button" title="Retirer" @click="clearAttachment">✕</button>
    </div>
    <template v-else>
      <div
        class="doczone"
        :style="dragOver ? 'border-color:#539bf5' : ''"
        @dragover.prevent="dragOver = true"
        @dragleave.prevent="dragOver = false"
        @drop.prevent="onDrop"
      >
        <div class="droptext">→ Glisser le PDF ici</div>
        <button class="btn" type="button" :disabled="saving" @click="browse">
          {{ saving ? 'Chargement…' : 'Parcourir' }}
        </button>
      </div>
      <button class="btn" type="button" style="width:100%;justify-content:center;margin-top:8px">
        ✦ {{ generateLabel }}
      </button>
    </template>
    <div v-if="error" style="color:#b94a48;font-size:12px;margin-top:4px">{{ error }}</div>
  </div>

  <!-- Modale prévisualisation (mode "reference" uniquement) -->
  <Teleport to="body">
    <div v-if="previewVisible" class="cv-modal-overlay" @click.self="previewVisible = false">
      <div class="cv-modal">
        <div class="cv-modal-header">
          <span class="cv-modal-title">{{ modalTitle }}</span>
          <button class="cv-modal-close" type="button" @click="previewVisible = false">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="cv-modal-body">
          <div v-if="previewLoading" class="cv-modal-loading">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="cv-spinner">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
            Compilation en cours…
          </div>
          <div v-else-if="previewError" class="cv-modal-error">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink:0">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            {{ previewError }}
          </div>
          <iframe
            v-else-if="previewData"
            :src="`data:application/pdf;base64,${previewData}`"
            class="cv-modal-iframe"
            :title="iframeTitle"
          />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.cv-current {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 0 6px;
}

.cv-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: #f0f4ff;
  border: 1px solid #d0d8f0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #4a70d9;
  flex: none;
}

.cv-info { flex: 1; min-width: 0; }

.cv-label {
  font-size: 13px;
  font-weight: 600;
  color: #1d1d1b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cv-meta {
  font-size: 11px;
  color: #888;
  margin-top: 1px;
}

.cv-actions {
  margin-top: 10px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.cv-upload-btn {
  display: flex;
  align-items: center;
  font-size: 13px;
  padding: 7px 14px;
  border-radius: 8px;
  background: #f5f5f2;
  border: 1px solid #d4d4ce;
  color: #1d1d1b;
  cursor: pointer;
  transition: background .15s;
}
.cv-upload-btn:hover:not(:disabled) { background: #eaeae6; border-color: #b4b4aa; }
.cv-upload-btn:disabled { opacity: 0.6; cursor: default; }

.cv-error {
  font-size: 12px;
  color: #c53a3a;
}

.cv-preview-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 30px;
  height: 30px;
  border-radius: 7px;
  border: 1px solid #d0d8f0;
  background: #f0f4ff;
  color: #4a70d9;
  cursor: pointer;
  transition: background .15s, border-color .15s;
}
.cv-preview-btn:hover { background: #e4ecff; border-color: #a0b4e8; }

/* ── Modale aperçu ── */
.cv-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cv-modal {
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.22);
  width: min(860px, calc(100vw - 48px));
  height: min(92vh, 1000px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.cv-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid #e6e6e1;
  flex: none;
}

.cv-modal-title {
  font-size: 14px;
  font-weight: 600;
  color: #1d1d1b;
}

.cv-modal-close {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  border: 1px solid #e2e2dc;
  background: #fafaf8;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #76766f;
}
.cv-modal-close:hover { background: #f1f1ec; color: #1d1d1b; }

.cv-modal-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cv-modal-iframe {
  width: 100%;
  height: 100%;
  border: none;
  display: block;
}

.cv-modal-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  color: #666;
  font-size: 13.5px;
}

.cv-spinner {
  animation: spin 1s linear infinite;
  color: #4a70d9;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.cv-modal-error {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  color: #c53a3a;
  font-size: 13px;
  max-width: 480px;
  padding: 20px;
  background: #fdf6f6;
  border: 1px solid #f0d0d0;
  border-radius: 8px;
  line-height: 1.5;
}
</style>
