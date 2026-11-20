<script setup lang="ts">
/**
 * Vue « Détail » d'une candidature : en-tête (statut, action selon le
 * statut), barre de progression en cinq étapes, bandeau d'état, puis le
 * suivi, le poste et les notes à gauche, les documents et les contacts dans
 * une colonne latérale fixe au défilement (une seule colonne sur un écran
 * étroit).
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import EditableField from '@/components/EditableField.vue'
import InfoBanner from '@/components/InfoBanner.vue'
import SectionCard from '@/components/SectionCard.vue'
import SessionLockedBanner from '@/components/SessionLockedBanner.vue'
import AddEventPopup from './components/AddEventPopup.vue'
import ApplicationProgressBar from './components/ApplicationProgressBar.vue'
import DetailContactsCard from './components/DetailContactsCard.vue'
import DetailHeader from './components/DetailHeader.vue'
import DetailPopup from './components/DetailPopup.vue'
import DocumentPreviewModal from './components/DocumentPreviewModal.vue'
import DocumentsCard from './components/DocumentsCard.vue'
import JobCard from './components/JobCard.vue'
import StatusTimelineCard from './components/StatusTimelineCard.vue'

// Importation des modules internes
import {
  state,
  allSessions,
  navigate,
  openDetailPopup,
  updateDetailCandidature,
} from '@/stores/app'
import { TRASHED_STATUSES, statusLabel } from '@/composables/candidature_display'
import type { DownloadDocType } from '@/composables/document_file_name'

const { t } = useI18n()

/** Session de la candidature ouverte (statut, verrou « lecture seule »). */
const detailSession = computed(() =>
  allSessions.value.find(s => s.id === state.detailCandidature?.session_id) ?? null,
)

/** Infobulle des contrôles bloqués quand la session est en pause ou terminée, sinon `null`. */
const lockedHint = computed(() =>
  detailSession.value && detailSession.value.statut !== 'active' ? t('session.locked.hint') : null,
)

// Les détails du poste et les notes ne sont modifiables que tant que la
// candidature n'a pas encore été engagée (statut "à postuler") et que sa
// session n'est pas en lecture seule.
const editable = computed(() => state.detailCandidature?.status === 'a_postuler' && lockedHint.value === null)

const closed = computed(() => TRASHED_STATUSES.includes(state.detailCandidature?.status ?? ''))

const previewModal = ref<InstanceType<typeof DocumentPreviewModal> | null>(null)
const addEventPopup = ref<InstanceType<typeof AddEventPopup> | null>(null)

/**
 * Ouvre l'aperçu PDF d'un document joint à la candidature.
 *
 * @param path - Chemin du document, relatif au dossier de données.
 * @param label - Titre affiché dans la modale d'aperçu.
 * @param docType - Type du document, pour nommer un éventuel téléchargement.
 */
function previewDocument(path: string, label: string, docType: DownloadDocType) {
  previewModal.value?.open(path, label, docType)
}
</script>

<template>
  <div class="px-5 sm:px-[30px] pt-[26px] pb-12">
    <button
      type="button"
      class="inline-block border-0 bg-transparent p-0 mb-[14px] text-[12.5px] font-sans text-neutral-500 cursor-pointer hover:text-neutral-950"
      @click="navigate('tracking')"
    >{{ t('detail.back_to_tracking') }}</button>

    <SessionLockedBanner v-if="detailSession" :statut="detailSession.statut" class="mb-4" />

    <!-- Loading -->
    <div v-if="state.detailLoading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-10">{{ t('common.loading') }}</div>

    <!-- Load error -->
    <div v-else-if="state.detailError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-10">{{ state.detailError }}</div>

    <template v-else-if="state.detailCandidature">
      <DetailHeader
        :candidature="state.detailCandidature"
        :locked-hint="lockedHint"
        @apply="openDetailPopup('apply')"
        @delete="openDetailPopup('confirmDelete')"
        @add-event="addEventPopup?.open()"
      />
      <ApplicationProgressBar :status="state.detailCandidature.status" :history="state.detailHistory" />

      <InfoBanner v-if="state.detailCandidature.status === 'accepte'" tone="ok">
        <strong>{{ t('detail.notice.accepted_title') }}</strong> {{ t('detail.notice.accepted') }}
      </InfoBanner>
      <InfoBanner v-else-if="closed" tone="muted">
        {{ t('detail.notice.closed', { status: statusLabel(state.detailCandidature.status).toLowerCase() }) }}
      </InfoBanner>

      <div class="grid grid-cols-1 min-[1100px]:grid-cols-[minmax(0,1fr)_360px] gap-5 items-start">
        <div class="min-w-0 flex flex-col gap-4">
          <StatusTimelineCard />
          <JobCard :candidature="state.detailCandidature" :editable="editable" :locked-hint="lockedHint" />
          <SectionCard
            :title="t('detail.notes.title')"
            :hint="editable ? t('detail.notes.editable_hint') : t('detail.notes.readonly_hint')"
          >
            <EditableField
              type="textarea"
              single-click
              :model-value="state.detailCandidature.notes"
              :editable="editable"
              :locked-hint="lockedHint ?? undefined"
              :placeholder="t('detail.notes.placeholder')"
              :on-save="v => updateDetailCandidature({ notes: v })"
            />
          </SectionCard>
        </div>

        <aside class="min-w-0 flex flex-col gap-4 min-[1100px]:sticky min-[1100px]:top-5">
          <DocumentsCard @preview="previewDocument" />
          <DetailContactsCard :locked-hint="lockedHint" />
        </aside>
      </div>
    </template>
  </div>

  <!-- Popups -->
  <DetailPopup />
  <AddEventPopup ref="addEventPopup" />
  <DocumentPreviewModal ref="previewModal" />
</template>
