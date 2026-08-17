<script setup lang="ts">
/**
 * Corbeille de la session courante : liste des candidatures abandonnées
 * (voir `state.trashPopup`), propre à `TrackingView`.
 */

// Importation des modules internes
import { state } from '@/stores/app'
import type { CandidatureView } from '@/types'

defineProps<{ list: CandidatureView[] }>()
defineEmits<{ close: [] }>()

/** Formate la date d'abandon pour l'affichage (ex. "12 juin 2026"). */
function formatAbandonDate(iso: string): string {
  return new Date(iso).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })
}
</script>

<template>
  <div v-if="state.trashPopup" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]" @click="$emit('close')">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">Corbeille</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="$emit('close')">×</button>
      </div>

      <div class="text-[13px] text-neutral-500 font-mono -mt-[6px] mb-4">
        Candidatures abandonnées pour cette session.
      </div>

      <div
        v-for="tc in list"
        :key="tc.id"
        class="flex items-center gap-3 py-3 border-b border-neutral-100"
      >
        <div class="w-[30px] h-[30px] rounded-7 flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
        <div class="flex-1">
          <div class="text-[13.5px] font-semibold leading-[1.25]">{{ tc.poste }}</div>
          <div class="text-[12px] text-neutral-500 mt-0.5">{{ tc.entreprise ?? '—' }}</div>
        </div>
        <div class="font-mono text-[11px] text-neutral-500">Abandon le {{ formatAbandonDate(tc.derniere_maj_statut) }}</div>
      </div>

      <div
        v-if="list.length === 0"
        class="text-center py-5 font-mono text-[12px] text-neutral-400"
      >
        Aucune candidature dans la corbeille.
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          @click="$emit('close')"
        >Fermer</button>
      </div>
    </div>
  </div>
</template>
