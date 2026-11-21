<script setup lang="ts">
/**
 * Carte de contenu : titre, phrase d'explication facultative et actions à
 * droite de l'en-tête (slot `actions`), puis le contenu. Utilisée par la
 * création d'une candidature et sa fiche.
 */

defineProps<{
  /** Titre de la carte. */
  title?: string
  /** Phrase d'explication sous le titre. */
  hint?: string
}>()
</script>

<template>
  <div class="bg-white border border-neutral-200 rounded-xl px-[18px] py-4 min-w-0">
    <div v-if="title || $slots.title" class="flex items-start justify-between gap-3 mb-[14px]">
      <div class="min-w-0">
        <div class="text-[13.5px] font-semibold text-neutral-950"><slot name="title">{{ title }}</slot></div>
        <span v-if="hint || $slots.hint" class="text-[12px] text-neutral-500 inline-flex items-center gap-[5px]"><slot name="hint">{{ hint }}</slot></span>
      </div>
      <div v-if="$slots.actions" class="flex items-center gap-2 flex-none">
        <slot name="actions" />
      </div>
    </div>
    <slot />
  </div>
</template>
