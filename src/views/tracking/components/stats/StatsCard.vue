<script setup lang="ts">
/**
 * Carte de l'onglet Statistiques : titre, phrase d'explication, contenu.
 */

defineProps<{
  title: string
  hint?: string
}>()
</script>

<template>
  <div class="bg-white border border-neutral-200 rounded-xl px-[18px] py-4 min-w-0">
    <div class="mb-[14px]">
      <div class="text-[13.5px] font-semibold text-neutral-950">{{ title }}</div>
      <span v-if="hint" class="text-[12px] text-neutral-500">{{ hint }}</span>
    </div>
    <slot />
  </div>
</template>
