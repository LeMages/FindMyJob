<script setup lang="ts">
/**
 * Vue « Nouvelle candidature » : import de l'offre (lien, capture de la
 * page, texte collé ou saisie manuelle), puis formulaire de création
 * (détails du poste, contenu de l'offre, liste de vérification « Prêt à
 * générer ? », contacts recruteur, notes) et popup de génération.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import InfoBanner from '@/components/InfoBanner.vue'
import SectionCard from '@/components/SectionCard.vue'
import SessionLockedBanner from '@/components/SessionLockedBanner.vue'
import ApplicationStepper from './components/ApplicationStepper.vue'
import BackToLinkButton from './components/BackToLinkButton.vue'
import GenerationProgressModal from './components/GenerationProgressModal.vue'
import GenerationReadyCard from './components/GenerationReadyCard.vue'
import JobFieldsCard from './components/JobFieldsCard.vue'
import OfferLinkStep from './components/OfferLinkStep.vue'
import ExtractionChoiceStep from './components/ExtractionChoiceStep.vue'
import CaptureStep from './components/CaptureStep.vue'
import PasteStep from './components/PasteStep.vue'
import DuplicateOfferWarning from './components/DuplicateOfferWarning.vue'
import ContactsSection from './components/ContactsSection.vue'

// Importation des modules internes
import type { CandidatureView, CreateCandidatureInput, OfferExtraction } from '@/types'
import {
  state,
  currentSession,
  currentSessionLocked,
  navigate,
  refreshSessions,
  addContactDraft,
  removeContactDraft,
  resetContactDrafts,
} from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'
import { draftFromScraped, draftToCandidatureContact } from '@/composables/contact_draft'
import { useDocumentsStatus } from '@/composables/useDocumentsStatus'
import { useGenerationProgress } from '@/composables/useGenerationProgress'
import { buildGenerationChecklist } from './generation_checklist'
import { useOfferImport } from './useOfferImport'
import { useDuplicateOffers } from './useDuplicateOffers'

const { t } = useI18n()

const roleRef = ref('')
const companyRef = ref('')
const locationRef = ref('')
const typeRef = ref('')
const sourceRef = ref('')
const remunerationRef = ref('')
const urlOffreRef = ref('')
const descriptionOffreRef = ref('')
const notesRef = ref('')
// Ligne de contact pré-remplie par la dernière extraction, remplacée par la
// suivante pour ne pas mélanger deux offres.
const extractedContactKey = ref<number | null>(null)
// Le formulaire vient-il d'une analyse (sous-titre de la carte « Le poste ») ?
const prefilled = ref(false)

const offerImport = useOfferImport(urlOffreRef, onOfferExtracted)
const { step, busy, urlError, urlUnavailable, choice, captureError, pasteError, pasteText } = offerImport
const { duplicates, checkError, check: checkDuplicates, clear: clearDuplicates } = useDuplicateOffers()

const isValid = computed(() =>
  roleRef.value.trim() !== '' &&
  companyRef.value.trim() !== '' &&
  locationRef.value.trim() !== '' &&
  typeRef.value !== '' &&
  sourceRef.value !== '' &&
  descriptionOffreRef.value.trim() !== ''
)

const saving = ref(false)
const saveError = ref<string | null>(null)

const showProgressModal = ref(false)
const { progress, pollError, start: startProgressPolling } = useGenerationProgress()

/**
 * Réagit à la fin de la génération des documents (voir `GenerationProgressModal`) :
 * recharge les sessions dans les deux cas (une génération échouée supprime
 * la candidature, le compteur change) ; en cas de succès, réinitialise le
 * formulaire et retourne au suivi comme avant le passage en asynchrone ; en
 * cas d'échec, la modale reste ouverte avec l'erreur et le formulaire est
 * conservé (l'utilisateur ne perd pas sa saisie).
 */
watch(
  () => progress.value?.status,
  (status) => {
    if (status === 'completed' || status === 'failed') void refreshSessions()
    if (status === 'completed') {
      showProgressModal.value = false
      resetForm()
      navigate('tracking')
    }
  }
)

/** Ferme la modale de progression (uniquement possible après un échec, voir la modale). */
function closeProgressModal() {
  showProgressModal.value = false
}

/** Réinitialise le formulaire et le parcours d'import après une création réussie. */
function resetForm() {
  roleRef.value = ''
  companyRef.value = ''
  locationRef.value = ''
  typeRef.value = ''
  sourceRef.value = ''
  remunerationRef.value = ''
  urlOffreRef.value = ''
  descriptionOffreRef.value = ''
  notesRef.value = ''
  resetContactDrafts()
  extractedContactKey.value = null
  prefilled.value = false
  saveError.value = null
  offerImport.reset()
  clearDuplicates()
}

const { documentsBlocking, loadDocumentsStatus } = useDocumentsStatus()
onMounted(loadDocumentsStatus)
watch(() => currentSession.value?.id, loadDocumentsStatus)

/** Infobulle du bouton de création quand la session est en pause ou terminée, sinon `null`. */
const lockedHint = computed(() => (currentSessionLocked.value ? t('session.locked.hint') : null))

/** Liste « Prêt à générer ? » : tous ses points conditionnent la création. */
const checklist = computed(() =>
  buildGenerationChecklist({
    fieldsValid: isValid.value,
    documentsReady: !documentsBlocking.value,
    sessionCity: currentSession.value?.ville ?? null,
    sessionLocked: currentSessionLocked.value,
  }),
)

// La fenêtre de capture n'a plus de sens hors de cet écran.
onBeforeUnmount(offerImport.closeCapture)

/** Valide et envoie le formulaire de nouvelle candidature au backend. */
async function submit() {
  const session = currentSession.value
  if (!checklist.value.canGenerate || !session) return
  saving.value = true
  saveError.value = null
  try {
    // 1. Construction de l'input à partir du formulaire
    const input: CreateCandidatureInput = {
      poste: roleRef.value.trim(),
      entreprise: companyRef.value.trim() || null,
      localisation: locationRef.value.trim() || null,
      type_contrat: typeRef.value.trim() || null,
      source: sourceRef.value || null,
      remuneration: remunerationRef.value.trim() || null,
      url_offre: urlOffreRef.value.trim() || null,
      description_offre: descriptionOffreRef.value.trim() || null,
      notes: notesRef.value.trim() || null,
      cv_path: null,
      lettre_motivation_path: null,
      session_id: session.id,
      contacts: state.newContacts.map(draftToCandidatureContact),
    }
    // 2. Création de la candidature côté backend, puis suivi de la génération
    //    des documents lancée en tâche de fond (modale bloquante)
    const candidature = await invoke<CandidatureView>('create_candidature', { input })
    void refreshSessions()
    showProgressModal.value = true
    startProgressPolling(candidature.id)
  } catch (e: unknown) {
    saveError.value = backendErrorMessage(e, t('error.fallback.create_application'))
  } finally {
    saving.value = false
  }
}

/** Domaines de job boards reconnus par `autoDetectSource`. */
const JOB_BOARD_DOMAINS = [
  'linkedin.com',
  'indeed.com',
  'indeed.fr',
  'welcometothejungle.com',
  'apec.fr',
  'monster.fr',
  'hellowork.com',
]

/**
 * Détecte la plateforme source depuis le domaine de l'URL et pré-remplit
 * `sourceRef` avec l'unique option du `<select>` correspondant aux
 * plateformes en ligne (le formulaire n'a pas d'option par plateforme).
 */
function autoDetectSource(url: string) {
  try {
    const hostname = new URL(url).hostname.toLowerCase().replace(/^www\./, '')
    const isJobBoard = JOB_BOARD_DOMAINS.some(
      (domain) => hostname === domain || hostname.endsWith(`.${domain}`)
    )
    if (isJobBoard) {
      sourceRef.value = 'Plateforme en ligne (Indeed, LinkedIn, Welcome to the Jungle...)'
    }
  } catch {
    // URL invalide : on ne pré-remplit pas la source
  }
}

/**
 * Remplit le formulaire avec les champs d'une nouvelle analyse : un champ
 * absent de cette analyse est vidé, pour ne pas mélanger deux offres.
 */
function applyFields(extraction: OfferExtraction) {
  const { fields } = extraction
  roleRef.value = fields.poste ?? ''
  companyRef.value = fields.entreprise ?? ''
  locationRef.value = fields.localisation ?? ''
  typeRef.value = fields.type_contrat ?? ''
  remunerationRef.value = fields.remuneration ?? ''
  descriptionOffreRef.value = fields.description_offre ?? ''
  applyExtractedContact(extraction)
}

/**
 * Pré-remplit une ligne de contact avec le contact nommé par l'offre, en
 * remplaçant celle de l'extraction précédente (les lignes saisies à la main
 * sont conservées). La ligne reste modifiable et supprimable.
 */
function applyExtractedContact(extraction: OfferExtraction) {
  const previousIndex = state.newContacts.findIndex(d => d.key === extractedContactKey.value)
  if (previousIndex !== -1) removeContactDraft(previousIndex)
  extractedContactKey.value = null

  const contact = extraction.fields.contact
  if (!contact) return
  const draft = draftFromScraped(contact)
  addContactDraft(draft)
  extractedContactKey.value = draft.key
}

/** Pré-remplit le formulaire après une extraction réussie et vérifie les doublons. */
function onOfferExtracted(extraction: OfferExtraction) {
  applyFields(extraction)
  prefilled.value = true
  if (urlOffreRef.value.trim()) {
    autoDetectSource(urlOffreRef.value)
    void checkDuplicates(urlOffreRef.value)
  }
}

/** Passe à la saisie manuelle en vérifiant les doublons du lien déjà saisi. */
async function goToManual() {
  prefilled.value = false
  await offerImport.goToManual()
  void checkDuplicates(urlOffreRef.value)
}
</script>

<template>
  <div class="px-5 sm:px-[30px] pt-[26px] pb-12">
    <div class="flex flex-wrap min-[1100px]:flex-nowrap items-center gap-x-6 gap-y-3 mb-6">
      <div class="flex-1 min-w-[200px]">
        <h1 class="text-2xl font-semibold leading-tight">{{ t('new_application.title') }}</h1>
        <div class="text-[13px] text-neutral-500 mt-[5px]">{{ currentSession?.nom }}</div>
      </div>
      <ApplicationStepper :active="step === 'form' ? 1 : 0" />
      <button
        type="button"
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        @click="navigate('tracking')"
      >{{ t('common.cancel') }}</button>
    </div>

    <SessionLockedBanner v-if="currentSession" :statut="currentSession.statut" class="mb-4" />

    <!-- Sans session courante, aucune candidature ne peut être créée -->
    <InfoBanner v-if="!currentSession" tone="warn">
      {{ t('new_application.no_session') }}
      <button
        type="button"
        class="border-0 bg-transparent p-0 text-[12.5px] font-sans text-neutral-500 cursor-pointer underline underline-offset-[3px] hover:text-neutral-950"
        @click="navigate('tracking')"
      >{{ t('new_application.go_to_tracking') }}</button>
    </InfoBanner>

    <OfferLinkStep
      v-if="step === 'url'"
      v-model="urlOffreRef"
      :busy="busy"
      :unavailable="urlUnavailable"
      :error="urlError"
      @analyze="offerImport.analyzeUrl"
      @manual="goToManual"
    />

    <ExtractionChoiceStep
      v-if="step === 'choice' && choice"
      :message="choice.message"
      :can-capture="choice.canCapture"
      :busy="busy"
      @capture="offerImport.openCapture"
      @paste="offerImport.goToPaste"
      @manual="goToManual"
      @back="offerImport.backToUrl"
    />

    <CaptureStep
      v-if="step === 'capture'"
      :busy="busy"
      :error="captureError"
      @capture="offerImport.capturePage"
      @reopen="offerImport.openCapture"
      @paste="offerImport.goToPaste"
      @manual="goToManual"
      @back="offerImport.backToUrl"
    />

    <PasteStep
      v-if="step === 'paste'"
      v-model="pasteText"
      :busy="busy"
      :error="pasteError"
      @analyze="offerImport.analyzePaste"
      @manual="goToManual"
      @back="offerImport.backToUrl"
    />

    <template v-if="step === 'form'">
      <BackToLinkButton class="mb-[14px]" @back="offerImport.backToUrl" />
      <DuplicateOfferWarning :duplicates="duplicates" :check-error="checkError" />

      <div class="grid grid-cols-1 min-[1100px]:grid-cols-[minmax(0,1fr)_360px] gap-5 items-start">
        <div class="min-w-0 flex flex-col gap-4">
          <JobFieldsCard
            v-model:role="roleRef"
            v-model:company="companyRef"
            v-model:location="locationRef"
            v-model:contract="typeRef"
            v-model:salary="remunerationRef"
            v-model:source="sourceRef"
            v-model:url="urlOffreRef"
            :prefilled="prefilled"
            @url-change="checkDuplicates(urlOffreRef)"
          />
          <SectionCard :hint="t('new_application.form.offer_hint')">
            <template #title>{{ t('detail.offer.title') }} <span class="text-red-500 font-semibold">*</span></template>
            <textarea
              v-model="descriptionOffreRef"
              class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[320px] focus:outline-none focus:border-neutral-400"
              :placeholder="t('new_application.form.offer_placeholder')"
              :aria-label="t('detail.offer.title')"
            />
          </SectionCard>
        </div>

        <aside class="flex flex-col gap-4 min-[1100px]:sticky min-[1100px]:top-5">
          <GenerationReadyCard
            :checklist="checklist"
            :saving="saving"
            :error="saveError"
            :locked-hint="lockedHint"
            @submit="submit"
          />
          <ContactsSection />
          <SectionCard :title="t('new_application.form.notes_title')" :hint="t('new_application.form.notes_hint')">
            <textarea
              v-model="notesRef"
              class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-24 focus:outline-none focus:border-neutral-400"
              :placeholder="t('new_application.form.notes_placeholder')"
              :aria-label="t('new_application.form.notes_title')"
            />
          </SectionCard>
        </aside>
      </div>
    </template>

    <GenerationProgressModal
      v-if="showProgressModal"
      :progress="progress"
      :poll-error="pollError"
      @close="closeProgressModal"
    />
  </div>
</template>
