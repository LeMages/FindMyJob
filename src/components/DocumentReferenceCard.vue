<script setup lang="ts">
/**
 * Carte d'un modèle de document (CV ou lettre de motivation) des paramètres
 * de session : version, pastille Prêt / Manquant, aperçu, remplacement et
 * historique des versions (`DocumentVersionsPopup`). Document Typst persistant,
 * sauvegardé immédiatement côté backend, avec compilation et aperçu PDF, et
 * accès à ses versions (`DocumentVersionsPopup`). Paramétré par `docType`
 * (labels et noms de commande dérivés de `cv` | `lettre_motivation`), ce qui
 * évite de dupliquer la même logique pour les deux documents.
 */

// Importation des modules Vue / framework
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import { formatDateTime } from '@/composables/date_time_format'
import DocumentVersionsPopup from '@/components/DocumentVersionsPopup.vue'
import StatusPill from '@/components/StatusPill.vue'
import { referenceIssueMessage, referenceIssuesOf } from '@/components/reference_issue_message'
import type { DocumentReferenceView, DocumentVersionView, ReferenceIssue } from '@/types/index'

const { t } = useI18n()

interface Props {
  docType: 'cv' | 'lettre_motivation'
  /** Identifiant de la session propriétaire des documents (CV/LM propres à chaque session). */
  sessionId?: number | null
  /** Contrôle l'affichage de l'upload. */
  editMode?: boolean
  /** Référence actuellement sauvegardée. */
  reference?: DocumentReferenceView | null
  /** Raison pour laquelle import, remplacement et restauration sont bloqués (session en lecture seule), sinon `null`. */
  lockedHint?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  sessionId: null,
  editMode: true,
  reference: null,
  lockedHint: null,
})

const emit = defineEmits<{
  'update:reference': [value: DocumentReferenceView]
}>()

const isCv = computed(() => props.docType === 'cv')

const saving = ref(false)
const error = ref<string | null>(null)
/** Problèmes bloquants du dernier import refusé (contrat non respecté). */
const issues = ref<ReferenceIssue[]>([])
/** Avertissements du dernier import réussi. */
const warnings = ref<ReferenceIssue[]>([])

// ── Mode "reference" (profil) ────────────────────────────────────────────────

const saveCommand = computed(() => (isCv.value ? 'save_cv' : 'save_lettre_motivation'))
const compileCommand = computed(() => (isCv.value ? 'compile_cv' : 'compile_lettre_motivation'))

const docLabel = computed(() => (isCv.value ? t('detail.documents.cv') : t('detail.documents.cover_letter')))
const emptyLabel = computed(() => (isCv.value ? t('documents.reference.none_cv') : t('documents.reference.none_lm')))
const modalTitle = computed(() => (isCv.value ? t('detail.documents.cv_preview_title') : t('detail.documents.lm_preview_title')))
const iframeTitle = computed(() => (isCv.value ? t('documents.reference.iframe_cv') : t('documents.reference.iframe_lm')))
const saveErrorFallback = computed(() =>
  isCv.value ? t('error.fallback.save_cv') : t('error.fallback.save_cover_letter'),
)
const compileErrorFallback = computed(() =>
  isCv.value ? t('error.fallback.compile_cv') : t('error.fallback.compile_cover_letter'),
)

const fileInput = ref<HTMLInputElement | null>(null)

/** Petit bouton secondaire des actions de la carte. */
const SM_BUTTON = 'h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed'
const justSaved = ref(false)

const previewVisible = ref(false)
const previewLoading = ref(false)
const previewError = ref<string | null>(null)
const previewData = ref<string | null>(null)
const previewVersionNumber = ref<number | null>(null)
const previewModalTitle = computed(() =>
  previewVersionNumber.value == null
    ? modalTitle.value
    : t('documents.reference.with_version', { title: modalTitle.value, version: previewVersionNumber.value }),
)

// ── Versions ─────────────────────────────────────────────────────────────────

const versions = ref<DocumentVersionView[]>([])
const versionsLoadError = ref<string | null>(null)
const versionsOpen = ref(false)

/** Version courante (la plus récente si aucune n'est marquée, cas d'une référence antérieure au versioning). */
const currentVersion = computed(
  () => versions.value.find(v => v.is_current) ?? versions.value.reduce<DocumentVersionView | null>(
    (latest, v) => (latest == null || v.version > latest.version ? v : latest),
    null,
  ),
)

// `DocumentReferenceView` ne porte pas le nom d'origine : on le lit sur la
// version courante, avec repli sur le nom du fichier stocké.
const displayFilename = computed(() => currentVersion.value?.original_filename ?? 'reference.typ')
const displayDate = computed(() =>
  formatDateTime(currentVersion.value?.created_at ?? props.reference?.updated_at ?? ''),
)

async function loadVersions() {
  versionsLoadError.value = null
  if (props.sessionId == null || !props.reference) {
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
    versionsLoadError.value = backendErrorMessage(e, t('error.fallback.load_versions'))
  }
}

// Chaque changement de référence (upload, restauration) recharge les versions.
watch(() => [props.sessionId, props.reference?.file_path, props.reference?.updated_at], loadVersions, {
  immediate: true,
})

/** Ferme la couche du dessus : l'aperçu s'il est ouvert par-dessus la popup, sinon la popup. */
function onVersionsClose() {
  if (previewVisible.value) {
    previewVisible.value = false
    return
  }
  versionsOpen.value = false
}

/** Efface le retour du dernier import (erreur, problèmes, avertissements). */
function clearImportFeedback() {
  error.value = null
  issues.value = []
  warnings.value = []
}

function triggerPicker() {
  clearImportFeedback()
  fileInput.value?.click()
}

async function onFileSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  if (!file.name.toLowerCase().endsWith('.typ')) {
    error.value = t('documents.reference.only_typ')
    input.value = ''
    return
  }

  saving.value = true
  clearImportFeedback()

  try {
    const content = await file.text()
    const saved = await invoke<DocumentReferenceView>(saveCommand.value, {
      sessionId: props.sessionId,
      filename: file.name,
      content,
    })
    emit('update:reference', saved)
    warnings.value = saved.warnings
    justSaved.value = true
  } catch (e: unknown) {
    const refused = referenceIssuesOf(e)
    if (refused) issues.value = refused
    else error.value = backendErrorMessage(e, saveErrorFallback.value)
  } finally {
    saving.value = false
    input.value = ''
  }
}

/**
 * Ouvre la modale d'aperçu et y charge un PDF base64.
 *
 * @param loadPdf - Charge le PDF encodé en base64.
 * @param errorFallback - Message affiché si l'erreur n'en porte pas.
 */
async function openPreview(loadPdf: () => Promise<string>, errorFallback: string) {
  previewVisible.value = true
  previewLoading.value = true
  previewError.value = null
  previewData.value = null
  try {
    previewData.value = await loadPdf()
  } catch (e: unknown) {
    previewError.value = backendErrorMessage(e, errorFallback)
  } finally {
    previewLoading.value = false
  }
}

async function preview() {
  if (!props.reference) return
  previewVersionNumber.value = null
  await openPreview(
    () => invoke<string>(compileCommand.value, { sessionId: props.sessionId }),
    compileErrorFallback.value,
  )
}

async function previewVersion(version: number) {
  previewVersionNumber.value = version
  await openPreview(
    () => invoke<string>('get_document_version_pdf', { sessionId: props.sessionId, docType: props.docType, version }),
    t('error.fallback.load_version_preview'),
  )
}

/** `true` si un nouveau document a été sauvegardé depuis l'entrée en mode édition. */
const hasChanges = computed(() => justSaved.value)

/** Efface l'indicateur de sauvegarde récente (utilisé quand l'utilisateur annule). */
function reset() {
  justSaved.value = false
  clearImportFeedback()
}

defineExpose({ hasChanges, reset })
</script>

<template>
  <div
    class="bg-white border rounded-xl px-5 py-[18px] flex flex-col"
    :class="reference ? 'border-neutral-200' : 'border-dashed border-neutral-300'"
  >
    <input
      ref="fileInput"
      type="file"
      accept=".typ"
      class="hidden"
      @change="onFileSelected"
    />

    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-9 bg-neutral-100 text-neutral-700 flex items-center justify-center flex-none">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
        </svg>
      </div>
      <div class="flex-1 min-w-0 flex items-center gap-2 text-[14px] font-semibold text-neutral-950">
        {{ docLabel }}
        <span v-if="reference && currentVersion" class="text-[10.5px] font-medium px-[7px] py-px rounded-full bg-neutral-100 text-neutral-700">v{{ currentVersion.version }}</span>
      </div>
      <StatusPill v-if="reference" tone="ok">{{ t('documents.reference.ready') }}</StatusPill>
      <StatusPill v-else tone="warn">{{ t('documents.reference.missing') }}</StatusPill>
    </div>

    <!-- Document actuel : nom du fichier et date de la version courante -->
    <template v-if="reference">
      <div class="font-mono text-[12.5px] text-neutral-800 mt-[14px] whitespace-nowrap overflow-hidden text-ellipsis">{{ displayFilename }}</div>
      <div class="text-[12px] text-neutral-500 mt-[3px]">{{ t('documents.reference.imported_on', { date: displayDate }) }}</div>
    </template>
    <div v-else class="text-[12.5px] text-neutral-500 mt-[14px]">{{ emptyLabel }}</div>

    <div v-if="saving" class="text-[12px] text-neutral-500 mt-[3px] flex items-center gap-[6px]">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin flex-none">
        <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
      </svg>
      <span>{{ t('documents.reference.importing') }}</span>
    </div>
    <div v-if="error" class="mt-[6px] text-[12px] text-red-600 whitespace-pre-line">{{ error }}</div>
    <div v-if="issues.length > 0" class="mt-[6px] text-[12px] text-red-600">
      <div class="font-medium">{{ t('documents.reference.issues_title') }}</div>
      <ul class="list-disc pl-4 mt-[3px] flex flex-col gap-[2px] break-words">
        <li v-for="(issue, index) in issues" :key="index">{{ referenceIssueMessage(issue) }}</li>
      </ul>
    </div>
    <div v-if="warnings.length > 0" class="mt-[6px] text-[12px] text-amber-700">
      <div class="font-medium">{{ t('documents.reference.warnings_title') }}</div>
      <ul class="list-disc pl-4 mt-[3px] flex flex-col gap-[2px] break-words">
        <li v-for="(issue, index) in warnings" :key="index">{{ referenceIssueMessage(issue) }}</li>
      </ul>
    </div>

    <div class="flex flex-wrap gap-2 mt-4 pt-[14px] border-t border-neutral-100">
      <template v-if="reference">
        <button type="button" :class="SM_BUTTON" @click="preview">{{ t('documents.reference.preview') }}</button>
        <button
          v-if="editMode"
          type="button"
          :class="SM_BUTTON"
          :title="lockedHint ?? undefined"
          :disabled="saving || lockedHint !== null"
          @click="triggerPicker"
        >{{ t('documents.reference.replace') }}</button>
        <button v-if="versions.length > 1" type="button" :class="SM_BUTTON" @click="versionsOpen = true">
          {{ t('documents.reference.history') }}
          <span class="font-mono text-[10.5px] text-neutral-500">{{ versions.length }}</span>
        </button>
      </template>
      <button
        v-else-if="editMode"
        type="button"
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800 disabled:opacity-40 disabled:cursor-not-allowed"
        :title="lockedHint ?? undefined"
        :disabled="saving || lockedHint !== null"
        @click="triggerPicker"
      >{{ t('documents.reference.import') }}</button>
    </div>
  </div>

  <DocumentVersionsPopup
    v-if="versionsOpen"
    :doc-type="docType"
    :session-id="sessionId"
    :versions="versions"
    :load-error="versionsLoadError"
    :edit-mode="editMode"
    :locked-hint="lockedHint"
    @close="onVersionsClose"
    @preview="previewVersion"
    @restored="emit('update:reference', $event)"
  />

  <!-- Modale prévisualisation -->
  <Teleport to="body">
    <div v-if="previewVisible" class="fixed inset-0 bg-black/45 z-[1000] flex items-center justify-center" @click.self="previewVisible = false">
      <div class="bg-surface-3 rounded-xl shadow-[0_8px_40px_rgba(0,0,0,0.22)] w-[min(860px,calc(100vw-48px))] h-[min(92vh,1000px)] flex flex-col overflow-hidden">
        <div class="flex items-center justify-between px-5 py-[14px] border-b border-neutral-200 flex-none">
          <span class="text-[14px] font-semibold text-neutral-950">{{ previewModalTitle }}</span>
          <button
            class="w-7 h-7 rounded-7 border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer text-neutral-600 hover:bg-surface-3 hover:text-neutral-950"
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
            {{ t('documents.reference.compiling') }}
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
