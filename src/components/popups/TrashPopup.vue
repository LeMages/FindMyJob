<script setup lang="ts">
import { state } from '@/stores/app'
import type { BoardCard } from '@/types'

defineProps<{ list: BoardCard[] }>()
defineEmits<{ close: [] }>()
</script>

<template>
  <div v-if="state.trashPopup" class="popup-backdrop" @click="$emit('close')">
    <div class="popup-box" @click.stop>
      <div class="popup-hd">
        <div class="popup-title">Corbeille</div>
        <button class="popup-close" @click="$emit('close')">×</button>
      </div>

      <div class="sub" style="margin:-6px 0 16px">
        Candidatures refusées ou abandonnées pour cette session.
      </div>

      <div
        v-for="(tc, i) in list"
        :key="i"
        style="display:flex;align-items:center;gap:12px;padding:12px 0;border-bottom:1px solid #f0f0ed"
      >
        <div class="logo ph" style="flex:none" />
        <div style="flex:1">
          <div class="crole">{{ tc.role }}</div>
          <div class="cco">{{ tc.company }} · {{ tc.location }}</div>
          <div class="cmeta" style="margin-top:2px">{{ tc.date }}</div>
        </div>
      </div>

      <div
        v-if="list.length === 0"
        style="text-align:center;padding:20px 0;font-family:'IBM Plex Mono',monospace;font-size:12px;color:#aeaea7"
      >
        Aucune candidature dans la corbeille.
      </div>

      <div class="popup-foot">
        <button class="btn pri" @click="$emit('close')">Fermer</button>
      </div>
    </div>
  </div>
</template>
