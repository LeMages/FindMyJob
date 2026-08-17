<script setup lang="ts">
/**
 * Popup de confirmation de déplacement d'une candidature entre colonnes du
 * kanban (voir `state.dragPopup`), propre à `TrackingView`.
 */

// Importation des modules internes
import { state, confirmDrag, cancelDrag } from '@/stores/app'
</script>

<template>
  <div v-if="state.dragPopup" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="cancelDrag">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">Déplacer la candidature</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="cancelDrag">×</button>
      </div>

      <div class="text-[14.5px] font-semibold mb-1">
        {{ state.dragPopup.card.role }}
      </div>
      <div class="font-mono text-[11px] text-neutral-500 mb-[18px]">
        {{ state.dragPopup.card.company }} · {{ state.dragPopup.card.location }}
      </div>

      <div class="flex items-center gap-3 mb-6">
        <span class="inline-block font-mono text-[10.5px] border border-neutral-300 rounded-5 px-[7px] py-0.5 text-neutral-700">{{ state.dragPopup.fromColTitle }}</span>
        <span class="text-neutral-500">→</span>
        <span class="inline-block font-mono text-[10.5px] border border-neutral-300 rounded-5 px-[7px] py-0.5 text-neutral-700">{{ state.dragPopup.toColTitle }}</span>
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="cancelDrag"
        >Annuler</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          @click="confirmDrag"
        >Confirmer le déplacement</button>
      </div>
    </div>
  </div>
</template>
