<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CandidatureView, BoardColumn } from '@/types'
import {
  state,
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
import DragPopup from '@/components/popups/DragPopup.vue'
import TrashPopup from '@/components/popups/TrashPopup.vue'

const TYPE_LABELS: Record<string, string> = {
  stage: 'Stage',
  alternance: 'Alternance',
  cdi: 'CDI',
  cdd: 'CDD',
  freelance: 'Freelance',
}

const KANBAN_COLUMNS: { title: string; statuses: string[] }[] = [
  { title: 'À postuler',          statuses: ['a_postuler'] },
  { title: 'Candidature envoyée', statuses: ['postule'] },
  { title: 'Discussion',          statuses: ['relance'] },
  { title: 'Entretien',           statuses: ['entretien'] },
  { title: 'Accepté',             statuses: ['offre', 'refus', 'abandon'] },
]

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
        company: c.entreprise ?? '—',
        role: c.poste,
        location: c.localisation ?? '',
        date: c.date_candidature
          ? `Ajouté ${new Date(c.date_candidature).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' })}`
          : `Créé ${new Date(c.created_at).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' })}`,
        meta: TYPE_LABELS[c.type_contrat ?? ''] ?? '',
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
    loadError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : typeof e === 'string'
          ? e
          : 'Impossible de charger les candidatures.'
  } finally {
    loading.value = false
  }
}

onMounted(loadCandidatures)
watch(sessionId, loadCandidatures)
</script>

<template>
  <!-- Aucune session -->
  <div v-if="!currentSession" class="no-session">
    <button class="btn pri" style="padding:12px 24px;font-size:14px" @click="addSession">
      + Créer ma première session
    </button>
  </div>

  <!-- Session sélectionnée -->
  <template v-else>
    <!-- Header -->
    <div class="head">
      <div style="display:flex; align-items:center; justify-content:space-between;">
        <div>
          <h1 class="h1">{{ currentSession.nom }}</h1>
          <div class="sub">
            {{ kanbanStats.total }} candidatures · dernière mise à jour {{ new Date().toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' }) }}
          </div>
        </div>
        <div class="btnrow">
          <button class="btn" @click="openTrashPopup">
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
            class="btn pri"
            :class="{ disabled: sessionIsAccepted }"
            @click="navigate('new')"
          >
            + Nouvelle candidature
          </button>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="cempty" style="margin-top:60px">Chargement…</div>

    <!-- Load error -->
    <div v-else-if="loadError" class="cempty" style="margin-top:60px;color:#b94a48">{{ loadError }}</div>

    <!-- Content -->
    <template v-else>
      <!-- Stats -->
      <div class="stats">
        <div class="stat">
          <div class="statn">{{ kanbanStats.total }}</div>
          <div class="statl">Total</div>
        </div>
        <div class="stat">
          <div class="statn">{{ kanbanStats.sent }}</div>
          <div class="statl">Envoyées</div>
        </div>
        <div class="stat">
          <div class="statn">{{ kanbanStats.interviews }}</div>
          <div class="statl">Entretiens</div>
        </div>
        <div class="stat">
          <div class="statn">{{ kanbanStats.accepted }}</div>
          <div class="statl">Réponses</div>
        </div>
      </div>

      <!-- Kanban board -->
      <div class="board">
        <div
          v-for="(col, colIndex) in kanbanBoard"
          :key="col.title"
          class="col"
          @dragover.prevent
          @drop.prevent="showDragPopup(colIndex)"
        >
          <div class="colh">
            <div class="sdot" />
            <span>{{ col.title }}</span>
            <span class="scount">{{ col.cards.length }}</span>
          </div>

          <div
            v-for="(card, cardIndex) in col.cards"
            :key="card.company + card.role"
            class="card"
            draggable="true"
            @dragstart="startDrag(colIndex, cardIndex)"
            @click="openCard(card)"
          >
            <div class="crow">
              <div class="logo ph" />
              <div class="f1">
                <div class="crole">{{ card.role }}</div>
                <div class="cco">{{ card.company }} · {{ card.location }}</div>
              </div>
            </div>
            <div class="cmeta">{{ card.date }}</div>
            <div v-if="card.meta">
              <span class="tag">{{ card.meta }}</span>
            </div>
          </div>

          <div v-if="col.cards.length === 0" class="cempty">Aucune candidature</div>
        </div>
      </div>
    </template>
  </template>

  <!-- Popups -->
  <DragPopup />
  <TrashPopup :list="currentTrashedList" @close="closeTrashPopup" />
</template>

<style scoped>
.no-session {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
</style>
