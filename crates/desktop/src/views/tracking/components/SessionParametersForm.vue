<script setup lang="ts">
/**
 * Formulaire de paramètres de la session courante (onglet Paramètres de la
 * page Candidatures). Titre, contexte, statut et zone cible sont reliés au
 * backend (`sessions`), édités au double-clic (même mécanique que la page
 * détail d'une candidature). Le CV et la lettre de motivation de référence
 * sont propres à cette session (`documents/sessions/<id>/`) et gérés par
 * `DocumentReferenceEditor` en mode "reference".
 */

// Importation des modules Vue / framework
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import {
  currentSession, updateSession,
  sessionSearchQueries, searchRunning, loadSearchQueries, addSearchQuery, removeSearchQuery, runSessionSearch,
} from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { DocumentReferenceView, SessionStatut } from '@/types'
import EditableField from '@/components/EditableField.vue'
import DocumentReferenceEditor from '@/components/DocumentReferenceEditor.vue'

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

/** Sauvegarde le contexte de la session. */
async function saveContexte(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { contexte: v })
}

/** Sauvegarde le statut de la session. */
async function saveStatut(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { statut: v as SessionStatut })
}

/** Sauvegarde la zone cible de la session. */
async function saveZone(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { zone: v })
}

/** Bornes du seuil de score (miroir de la contrainte CHECK en base et de la validation agent). */
const MIN_SCORE_LOWER = 0.05
const MIN_SCORE_UPPER = 1

const minScoreDraft = ref('')
const minScoreError = ref<string | null>(null)

// Resynchronise le champ sur la valeur de la session courante (changement de
// session ou sauvegarde réussie).
watch(
  () => currentSession.value?.min_score,
  (v) => { minScoreDraft.value = v === undefined ? '' : String(v) },
  { immediate: true },
)

/**
 * Valide puis sauvegarde le seuil de score de la session courante. En cas de
 * valeur invalide ou d'échec, affiche l'erreur et rétablit la valeur enregistrée.
 */
async function saveMinScore() {
  const session = currentSession.value
  if (!session) return

  minScoreError.value = null
  const value = Number(minScoreDraft.value)
  if (minScoreDraft.value.trim() === '' || Number.isNaN(value) || value < MIN_SCORE_LOWER || value > MIN_SCORE_UPPER) {
    minScoreError.value = `Le seuil doit être compris entre ${MIN_SCORE_LOWER} et ${MIN_SCORE_UPPER}.`
    minScoreDraft.value = String(session.min_score)
    return
  }
  if (value === session.min_score) return

  const err = await updateSession(session.id, { min_score: value })
  if (err) {
    minScoreError.value = err
    minScoreDraft.value = String(session.min_score)
  }
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

// ---------------------------------------------------------------------------
// Recherche d'offres — requêtes de recherche de la session courante
// ---------------------------------------------------------------------------

const newQueryKeywords = ref('')
const newQueryLocation = ref('')
const searchError = ref<string | null>(null)

watch(sessionId, (id) => {
  if (id !== null) void loadSearchQueries(id)
}, { immediate: true })

const canRunSearch = computed(
  () => sessionSearchQueries.value.length > 0 && currentSession.value?.statut === 'active',
)

/** Ajoute la requête de recherche en cours de saisie à la session courante. */
async function onAddQuery() {
  const session = currentSession.value
  const keywords = newQueryKeywords.value.trim()
  if (!session || !keywords) return

  searchError.value = null
  const err = await addSearchQuery(session.id, {
    keywords,
    location: newQueryLocation.value.trim() || null,
    country_code: null,
  })
  if (err) {
    searchError.value = err
    return
  }
  newQueryKeywords.value = ''
  newQueryLocation.value = ''
}

/** Supprime une requête de recherche de la session courante. */
async function onRemoveQuery(id: number) {
  const session = currentSession.value
  if (!session) return

  const err = await removeSearchQuery(session.id, id)
  if (err) searchError.value = err
}

/** Lance une recherche complète (scraping + filtre + scoring) pour la session courante. */
async function onRunSearch() {
  const session = currentSession.value
  if (!session || !canRunSearch.value) return

  searchError.value = null
  const err = await runSessionSearch(session.id)
  if (err) searchError.value = err
}
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

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Contexte</label>
        <EditableField
          type="textarea"
          :model-value="currentSession?.contexte ?? null"
          :editable="true"
          placeholder="Contexte de la session"
          :on-save="saveContexte"
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

    <!-- Zone géographique -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">ZONE GÉOGRAPHIQUE</div>
      <div>
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Zone cible</label>
        <EditableField
          :model-value="currentSession?.zone ?? null"
          :editable="true"
          placeholder="Ex. Genève / Lausanne — Romandie"
          :on-save="saveZone"
        />
      </div>
    </div>

    <!-- Seuil d'affichage des offres -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">MARCHÉ</div>
      <label class="text-[12px] text-neutral-600 mb-[5px] block">Seuil de score minimum (0.05 à 1)</label>
      <input
        v-model="minScoreDraft"
        type="number"
        :min="MIN_SCORE_LOWER"
        :max="MIN_SCORE_UPPER"
        step="0.05"
        class="w-[120px] h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-mono bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
        @blur="saveMinScore"
        @keyup.enter="saveMinScore"
      />
      <div v-if="minScoreError" class="font-mono text-[12px] text-red-500 mt-[8px]">{{ minScoreError }}</div>
    </div>

    <!-- Recherche d'offres -->
    <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">RECHERCHE D'OFFRES</div>

      <template v-if="sessionSearchQueries.length">
        <div
          v-for="q in sessionSearchQueries"
          :key="q.id"
          class="flex items-center justify-between gap-[10px] py-[10px] border-b border-neutral-100 last:border-b-0"
        >
          <div class="min-w-0">
            <div class="text-[13.5px] font-medium truncate">{{ q.keywords }}</div>
            <div v-if="q.location" class="text-[12px] text-neutral-500 mt-px">{{ q.location }}</div>
          </div>
          <button
            class="w-[30px] h-[30px] rounded-7 flex items-center justify-center cursor-pointer flex-none text-red-600 border border-red-200 bg-red-50 hover:bg-red-100 hover:border-red-300"
            type="button"
            title="Supprimer"
            @click="onRemoveQuery(q.id)"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
              <path d="M10 11v6M14 11v6"/>
              <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
            </svg>
          </button>
        </div>
      </template>
      <div v-else class="text-[12px] text-neutral-400 text-center py-4 font-mono mb-[14px]">Aucune recherche configurée</div>

      <div class="flex gap-[10px] flex-wrap items-end mt-[14px]">
        <div class="flex-1 min-w-[160px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Mots-clés</label>
          <input
            v-model="newQueryKeywords"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            placeholder="Stage Data Engineer"
            @keyup.enter="onAddQuery"
          />
        </div>
        <div class="flex-1 min-w-[140px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Localisation</label>
          <input
            v-model="newQueryLocation"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            placeholder="Paris"
            @keyup.enter="onAddQuery"
          />
        </div>
        <button
          class="h-[38px] px-3 text-[12px] rounded-lg border border-neutral-300 bg-surface-2 font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !newQueryKeywords.trim() }"
          type="button"
          :disabled="!newQueryKeywords.trim()"
          @click="onAddQuery"
        >
          + Ajouter
        </button>
      </div>

      <div v-if="searchError" class="font-mono text-[12px] text-red-500 mt-[14px]">{{ searchError }}</div>

      <button
        class="h-9 px-4 mt-[14px] rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !canRunSearch || searchRunning }"
        type="button"
        :disabled="!canRunSearch || searchRunning"
        @click="onRunSearch"
      >
        {{ searchRunning ? 'Recherche en cours…' : 'Rechercher' }}
      </button>
    </div>

    <!-- Documents -->
    <div v-if="documentsLoadError" class="font-mono text-[12px] text-red-500 mb-[18px]">{{ documentsLoadError }}</div>
    <DocumentReferenceEditor
      doc-type="cv"
      :session-id="sessionId"
      :reference="cvReference"
      @update:reference="v => (cvReference = v)"
    />
    <DocumentReferenceEditor
      doc-type="lettre_motivation"
      :session-id="sessionId"
      :reference="lettreReference"
      @update:reference="v => (lettreReference = v)"
    />
  </div>
</template>
