<script setup lang="ts">
/**
 * Vue « Suivi » : tableau kanban des candidatures de la session courante,
 * avec statistiques, glisser-déposer entre colonnes et corbeille.
 */

// Importation des modules Vue / framework
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import type { CandidatureView, BoardColumn } from '@/types'
import {
  currentSession,
  currentTrashedList,
  navigate,
  openCard,
  startDrag,
  showDragPopup,
  openTrashPopup,
  closeTrashPopup,
  addSession,
} from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import { KANBAN_COLUMNS, TYPE_CONTRAT_LABELS } from '@/composables/candidature_display'
import DragPopup from '@/views/tracking/components/DragPopup.vue'
import TrashPopup from '@/views/tracking/components/TrashPopup.vue'

const candidatures = ref<CandidatureView[]>([])
const loading = ref(true)
const loadError = ref<string | null>(null)

const sessionId = computed(() => currentSession.value?.id ?? null)

const sessionIsAccepted = computed(() =>
  candidatures.value.some(c => ['offre', 'refus', 'abandon'].includes(c.status))
)

const kanbanBoard = computed<BoardColumn[]>(() =>
  KANBAN_COLUMNS.map(col => ({
    title: col.title,
    cards: candidatures.value
      .filter(c => col.statuses.includes(c.status))
      .map(c => ({
        id: c.id,
        company: c.entreprise ?? '—',
        role: c.poste,
        location: c.localisation ?? '',
        date: c.date_candidature
          ? `Ajouté ${new Date(c.date_candidature).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' })}`
          : `Créé ${new Date(c.created_at).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' })}`,
        meta: TYPE_CONTRAT_LABELS[c.type_contrat ?? ''] ?? '',
      })),
  }))
)

const kanbanStats = computed(() => {
  let total = 0, sent = 0, interviews = 0, accepted = 0
  for (const col of kanbanBoard.value) {
    total += col.cards.length
    if (col.title === 'Candidature envoyée') sent = col.cards.length
    if (col.title === 'Entretien') interviews = col.cards.length
    if (col.title === 'Accepté') accepted = col.cards.length
  }
  return { total, sent, interviews, accepted }
})

/** Charge les candidatures de la session courante depuis le backend. */
async function loadCandidatures() {
  const sid = currentSession.value?.id
  if (!sid) {
    candidatures.value = []
    loading.value = false
    return
  }
  loading.value = true
  loadError.value = null
  try {
    candidatures.value = await invoke<CandidatureView[]>('get_candidatures', {
      sessionId: sid,
    })
  } catch (e: unknown) {
    loadError.value = extractErrorMessage(e, 'Impossible de charger les candidatures.')
  } finally {
    loading.value = false
  }
}

onMounted(loadCandidatures)
watch(sessionId, loadCandidatures)
</script>

<template>
  <!-- Aucune session -->
  <div v-if="!currentSession" class="flex items-center justify-center h-full">
    <button
      class="px-6 py-3 text-[14px] h-9 rounded-lg inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap font-medium border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
      @click="addSession"
    >
      + Créer ma première session
    </button>
  </div>

  <!-- Session sélectionnée -->
  <template v-else>
    <!-- Header -->
    <div class="px-[30px] pt-[26px] pb-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold leading-tight">{{ currentSession.nom }}</h1>
          <div class="text-[13px] text-neutral-500 mt-[5px] font-mono">
            {{ kanbanStats.total }} candidatures · dernière mise à jour {{ new Date().toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' }) }}
          </div>
        </div>
        <div class="flex items-center gap-[10px]">
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
            @click="openTrashPopup"
          >
            <svg width="15" height="16" viewBox="0 0 15 16" fill="none">
              <rect x="2" y="5" width="11" height="10" rx="1.5" stroke="currentColor" stroke-width="1.4" />
              <line x1="1" y1="5" x2="14" y2="5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              <line x1="5.5" y1="2" x2="9.5" y2="2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              <line x1="5.5" y1="8" x2="5.5" y2="12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
              <line x1="9.5" y1="8" x2="9.5" y2="12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
            Corbeille
          </button>
          <button
            class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
            :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': sessionIsAccepted }"
            @click="navigate('new')"
          >
            + Nouvelle candidature
          </button>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono mt-[60px]">Chargement…</div>

    <!-- Load error -->
    <div v-else-if="loadError" class="text-[12px] text-red-500 text-center py-6 font-mono mt-[60px]">{{ loadError }}</div>

    <!-- Content -->
    <template v-else>
      <!-- Kanban board -->
      <div class="grid grid-cols-5 gap-4 px-[30px] pt-[18px] pb-[30px]">
        <div
          v-for="(col, colIndex) in kanbanBoard"
          :key="col.title"
          class="bg-neutral-100 border border-neutral-200 rounded-10 p-3 flex flex-col gap-[10px] min-h-[340px]"
          @dragover.prevent
          @drop.prevent="showDragPopup(colIndex)"
        >
          <div class="flex items-center gap-2 text-[13px] font-semibold px-1 py-0.5">
            <div class="w-[7px] h-[7px] rounded-sm flex-none bg-neutral-300" />
            <span>{{ col.title }}</span>
            <span class="ml-auto font-mono text-[11px] bg-neutral-200 text-neutral-600 rounded-md px-[7px] py-px flex-none">{{ col.cards.length }}</span>
          </div>

          <div
            v-for="(card, cardIndex) in col.cards"
            :key="card.id"
            class="bg-white border border-neutral-200 rounded-9 p-3 cursor-pointer flex flex-col gap-2 hover:border-neutral-300 hover:shadow-[0_2px_10px_rgba(0,0,0,.06)]"
            draggable="true"
            @dragstart="startDrag(colIndex, cardIndex)"
            @click="openCard(card)"
          >
            <div class="flex items-start gap-[9px]">
              <div class="w-[30px] h-[30px] rounded-7 flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
              <div class="flex-1 min-w-0">
                <div class="text-[13.5px] font-semibold leading-[1.25]">{{ card.role }}</div>
                <div class="text-[12px] text-neutral-500 mt-0.5">{{ card.company }} · {{ card.location }}</div>
              </div>
            </div>
            <div class="font-mono text-[11px] text-neutral-500">{{ card.date }}</div>
            <div v-if="card.meta">
              <span class="inline-block font-mono text-[10.5px] border border-neutral-300 rounded-5 px-[7px] py-0.5 text-neutral-700">{{ card.meta }}</span>
            </div>
          </div>

          <div v-if="col.cards.length === 0" class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucune candidature</div>
        </div>
      </div>
    </template>
  </template>

  <!-- Popups -->
  <DragPopup />
  <TrashPopup :list="currentTrashedList" @close="closeTrashPopup" />
</template>
