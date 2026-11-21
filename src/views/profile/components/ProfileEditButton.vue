<script setup lang="ts">
/**
 * Crayon discret d'une carte du profil : passe la carte en édition.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <button
    class="w-7 h-7 rounded-7 border border-transparent bg-transparent text-neutral-400 flex items-center justify-center cursor-pointer flex-none hover:border-neutral-200 hover:text-neutral-950 hover:bg-neutral-50"
    type="button"
    :title="t('common.edit')"
    :aria-label="t('common.edit')"
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
    </svg>
  </button>
</template>
