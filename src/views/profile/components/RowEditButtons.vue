<script setup lang="ts">
/**
 * Boutons Modifier / Supprimer d'un élément de liste du profil, affichés
 * pendant l'édition de sa carte.
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

const emit = defineEmits<{
  edit: []
  remove: []
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-row gap-[5px] flex-none self-center ml-[6px]">
    <button
      class="w-[30px] h-[30px] rounded-7 border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer flex-none text-neutral-600 hover:bg-surface-3 hover:text-neutral-950 hover:border-neutral-300"
      type="button"
      :title="t('common.edit')"
      @click="emit('edit')"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
      </svg>
    </button>
    <button
      class="w-[30px] h-[30px] rounded-7 flex items-center justify-center cursor-pointer flex-none text-red-600 border border-red-200 bg-red-50 hover:bg-red-100 hover:border-red-300"
      type="button"
      :title="t('common.delete')"
      @click="emit('remove')"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="3 6 5 6 21 6" />
        <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
        <path d="M10 11v6M14 11v6" />
        <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
      </svg>
    </button>
  </div>
</template>
