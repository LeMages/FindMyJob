<script setup lang="ts">
/**
 * Vue « Détail » d'une candidature : informations du poste, statut et suivi
 * (événements), documents joints, contacts recruteur et historique.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'

// Importation des modules internes
import {
  state,
  selectedCardColumn,
  navigate,
  openDetailPopup,
  removeEvent,
  trashCard,
  acceptCard,
  cancelAcceptation,
  updateDetailCandidature,
} from '@/stores/app'
import { TYPE_CONTRAT_OPTIONS, SOURCE_OPTIONS, statusLabel } from '@/composables/candidature_display'
import { formatFileSize } from '@/composables/format_file_size'
import DetailPopup from '@/views/detail/components/DetailPopup.vue'
import DocumentPreviewModal from '@/views/detail/components/DocumentPreviewModal.vue'
import EditableField from '@/components/EditableField.vue'

// Libellés d'affichage des événements de suivi
const eventLabel = (type: string) => {
  if (type === 'mail_relance') return 'Mail de relance'
  if (type === 'mail_reponse') return 'Mail de réponse'
  return 'Entretien'
}
const eventIcon = (type: string) => {
  if (type === 'mail_relance') return 'ML'
  if (type === 'mail_reponse') return 'MR'
  return 'RV'
}
const eventAction = (type: string) => type === 'entretien' ? 'Préparer →' : 'Rédiger →'
const initials = (nom: string | null) =>
  (nom ?? '').split(' ').map(w => w[0] ?? '').join('').slice(0, 2).toUpperCase()

/** Formate une date ISO pour l'affichage court (ex. "12 juin 2025"). */
const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })

// Les détails du poste et les notes ne sont modifiables que tant que la
// candidature n'a pas encore été engagée (statut "à postuler").
const editable = computed(() => state.detailCandidature?.status === 'a_postuler')

const cvPath = computed(() => state.detailCandidature?.cv_path ?? null)
const cvExists = computed(() => !!cvPath.value && !!state.detailCvInfo?.exists)
const cvSize = computed(() => state.detailCvInfo?.size ?? 0)
const lettrePath = computed(() => state.detailCandidature?.lettre_motivation_path ?? null)
const lettreExists = computed(() => !!lettrePath.value && !!state.detailLettreInfo?.exists)
const lettreSize = computed(() => state.detailLettreInfo?.size ?? 0)

const previewModal = ref<InstanceType<typeof DocumentPreviewModal> | null>(null)

/**
 * Ouvre l'aperçu PDF d'un document joint à la candidature.
 *
 * @param path - Chemin absolu du document.
 * @param label - Titre affiché dans la modale d'aperçu.
 */
function previewDocument(path: string, label: string) {
  previewModal.value?.open(path, label)
}
</script>

<template>
  <div class="px-9 pt-[26px] pb-11">
    <button class="bg-transparent border-none font-mono text-[12px] text-neutral-500 cursor-pointer py-1 mb-[14px] inline-block hover:text-neutral-950" @click="navigate('tracking')">← Retour au suivi</button>

    <div class="mb-1">
      <h1 class="text-2xl font-semibold leading-tight">{{ state.selectedCard?.role }}</h1>
      <div class="text-[13px] text-neutral-500 mt-[5px] font-mono">{{ state.selectedCard?.company }} · {{ state.selectedCard?.location }}</div>
    </div>

    <!-- Loading -->
    <div v-if="state.detailLoading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[40px]">Chargement…</div>

    <!-- Load error -->
    <div v-else-if="state.detailError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[40px]">{{ state.detailError }}</div>

    <div v-else-if="state.detailCandidature" class="grid grid-cols-[1fr_320px] gap-6 mt-[18px] items-start">
      <!-- Left column -->
      <div class="min-w-0">
        <!-- Détails du poste -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="flex items-center justify-between mb-[14px]">
            <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">DÉTAILS DU POSTE</div>
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
              @click="openDetailPopup('offer')"
            >Voir l'offre →</button>
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Intitulé du poste</label>
            <EditableField
              :model-value="state.detailCandidature.poste"
              :editable="editable"
              placeholder="Ex. Stage Data Scientist"
              :on-save="v => updateDetailCandidature({ poste: v })"
            />
          </div>
          <div class="flex gap-[14px]">
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Entreprise</label>
              <EditableField
                :model-value="state.detailCandidature.entreprise"
                :editable="editable"
                placeholder="Nom de l'entreprise"
                :on-save="v => updateDetailCandidature({ entreprise: v })"
              />
            </div>
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Lieu</label>
              <EditableField
                :model-value="state.detailCandidature.localisation"
                :editable="editable"
                placeholder="Ville"
                :on-save="v => updateDetailCandidature({ localisation: v })"
              />
            </div>
          </div>
          <div class="flex gap-[14px]">
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Type de contrat</label>
              <EditableField
                type="select"
                :options="TYPE_CONTRAT_OPTIONS"
                :model-value="state.detailCandidature.type_contrat"
                :editable="editable"
                :on-save="v => updateDetailCandidature({ type_contrat: v })"
              />
            </div>
            <div class="flex-1 mb-[14px]">
              <label class="text-[12px] text-neutral-600 mb-[5px] block">Gratification</label>
              <EditableField
                :model-value="state.detailCandidature.remuneration"
                :editable="editable"
                placeholder="Ex. 1 300 €/mois"
                :on-save="v => updateDetailCandidature({ remuneration: v })"
              />
            </div>
          </div>
          <div class="mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Plateforme de l'offre</label>
            <EditableField
              type="select"
              :options="SOURCE_OPTIONS"
              :model-value="state.detailCandidature.source"
              :editable="editable"
              :on-save="v => updateDetailCandidature({ source: v })"
            />
          </div>
          <div class="mb-0">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">Lien de l'offre</label>
            <EditableField
              :model-value="state.detailCandidature.url_offre"
              :editable="editable"
              placeholder="https://..."
              :on-save="v => updateDetailCandidature({ url_offre: v })"
            />
          </div>
        </div>

        <!-- Notes -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">NOTES</div>
          <EditableField
            type="textarea"
            :model-value="state.detailCandidature.notes"
            :editable="editable"
            placeholder="Notes libres, retours d'entretien, contacts..."
            :on-save="v => updateDetailCandidature({ notes: v })"
          />
        </div>

        <!-- Statut & Suivi -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">STATUT &amp; SUIVI</div>

          <!-- Mail de candidature (always first) -->
          <div class="flex items-center gap-3 bg-neutral-50 border border-neutral-200 rounded-10 px-[14px] py-3 mb-[10px]">
            <div class="w-9 h-9 rounded-9 bg-white border border-neutral-200 flex items-center justify-center text-[15px] flex-none">✉</div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold">Mail de candidature</div>
              <div class="text-[12px] text-neutral-500 font-mono mt-0.5">Par défaut · En attente d'envoi</div>
            </div>
            <button
              class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
              @click="openDetailPopup('mail')"
            >Rédiger le mail →</button>
          </div>

          <!-- Dynamic events -->
          <div v-for="ev in state.detailEvents" :key="ev.id" class="flex items-center gap-3 bg-neutral-50 border border-neutral-200 rounded-10 px-[14px] py-3 mb-[10px]">
            <div class="w-9 h-9 rounded-9 bg-white border border-neutral-200 flex items-center justify-center flex-none font-mono text-[10px] font-bold text-neutral-700">
              {{ eventIcon(ev.type) }}
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold">{{ eventLabel(ev.type) }}</div>
              <div class="text-[12px] text-neutral-500 font-mono mt-0.5">{{ ev.type === 'entretien' ? 'Date à définir' : 'En attente' }}</div>
            </div>
            <div class="flex items-center gap-[10px]">
              <button
                class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
                @click="openDetailPopup('mail')"
              >{{ eventAction(ev.type) }}</button>
              <button
                class="h-9 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100 px-[10px] text-neutral-300"
                @click="removeEvent(ev.id)"
              >✕</button>
            </div>
          </div>

          <button
            class="w-full justify-center mt-1 h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
            @click="openDetailPopup('addEvent')"
          >
            + Ajouter un événement
          </button>
        </div>
      </div>

      <!-- Right column -->
      <div class="sticky top-[26px]">
        <!-- Documents -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DOCUMENTS</div>

          <!-- CV -->
          <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
            <div class="w-7 h-9 rounded-md flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">CV</div>
              <div class="font-mono text-[11px] text-neutral-500">{{ cvExists ? `Joint · ${formatFileSize(cvSize)}` : 'Non existant' }}</div>
            </div>
            <button
              v-if="cvExists"
              class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-neutral-100"
              title="Visualiser"
              @click="previewDocument(cvPath!, 'Aperçu du CV')"
            >
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
          </div>

          <!-- Lettre de motivation -->
          <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
            <div class="w-7 h-9 rounded-md flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">Lettre de motivation</div>
              <div class="font-mono text-[11px] text-neutral-500">{{ lettreExists ? `Joint · ${formatFileSize(lettreSize)}` : 'Non existant' }}</div>
            </div>
            <button
              v-if="lettreExists"
              class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-neutral-100"
              title="Visualiser"
              @click="previewDocument(lettrePath!, 'Aperçu de la lettre de motivation')"
            >
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
          </div>

        </div>

        <!-- Contacts recruteur -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="flex items-center justify-between mb-[14px]">
            <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">CONTACTS RECRUTEUR</div>
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
              @click="openDetailPopup('addRecruiter')"
            >+ Ajouter</button>
          </div>
          <div v-for="rec in state.detailContacts" :key="rec.id" class="flex items-center gap-3 py-[10px] border-b border-neutral-100">
            <div class="w-[34px] h-[34px] rounded-full flex-none bg-neutral-200 flex items-center justify-center font-semibold text-[12px] text-neutral-700">{{ initials(rec.nom) }}</div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">{{ rec.nom ?? '—' }}</div>
              <div class="text-[12px] text-neutral-500 mt-0.5">{{ rec.role ?? '—' }}</div>
              <div class="font-mono text-[11px] text-neutral-500">{{ rec.email ?? '—' }}</div>
            </div>
          </div>
          <div v-if="state.detailContacts.length === 0" class="text-[12px] text-neutral-400 font-mono py-2">Aucun recruteur renseigné.</div>
        </div>

        <!-- Historique -->
        <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">HISTORIQUE</div>
          <div v-if="state.detailHistory.length" class="flex flex-col">
            <div v-for="(entry, i) in state.detailHistory" :key="entry.id" class="flex gap-3">
              <div class="flex flex-col items-center">
                <div class="w-[13px] h-[13px] rounded-full border-2 flex-none mt-0.5 bg-neutral-950 border-neutral-950" />
                <div v-if="i < state.detailHistory.length - 1" class="flex-1 w-0.5 bg-neutral-200 min-h-[22px] my-0.5" />
              </div>
              <div>
                <div class="text-[13px] font-semibold">{{ statusLabel(entry.nouveau_status) }}</div>
                <div class="text-[11.5px] text-neutral-400 mt-px font-mono pb-[14px]">{{ formatDate(entry.changed_at) }}</div>
              </div>
            </div>
          </div>
          <div v-else class="text-[12px] text-neutral-400 font-mono py-2">Aucun historique.</div>
        </div>
      </div>
    </div>
  </div>

  <!-- Action footer -->
  <div v-if="selectedCardColumn === 'Accepté'" class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <div class="flex-1 text-[13px] text-green-700 font-mono">
      ✓ Candidature acceptée — session verrouillée.
    </div>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
      @click="cancelAcceptation"
    >Annuler l'acceptation</button>
  </div>
  <div v-else class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-neutral-100 hover:text-neutral-800"
      @click="trashCard"
    >Refusé entreprise</button>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-neutral-100 hover:text-neutral-800"
      @click="trashCard"
    >Refus de ma part</button>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-950 bg-neutral-950 text-[13px] font-medium text-white inline-flex items-center cursor-pointer font-sans hover:bg-neutral-800"
      @click="acceptCard"
    >Acceptation</button>
  </div>

  <!-- Popups -->
  <DetailPopup />
  <DocumentPreviewModal ref="previewModal" />
</template>
