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
  navigate,
  openDetailPopup,
  openContactPopup,
  openDeleteContactPopup,
  unlinkContact,
  updateDetailCandidature,
} from '@/stores/app'
import { TYPE_CONTRAT_OPTIONS, SOURCE_OPTIONS, TRASHED_STATUSES, statusLabel } from '@/composables/candidature_display'
import { formatFileSize } from '@/composables/format_file_size'
import { useDocumentDownload } from '@/composables/useDocumentDownload'
import DetailPopup from '@/views/detail/components/DetailPopup.vue'
import DocumentPreviewModal from '@/views/detail/components/DocumentPreviewModal.vue'
import StatusTimelineCard from '@/views/detail/components/StatusTimelineCard.vue'
import EditableField from '@/components/EditableField.vue'
import type { DownloadDocType } from '@/composables/document_file_name'

const initials = (nom: string) =>
  nom.split(' ').map(w => w[0] ?? '').join('').slice(0, 2).toUpperCase()

/** Erreur du dernier retrait de contact (affichée dans le bloc des contacts). */
const contactActionError = ref<string | null>(null)

/**
 * Retire un contact de la candidature (il reste dans le carnet).
 *
 * @param contactId - Contact à retirer.
 */
async function removeContactFromCandidature(contactId: number) {
  contactActionError.value = await unlinkContact(contactId)
}

// Les détails du poste et les notes ne sont modifiables que tant que la
// candidature n'a pas encore été engagée (statut "à postuler").
const editable = computed(() => state.detailCandidature?.status === 'a_postuler')

const cvPath = computed(() => state.detailCandidature?.cv_path ?? null)
const cvExists = computed(() => !!cvPath.value && !!state.detailCvInfo?.exists)
const cvSize = computed(() => state.detailCvInfo?.size ?? 0)
const lettrePath = computed(() => state.detailCandidature?.lettre_motivation_path ?? null)
const lettreExists = computed(() => !!lettrePath.value && !!state.detailLettreInfo?.exists)
const lettreSize = computed(() => state.detailLettreInfo?.size ?? 0)
const cvReferenceVersion = computed(() => state.detailCandidature?.cv_reference_version ?? null)
const generationWarnings = computed(() => state.detailCandidature?.generation_warnings ?? [])
const lettreReferenceVersion = computed(() => state.detailCandidature?.lm_reference_version ?? null)

const previewModal = ref<InstanceType<typeof DocumentPreviewModal> | null>(null)
const { downloading, download } = useDocumentDownload()

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
        <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="flex items-center justify-between mb-[14px]">
            <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">DÉTAILS DU POSTE</div>
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
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
        <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
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
        <StatusTimelineCard />
      </div>

      <!-- Right column -->
      <div class="sticky top-[26px]">
        <!-- Documents -->
        <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DOCUMENTS</div>

          <!-- CV -->
          <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
            <div class="w-7 h-9 rounded-md flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">CV</div>
              <div class="font-mono text-[11px] text-neutral-500">{{ cvExists ? `Joint · ${formatFileSize(cvSize)}` : 'Non existant' }}</div>
              <div v-if="cvReferenceVersion != null" class="font-mono text-[11px] text-neutral-400">Généré avec la référence v{{ cvReferenceVersion }}</div>
            </div>
            <button
              v-if="cvExists"
              class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-surface-2"
              title="Visualiser"
              @click="previewDocument(cvPath!, 'Aperçu du CV', 'cv')"
            >
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
            <button
              v-if="cvExists"
              class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none hover:text-neutral-950 hover:bg-surface-2 disabled:opacity-40 disabled:cursor-not-allowed"
              title="Télécharger"
              :disabled="downloading !== null"
              @click="download('cv', cvPath!)"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
            </button>
          </div>

          <!-- Lettre de motivation -->
          <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px] mb-2">
            <div class="w-7 h-9 rounded-md flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">Lettre de motivation</div>
              <div class="font-mono text-[11px] text-neutral-500">{{ lettreExists ? `Joint · ${formatFileSize(lettreSize)}` : 'Non existant' }}</div>
              <div v-if="lettreReferenceVersion != null" class="font-mono text-[11px] text-neutral-400">Générée avec la référence v{{ lettreReferenceVersion }}</div>
            </div>
            <button
              v-if="lettreExists"
              class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none ml-auto hover:text-neutral-950 hover:bg-surface-2"
              title="Visualiser"
              @click="previewDocument(lettrePath!, 'Aperçu de la lettre de motivation', 'lm')"
            >
              <svg width="17" height="13" viewBox="0 0 17 13" fill="none">
                <ellipse cx="8.5" cy="6.5" rx="7.5" ry="5.5" stroke="currentColor" stroke-width="1.4" />
                <circle cx="8.5" cy="6.5" r="2.5" stroke="currentColor" stroke-width="1.4" />
              </svg>
            </button>
            <button
              v-if="lettreExists"
              class="bg-transparent border-none px-[6px] py-1 cursor-pointer text-neutral-400 flex items-center rounded-md flex-none hover:text-neutral-950 hover:bg-surface-2 disabled:opacity-40 disabled:cursor-not-allowed"
              title="Télécharger"
              :disabled="downloading !== null"
              @click="download('lm', lettrePath!)"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
            </button>
          </div>

          <!-- Avertissements de la génération -->
          <div
            v-for="warning in generationWarnings"
            :key="warning"
            class="mt-2 rounded-lg border border-amber-200 bg-amber-50 px-[10px] py-2 text-[12px] leading-[1.4] text-amber-800"
          >
            {{ warning }}
          </div>
        </div>

        <!-- Contacts recruteur -->
        <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
          <div class="flex items-center justify-between mb-[14px]">
            <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">CONTACTS RECRUTEUR</div>
            <button
              class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
              @click="openContactPopup(null)"
            >+ Ajouter</button>
          </div>
          <div v-for="contact in state.detailContacts" :key="contact.id" class="flex items-start gap-3 py-[10px] border-b border-neutral-100">
            <div class="w-[34px] h-[34px] rounded-full flex-none bg-neutral-200 flex items-center justify-center font-semibold text-[12px] text-neutral-700">{{ initials(contact.nom) }}</div>
            <div class="flex-1 min-w-0">
              <div class="text-[13.5px] font-semibold leading-[1.25]">{{ contact.nom }}</div>
              <div v-if="contact.role" class="text-[12px] text-neutral-500 mt-0.5">{{ contact.role }}</div>
              <div v-if="contact.email" class="font-mono text-[11px] text-neutral-500">{{ contact.email }}</div>
              <div v-if="contact.telephone" class="font-mono text-[11px] text-neutral-500">{{ contact.telephone }}</div>
              <div v-if="contact.notes" class="text-[12px] text-neutral-600 mt-1 whitespace-pre-wrap">{{ contact.notes }}</div>
              <div class="flex flex-wrap gap-x-3 mt-1">
                <button class="bg-transparent border-none p-0 text-[11.5px] text-neutral-500 cursor-pointer font-sans hover:text-neutral-950" @click="openContactPopup(contact)">Modifier</button>
                <button class="bg-transparent border-none p-0 text-[11.5px] text-neutral-500 cursor-pointer font-sans hover:text-neutral-950" @click="removeContactFromCandidature(contact.id)">Retirer de la candidature</button>
                <button class="bg-transparent border-none p-0 text-[11.5px] text-neutral-500 cursor-pointer font-sans hover:text-red-600" @click="openDeleteContactPopup(contact)">Supprimer définitivement</button>
              </div>
            </div>
          </div>
          <div v-if="state.detailContacts.length === 0" class="text-[12px] text-neutral-400 font-mono py-2">Aucun contact renseigné.</div>
          <div v-if="contactActionError" class="text-[12.5px] text-red-600 mt-2">{{ contactActionError }}</div>
        </div>
      </div>
    </div>
  </div>

  <!-- Action footer -->
  <div v-if="state.detailCandidature?.status === 'accepte'" class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <div class="flex-1 text-[13px] text-green-700 font-mono">
      ✓ Candidature acceptée — félicitations !
    </div>
  </div>
  <div v-else-if="TRASHED_STATUSES.includes(state.detailCandidature?.status ?? '')" class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <div class="flex-1 text-[13px] text-neutral-500 font-mono">
      Candidature clôturée — {{ statusLabel(state.detailCandidature?.status ?? '') }}.
    </div>
  </div>
  <div v-else-if="state.detailCandidature?.status === 'a_postuler'" class="flex gap-3 px-9 pt-[18px] pb-[26px] border-t border-neutral-200 bg-neutral-50 items-center">
    <button
      class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-red-50 hover:border-red-300 hover:text-red-600"
      @click="openDetailPopup('confirmDelete')"
    >Supprimer</button>
    <button
      class="h-9 px-4 rounded-lg border border-neutral-950 bg-neutral-950 text-[13px] font-medium text-white inline-flex items-center cursor-pointer font-sans hover:bg-neutral-800"
      @click="openDetailPopup('apply')"
    >Postuler</button>
  </div>

  <!-- Popups -->
  <DetailPopup />
  <DocumentPreviewModal ref="previewModal" />
</template>
