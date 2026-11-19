<script setup lang="ts">
/**
 * Paramètres de la session courante (onglet Paramètres de la page
 * Candidatures). Section « Session » : nom, ville + code postal (édités d'un
 * simple clic) et statut (sélecteur segmenté), reliés au backend
 * (`sessions`). Section « Modèles de documents » : le CV et la lettre de
 * référence propres à cette session (`documents/sessions/<id>/`), une
 * `DocumentReferenceCard` par document, puis une aide au balisage repliable.
 */

// Importation des modules Vue / framework
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { currentSession, currentSessionLocked, updateSession, updateSessionLocation } from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'
import type { DocumentReferenceView, SessionStatut } from '@/types'
import DocumentReferenceCard from '@/components/DocumentReferenceCard.vue'
import EditableField from '@/components/EditableField.vue'
import ReferenceMarkupHelp from '@/components/ReferenceMarkupHelp.vue'
import SettingsRow from '@/components/SettingsRow.vue'
import SettingsSection from '@/components/SettingsSection.vue'

const { t } = useI18n()

/** Infobulle des contrôles bloqués d'une session en lecture seule (seul le statut reste modifiable). */
const lockedHint = computed(() => (currentSessionLocked.value ? t('session.locked.hint') : null))

/** Statuts de session proposés : valeurs stockées inchangées, libellés traduits. */
const statusOptions = computed<{ value: SessionStatut; label: string }[]>(() =>
  (['active', 'pause', 'termine'] as const).map((value) => ({ value, label: t(`session.status.${value}`) })),
)

/** Sauvegarde le titre de la session. */
async function saveNom(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { nom: v })
}

const statusError = ref<string | null>(null)

/** Sauvegarde le statut de la session choisi dans le sélecteur segmenté. */
async function saveStatut(v: SessionStatut) {
  const session = currentSession.value
  if (!session || session.statut === v) return
  statusError.value = (await updateSession(session.id, { statut: v })) ?? null
}

/** La ville est requise pour créer une candidature : l'aide le rappelle. */
const missingCity = computed(() => !(currentSession.value?.ville ?? '').trim())

/** Sauvegarde la ville de l'expéditeur (le code postal est conservé). */
async function saveVille(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSessionLocation(session.id, { ville: v.trim() || null, zip_code: session.zip_code })
}

/** Sauvegarde le code postal de l'expéditeur (la ville est conservée). */
async function saveZipCode(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSessionLocation(session.id, { ville: session.ville, zip_code: v.trim() || null })
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
    documentsLoadError.value = backendErrorMessage(e, t('error.fallback.load_session_documents'))
  }
}

watch(sessionId, loadDocumentReferences, { immediate: true })

</script>

<template>
  <div class="px-5 sm:px-[30px] pt-6 pb-12">
    <SettingsSection :title="t('session.params.section')">
      <SettingsRow :label="t('session.params.name')" :help="t('session.params.name_help')">
        <div class="w-[340px] max-w-full">
          <EditableField
            :model-value="currentSession?.nom ?? null"
            :editable="!currentSessionLocked"
            :locked-hint="t('session.locked.hint')"
            single-click
            :placeholder="t('session.params.name')"
            :on-save="saveNom"
          />
        </div>
      </SettingsRow>

      <SettingsRow :label="t('session.params.city')">
        <div class="w-[220px] max-w-full">
          <EditableField
            :model-value="currentSession?.ville ?? null"
            :editable="!currentSessionLocked"
            :locked-hint="t('session.locked.hint')"
            single-click
            :placeholder="t('session.params.city_placeholder')"
            :on-save="saveVille"
          />
        </div>
        <div class="w-[110px]">
          <EditableField
            :model-value="currentSession?.zip_code ?? null"
            :editable="!currentSessionLocked"
            :locked-hint="t('session.locked.hint')"
            single-click
            :placeholder="t('session.params.zip_code')"
            :on-save="saveZipCode"
          />
        </div>
        <template #help>
          <span v-if="missingCity" class="text-red-600">{{ t('session.params.city_required') }}</span>
          <template v-else>{{ t('session.params.city_help') }}</template>
        </template>
      </SettingsRow>

      <SettingsRow :label="t('session.params.status')" :help="t('session.params.status_help')">
        <div class="inline-flex p-[3px] rounded-9 bg-neutral-100 gap-0.5">
          <button
            v-for="opt in statusOptions"
            :key="opt.value"
            type="button"
            class="border-0 px-[14px] py-[5px] rounded-7 text-[12.5px] font-sans cursor-pointer"
            :class="currentSession?.statut === opt.value
              ? 'bg-white text-neutral-950 font-medium shadow-[0_1px_2px_rgba(0,0,0,.08)]'
              : 'bg-transparent text-neutral-600'"
            @click="saveStatut(opt.value)"
          >{{ opt.label }}</button>
        </div>
        <template #below>
          <div v-if="statusError" class="text-[12px] font-mono mt-2 text-red-500">{{ statusError }}</div>
        </template>
      </SettingsRow>
    </SettingsSection>

    <section class="mb-[30px]">
      <div class="mb-[10px]">
        <h2 class="text-[15px] font-semibold text-neutral-950 m-0">{{ t('session.params.documents_title') }}</h2>
        <p class="text-[12.5px] text-neutral-500 mt-[3px] mb-0">{{ t('session.params.documents_description') }}</p>
      </div>
      <div v-if="documentsLoadError" class="font-mono text-[12px] text-red-500 mb-[10px]">{{ documentsLoadError }}</div>
      <div class="grid grid-cols-1 min-[900px]:grid-cols-2 gap-4">
        <DocumentReferenceCard
          doc-type="cv"
          :session-id="sessionId"
          :reference="cvReference"
          :locked-hint="lockedHint"
          @update:reference="v => (cvReference = v)"
        />
        <DocumentReferenceCard
          doc-type="lettre_motivation"
          :session-id="sessionId"
          :reference="lettreReference"
          :locked-hint="lockedHint"
          @update:reference="v => (lettreReference = v)"
        />
      </div>
      <div class="mt-3 px-5 py-3 border border-neutral-200 rounded-xl bg-white">
        <ReferenceMarkupHelp :summary="t('session.params.markup_summary')" />
      </div>
    </section>
  </div>
</template>
