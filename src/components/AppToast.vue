<script setup lang="ts">
/**
 * Notification transitoire globale (confirmation ou erreur), pilotée par
 * `state.toast` / `showToast` du store. Montée une seule fois dans `App.vue`
 * pour survivre aux changements d'écran.
 */

// Importation des modules internes
import { state } from '@/stores/app'
</script>

<template>
  <Teleport to="body">
    <div
      v-if="state.toast"
      class="fixed bottom-6 right-6 z-[1100] max-w-[460px] px-4 py-3 rounded-lg border shadow-[0_8px_24px_rgba(0,0,0,0.16)] text-[13px] leading-[1.45] break-all cursor-pointer"
      :class="state.toast.kind === 'error'
        ? 'bg-red-50 border-red-200 text-red-700'
        : 'bg-surface-3 border-neutral-200 text-neutral-900'"
      role="status"
      @click="state.toast = null"
    >
      {{ state.toast.message }}
    </div>
  </Teleport>
</template>
