<script setup lang="ts">
import { state, confirmDrag, cancelDrag } from '@/stores/app'
</script>

<template>
  <div v-if="state.dragPopup" class="popup-backdrop" @click="cancelDrag">
    <div class="popup-box" @click.stop>
      <div class="popup-hd">
        <div class="popup-title">Déplacer la candidature</div>
        <button class="popup-close" @click="cancelDrag">×</button>
      </div>

      <div style="font-size:14.5px;font-weight:600;margin-bottom:4px">
        {{ state.dragPopup.card.role }}
      </div>
      <div class="cmeta" style="margin-bottom:18px">
        {{ state.dragPopup.card.company }} · {{ state.dragPopup.card.location }}
      </div>

      <div style="display:flex;align-items:center;gap:12px;margin-bottom:24px">
        <span class="tag">{{ state.dragPopup.fromColTitle }}</span>
        <span style="color:#83837e">→</span>
        <span class="tag">{{ state.dragPopup.toColTitle }}</span>
      </div>

      <div class="popup-foot">
        <button class="btn" @click="cancelDrag">Annuler</button>
        <button class="btn pri" @click="confirmDrag">Confirmer le déplacement</button>
      </div>
    </div>
  </div>
</template>
