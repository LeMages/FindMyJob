<script setup lang="ts">
import {
  state,
  currentSession,
  currentBoard,
  currentStats,
  currentTrashedList,
  sessionIsAccepted,
  navigate,
  openCard,
  startDrag,
  showDragPopup,
  openTrashPopup,
  closeTrashPopup,
} from '@/stores/app'
import DragPopup from '@/components/popups/DragPopup.vue'
import TrashPopup from '@/components/popups/TrashPopup.vue'
</script>

<template>
  <!-- Header -->
  <div class="head">
    <div style="display:flex; align-items:center; justify-content:space-between;">
      <div>
        <h1 class="h1">{{ currentSession?.name }}</h1>
        <div class="sub">
          {{ currentStats.total }} candidatures · dernière mise à jour {{ new Date().toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' }) }}
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

  <!-- Stats -->
  <div class="stats">
    <div class="stat">
      <div class="statn">{{ currentStats.total }}</div>
      <div class="statl">Total</div>
    </div>
    <div class="stat">
      <div class="statn">{{ currentStats.sent }}</div>
      <div class="statl">Envoyées</div>
    </div>
    <div class="stat">
      <div class="statn">{{ currentStats.interviews }}</div>
      <div class="statl">Entretiens</div>
    </div>
    <div class="stat">
      <div class="statn">{{ currentStats.accepted }}</div>
      <div class="statl">Réponses</div>
    </div>
  </div>

  <!-- Kanban board -->
  <div class="board">
    <div
      v-for="(col, colIndex) in currentBoard"
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

  <!-- Popups -->
  <DragPopup />
  <TrashPopup :list="currentTrashedList" @close="closeTrashPopup" />
</template>
