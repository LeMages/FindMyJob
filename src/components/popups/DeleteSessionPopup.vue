<script setup lang="ts">
import { state, confirmDeleteSession, cancelDeleteSession } from '@/stores/app'
</script>

<template>
  <div v-if="state.deleteSessionPopup" class="popup-backdrop" @click="cancelDeleteSession">
    <div class="popup-box" style="width:440px" @click.stop>
      <div class="popup-hd">
        <div class="popup-title">Supprimer la session</div>
        <button class="popup-close" @click="cancelDeleteSession">×</button>
      </div>

      <div class="sub" style="margin:-4px 0 6px">Cette action est irréversible.</div>
      <div style="background:#f8f8f6;border:1px solid #e6e6e1;border-radius:9px;padding:12px 14px;margin-bottom:20px;font-size:13.5px">
        <strong>{{ state.deleteSessionPopup.nom }}</strong> et toutes ses candidatures seront supprimées définitivement.
      </div>

      <div class="popup-foot">
        <button class="btn" @click="cancelDeleteSession">Annuler</button>
        <button class="btn-refuse" @click="confirmDeleteSession">Supprimer</button>
      </div>
    </div>
  </div>
</template>
