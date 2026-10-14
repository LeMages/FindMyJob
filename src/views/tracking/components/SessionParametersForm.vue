<script setup lang="ts">
/**
 * Formulaire de paramètres de la session courante (onglet Paramètres de la
 * page Candidatures). Titre et statut sont reliés au backend (`sessions`),
 * édités au double-clic (même mécanique que la page détail d'une
 * candidature). Le CV et la lettre de motivation de référence
 * sont propres à cette session (`documents/sessions/<id>/`) et affichés dans
 * la card « Documents », une `DocumentReferenceRow` par document.
 */

// Importation des modules Vue / framework
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { currentSession, updateSession } from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { DocumentReferenceView, SessionStatut } from '@/types'
import EditableField from '@/components/EditableField.vue'
import DocumentReferenceRow from '@/components/DocumentReferenceRow.vue'

const STATUS_OPTIONS: { value: SessionStatut; label: string }[] = [
  { value: 'active', label: 'Active' },
  { value: 'pause', label: 'En pause' },
  { value: 'termine', label: 'Terminée' },
]

/** Sauvegarde le titre de la session. */
async function saveNom(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { nom: v })
}

/** Sauvegarde le statut de la session. */
async function saveStatut(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { statut: v as SessionStatut })
}

const sessionId = computed(() => currentSession.value?.id ?? null)

const cvReference = ref<DocumentReferenceView | null>(null)
const lettreReference = ref<DocumentReferenceView | null>(null)
const documentsLoadError = ref<string | null>(null)

/** Charge le CV et la lettre de motivation de référence de la session courante. */
async function loadDocumentReferences() {
  documentsLoadError.value = null
  if (sessionId.value === null) {
    cvReference.value = null
    lettreReference.value = null
    return
  }
  try {
    const [cv, lettre] = await Promise.all([
      invoke<DocumentReferenceView | null>('get_cv_reference', { sessionId: sessionId.value }),
      invoke<DocumentReferenceView | null>('get_lettre_motivation_reference', { sessionId: sessionId.value }),
    ])
    cvReference.value = cv
    lettreReference.value = lettre
  } catch (e: unknown) {
    documentsLoadError.value = extractErrorMessage(e, 'Impossible de charger les documents de la session.')
  }
}

watch(sessionId, loadDocumentReferences, { immediate: true })

</script>

<template>
  <div class="px-[30px] pt-[18px] pb-[30px]">
    <!-- Informations générales -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">INFORMATIONS GÉNÉRALES</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Titre de la session</label>
        <EditableField
          :model-value="currentSession?.nom ?? null"
          :editable="true"
          placeholder="Titre de la session"
          :on-save="saveNom"
        />
      </div>

      <div>
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Statut de la session</label>
        <EditableField
          type="select"
          :options="STATUS_OPTIONS"
          :model-value="currentSession?.statut ?? null"
          :editable="true"
          :on-save="saveStatut"
        />
      </div>
    </div>

    <!-- Documents -->
    <div v-if="documentsLoadError" class="font-mono text-[12px] text-red-500 mb-[18px]">{{ documentsLoadError }}</div>
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] pt-5 pb-2 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[4px] font-mono tracking-[.06em] text-neutral-500">DOCUMENTS</div>
      <DocumentReferenceRow
        doc-type="cv"
        :session-id="sessionId"
        :reference="cvReference"
        @update:reference="v => (cvReference = v)"
      />
      <div class="border-t border-neutral-200" />
      <DocumentReferenceRow
        doc-type="lettre_motivation"
        :session-id="sessionId"
        :reference="lettreReference"
        @update:reference="v => (lettreReference = v)"
      />
    </div>
  </div>
</template>
